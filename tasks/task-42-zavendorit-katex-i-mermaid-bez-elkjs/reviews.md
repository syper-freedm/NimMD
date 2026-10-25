# Заключения проверок: Завендорить KaTeX и Mermaid без elkjs

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED
- ID: ARCH-001
- Уровень: NOTICE
- Ссылка: Карта реализации
- Проблема: карта относит `packages/vendor/` к backend, а в `stack.py` этот путь входит в область frontend (там же `pnpm -C packages/vendor audit`); `justfile`, `.gitignore`, `ci.yml` не входят ни в одну область.
- Требуемый результат: при реализации учесть, что diff-lint и самопроверки по `packages/vendor/` относятся к frontend; запустить `pnpm -C packages/vendor audit` вручную.

### Специалист по ИБ
- Статус: APPROVED
- Сверено: новых прав, данных пользователя, эндпоинтов и секретов нет; цепочка поставки закрыта точными версиями, `pnpm-lock.yaml` с `integrity`, `--frozen-lockfile`, `ignore-scripts=true`; исключение `/packages/vendor/**` в `.semgrep.yml` уже есть, правила `nimmd-mermaid-strict`/`nimmd-katex-trust` для хостов не ослаблены; `npm-licenses` уже проверяет `packages/vendor`; CI без новых actions, `permissions: contents: read` сохраняется.

### Тестировщик
- Статус: APPROVED
- ID: QA-001
- Уровень: NOTICE
- Ссылка: AC-2
- Проблема: «пустой контекст `node:vm`» не говорит, что загрузка бандла не должна обращаться к `window`/`document`; если обращается, Ожидание недостижимо.
- Требуемый результат: уточнить: `vm.createContext({})`, загрузка без исключения (так проверяет SC-004).

### UX-ревьюер
- Статус: PENDING

## Проверка реализации

### Качество кода
- Статус: APPROVED
- Соответствие FR/AC: CONFIRMED
- Несвязанные изменения: NONE

### Безопасность кода
- Статус: APPROVED
- Проверены меры ИБ: CONFIRMED
- Сверено: lockfile — 144 `integrity`, только реестр, elkjs только в `overrides: '-'`; `.npmrc ignore-scripts=true`, `allowBuilds: esbuild: false`; точные версии; `fetch.sh` под `set -euo pipefail`, без недоверенного ввода; CI — pnpm `@12.15.0`, `permissions` и `uses` не менялись.
- Прогоны: `pnpm -C packages/vendor audit` — 0 уязвимостей; `cargo deny check` — ok (Rust не затронут); gitleaks по 12 файлам — no leaks; semgrep (`.semgrep.yml`, p/javascript, p/typescript, p/rust, p/secrets) — 0 находок; actionlint чисто, zizmor — нет находок (1 ignored `adhoc-packages` с обоснованием).

### Выполнение тестов
- Статус: APPROVED
- Прогоны (suite-run нет, всё запущено заново; pnpm — вне песочницы из-за блокировки store): `just lint` — 0; `just test` и `cargo test --workspace --all-targets` — 0 (17 Rust, 2 node, 0 skip).
- SC-001: `fetch.sh` в чистой копии — 0, `ok: elkjs не найден`, 5 позиций, 60 шрифтов, `why elkjs` пуст. SC-002: `git status` до/после `just vendor` совпал.
- SC-003: ELK 0 против 9; 3749361 < 5494801 байт. SC-004: `initialize`, `run` — function.
- SC-005: код 1, `node_modules/.pnpm/elkjs@0.9.3/…`. SC-006: код 1, `найдена строка org.eclipse.elk`.
- SC-008/009: версии точные, `pnpm@12.15.0`, elkjs только в `overrides`; `--frozen-lockfile` — 0, lockfile не тронут.

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
