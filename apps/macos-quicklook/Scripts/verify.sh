#!/usr/bin/env bash
# Структурная проверка #50 без macOS/Xcode.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
fail=0

need() {
  if [[ ! -e "$1" ]]; then
    echo "missing: $1" >&2
    fail=1
  fi
}

need "$ROOT/QuickLook/PreviewViewController.swift"
need "$ROOT/QuickLook/AssetSchemeHandler.swift"
need "$ROOT/QuickLook/NimmdFFI.swift"
need "$ROOT/QuickLook/Bridging-Header.h"
need "$ROOT/QuickLook/Info.plist"
need "$ROOT/QuickLook/NimMDQuickLook.entitlements"
need "$ROOT/Host/AppDelegate.swift"
need "$ROOT/Host/Info.plist"
need "$ROOT/project.yml"
need "$ROOT/../../crates/nimmd-ffi/include/nimmd.h"

if grep -q 'public.plain-text' "$ROOT/QuickLook/Info.plist"; then
  echo "QLSupportedContentTypes must not include public.plain-text (В3/Р21)" >&2
  fail=1
fi

if ! grep -q 'QLSupportedContentTypes' "$ROOT/QuickLook/Info.plist"; then
  echo "Info.plist: no QLSupportedContentTypes" >&2
  fail=1
fi

if ! grep -q '10 \* 1024 \* 1024' "$ROOT/QuickLook/NimmdFFI.swift"; then
  echo "NimmdFFI: expected maxBytes = 10 MiB" >&2
  fail=1
fi

if ! grep -q 'com.apple.security.network.client' "$ROOT/QuickLook/NimMDQuickLook.entitlements"; then
  echo "entitlements: network.client required (К20)" >&2
  fail=1
fi

if ! grep -q 'HTTPURLResponse' "$ROOT/QuickLook/AssetSchemeHandler.swift"; then
  echo "AssetSchemeHandler: expected HTTPURLResponse" >&2
  fail=1
fi

if ! grep -q 'decidePolicyFor' "$ROOT/QuickLook/PreviewViewController.swift"; then
  echo "PreviewViewController: expected decidePolicyFor (SEC-002)" >&2
  fail=1
fi

if ! grep -q 'readCapped\|read(upToCount' "$ROOT/QuickLook/PreviewViewController.swift"; then
  echo "PreviewViewController: expected capped file read" >&2
  fail=1
fi

python3 - <<'PY' "$ROOT/QuickLook/Info.plist" "$ROOT/Host/Info.plist" "$ROOT/QuickLook/NimMDQuickLook.entitlements" "$ROOT/Host/NimMD.entitlements"
import plistlib, sys
for path in sys.argv[1:]:
    with open(path, "rb") as f:
        plistlib.load(f)
    print("plist ok:", path)
PY

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi
echo "macos-quicklook verify: ok"
