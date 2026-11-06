#!/usr/bin/env bash
# Собирает libnimmd_ffi.a (+ nimmd.h) для линковки Quick Look. Только macOS.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="$ROOT/apps/macos-quicklook/Build/ffi"
mkdir -p "$OUT"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "build-ffi.sh: нужен macOS (сейчас $(uname -s)); пропускаю сборку lib." >&2
  exit 0
fi

cd "$ROOT"
targets=()
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  if rustup target list --installed | grep -qx "$t"; then
    targets+=("$t")
  fi
done
if [[ ${#targets[@]} -eq 0 ]]; then
  echo "Нет установленного apple-darwin target (rustup target add …)." >&2
  exit 1
fi

libs=()
for t in "${targets[@]}"; do
  cargo build -p nimmd-ffi --release --target "$t"
  libs+=("$ROOT/target/$t/release/libnimmd_ffi.a")
done

if [[ ${#libs[@]} -eq 1 ]]; then
  cp "${libs[0]}" "$OUT/libnimmd_ffi.a"
else
  lipo -create -output "$OUT/libnimmd_ffi.a" "${libs[@]}"
fi
cp "$ROOT/crates/nimmd-ffi/include/nimmd.h" "$OUT/nimmd.h"
echo "FFI → $OUT/libnimmd_ffi.a"
