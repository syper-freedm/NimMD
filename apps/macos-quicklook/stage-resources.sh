#!/bin/bash
set -euo pipefail

resources="$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH"
mkdir -p "$resources"
cp "$NIMMD_VENDOR_DIR/katex.min.css" "$NIMMD_VENDOR_DIR/katex.min.js" \
  "$NIMMD_VENDOR_DIR/mermaid.min.js" "$resources/"
cp "$SRCROOT/../../packages/theme/katex-init.js" \
  "$SRCROOT/../../packages/theme/mermaid-init.js" "$resources/"
ditto "$NIMMD_VENDOR_DIR/fonts" "$resources/fonts"
