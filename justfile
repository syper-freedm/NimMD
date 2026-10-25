test:
    cargo test --workspace
    node --test "scripts/*.test.mjs"

bench:
    cargo bench -p nimmd-core

# Установка: cargo install cargo-insta
snap:
    cargo insta test --accept -p nimmd-core

# Все проверки кода: стиль, линтеры, инварианты проекта, зависимости, секреты, CI
lint: fmt-check clippy semgrep deny vendor npm-licenses unused-deps secrets workflows

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Инварианты проекта из .semgrep.yml (сеть не нужна)
semgrep:
    semgrep scan --config .semgrep.yml --metrics=off --disable-version-check --error --quiet

# Уязвимости RustSec, лицензии, источники crates (deny.toml); база RustSec скачивается при запуске
deny:
    cargo deny check

# KaTeX и Mermaid без elkjs в packages/vendor/dist; npm-реестр нужен для установки
vendor:
    bash packages/vendor/fetch.sh

# Лицензии npm-зависимостей по тому же белому списку deny.toml; KaTeX и Mermaid в packages/vendor — devDependencies
npm-licenses:
    node scripts/check-npm-licenses.mjs apps/desktop --prod
    node scripts/check-npm-licenses.mjs packages/vendor

unused-deps:
    cargo machete

# Секреты во всей истории git, в выводе значения скрыты
secrets:
    gitleaks git --redact --no-banner

# GitHub Actions: проверяется, когда появится .github/workflows
workflows:
    #!/usr/bin/env bash
    set -euo pipefail
    [ -d .github/workflows ] || exit 0
    actionlint
    zizmor --offline .github/workflows
