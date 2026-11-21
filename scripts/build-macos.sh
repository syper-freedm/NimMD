#!/bin/bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
root="$PWD"

fail() {
    printf '%s\n' "$1" >&2
    exit 1
}

[[ $(uname -s) == Darwin ]] || fail 'Для сборки NimMD.app нужна macOS.'
if [[ -z ${TAURI_SIGNING_PRIVATE_KEY:-} ]]; then
    key_path=${TAURI_SIGNING_PRIVATE_KEY_PATH:-}
    [[ $key_path == /* && -f $key_path && -r $key_path ]] ||
        fail 'Укажите ключ архива: TAURI_SIGNING_PRIVATE_KEY или абсолютный путь TAURI_SIGNING_PRIVATE_KEY_PATH.'
fi

for tool in cargo cbindgen cargo-about just node pnpm rustc xcodebuild codesign hdiutil; do
    command -v "$tool" >/dev/null || fail "Требуется $tool."
done

target=$(rustc -vV | sed -n 's/^host: //p')
case "$target" in
    aarch64-apple-darwin) arch=arm64 ;;
    x86_64-apple-darwin) arch=x86_64 ;;
    *) fail "Не поддерживается нативная цель Rust $target." ;;
esac

release_root="$root/target/macos-release"
release="$release_root/$target"
lock="$release_root/.build.lock"
mkdir -p "$release_root"
mkdir "$lock" || fail "В этой копии уже выполняется сборка macOS ($lock)."
stage=''
cleanup() {
    if [[ -n "$stage" ]]; then
        if [[ -d "$stage/previous" && ! -e "$release" ]] && ! mv "$stage/previous" "$release"; then
            printf 'Предыдущий выпуск сохранён в %s\n' "$stage/previous" >&2
        else
            rm -rf "$stage"
        fi
    fi
    rmdir "$lock"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
stage=$(mktemp -d "$release_root/.build.XXXXXX")

(
    # Сборочные процессы не получают ключ обновления.
    unset TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PATH TAURI_SIGNING_PRIVATE_KEY_PASSWORD
    just desktop-deps vendor
    target_dir=$(cargo metadata --no-deps --format-version 1 | node -p \
        'JSON.parse(require("node:fs").readFileSync(0, "utf8")).target_directory')
    version=$(node -p 'require("./apps/desktop/src-tauri/tauri.conf.json").version')
    cargo build --locked --release --target "$target" -p nimmd-ffi
    mkdir "$stage/ffi"
    ln -s "$target_dir/$target/release/libnimmd_ffi.a" "$stage/ffi/libnimmd_ffi.a"
    cbindgen --config crates/nimmd-ffi/cbindgen.toml --crate nimmd-ffi --output "$stage/ffi/nimmd_ffi.h"

    xcodebuild -project apps/macos-quicklook/NimMDQuickLook.xcodeproj \
        -scheme NimMDQuickLook -configuration Release -destination 'generic/platform=macOS' \
        -derivedDataPath "$stage/xcode" ARCHS="$arch" ONLY_ACTIVE_ARCH=NO CODE_SIGNING_ALLOWED=NO \
        CONFIGURATION_BUILD_DIR="$stage/products" NIMMD_FFI_DIR="$stage/ffi" \
        MARKETING_VERSION="$version" CURRENT_PROJECT_VERSION="$version" build

    appex="$stage/products/NimMDQuickLook.appex"
    [[ -d "$appex" ]] || fail 'Xcode не собрал NimMDQuickLook.appex.'
    node -e 'console.log(JSON.stringify({bundle: {createUpdaterArtifacts: false,
        macOS: {files: {"PlugIns/NimMDQuickLook.appex": process.argv[1]}}}}))' "$appex" \
        > "$stage/tauri.conf.json"
    pnpm -C apps/desktop tauri build --target "$target" --bundles app --no-sign --ci \
        --config "$stage/tauri.conf.json" -- --locked

    mkdir "$stage/release"
    ditto "$target_dir/$target/release/bundle/macos/NimMD.app" "$stage/release/NimMD.app"
)

app="$stage/release/NimMD.app"
bash scripts/sign-macos.sh "$app"
version=$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$app/Contents/Info.plist")
archive="$stage/release/NimMD.app.tar.gz"
tar -czf "$archive" -C "$stage/release" NimMD.app
pnpm -C apps/desktop tauri signer sign --app-version "$version" "$archive"
[[ -s "$archive.sig" ]] || fail 'Не создана подпись архива обновления.'

mkdir "$stage/dmg"
ditto "$app" "$stage/dmg/NimMD.app"
ln -s /Applications "$stage/dmg/Applications"
hdiutil create -volname NimMD -srcfolder "$stage/dmg" -format UDZO "$stage/release/NimMD.dmg"

# Только полный комплект заменяет результат предыдущего запуска.
if [[ -d "$release" ]]; then
    mv "$release" "$stage/previous"
fi
mv "$stage/release" "$release"
printf 'Артефакты сборки: %s\n' "$release"
if [[ -z ${CI:-} ]] && ! (qlmanage -r && qlmanage -r cache); then
    printf 'Не удалось обновить кэш Quick Look.\n' >&2
fi
