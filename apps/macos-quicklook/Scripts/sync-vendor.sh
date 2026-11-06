#!/usr/bin/env bash
# Копирует KaTeX/Mermaid в Resources/Vendor appex. Сначала: just vendor
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SRC="$ROOT/packages/vendor/dist"
DST="$ROOT/apps/macos-quicklook/QuickLook/Vendor"
if [[ ! -f "$SRC/katex.min.js" || ! -f "$SRC/mermaid.min.js" ]]; then
  echo "sync-vendor: нет $SRC (запустите: just vendor)" >&2
  exit 1
fi
rm -rf "$DST"
mkdir -p "$DST"
cp "$SRC/katex.min.css" "$SRC/katex.min.js" "$SRC/mermaid.min.js" "$DST/"
if [[ -d "$SRC/fonts" ]]; then
  cp -R "$SRC/fonts" "$DST/fonts"
fi
echo "Vendor → $DST"
