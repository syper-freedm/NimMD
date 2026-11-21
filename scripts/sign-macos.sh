#!/bin/bash
set -euo pipefail

if [[ $# != 1 || ! -d $1/Contents/PlugIns/NimMDQuickLook.appex ]]; then
    printf 'Ожидается путь к NimMD.app с вложенным NimMDQuickLook.appex.\n' >&2
    exit 1
fi

app=$1
entitlements="$(dirname "${BASH_SOURCE[0]}")/../apps/macos-quicklook/NimMDQuickLook.entitlements"
codesign --force --sign - --entitlements "$entitlements" "$app/Contents/PlugIns/NimMDQuickLook.appex"
codesign --force --sign - "$app"
codesign --verify --deep --strict "$app"
