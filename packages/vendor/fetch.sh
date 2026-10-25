#!/usr/bin/env bash
# Сборка KaTeX и Mermaid (без elkjs) в packages/vendor/dist; запуск: just vendor
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
pnpm install --frozen-lockfile
rm -rf dist
mkdir dist
cp node_modules/katex/dist/katex.min.css node_modules/katex/dist/katex.min.js dist/
cp -R node_modules/katex/dist/fonts dist/
node build-mermaid.mjs
node check-mermaid.mjs
