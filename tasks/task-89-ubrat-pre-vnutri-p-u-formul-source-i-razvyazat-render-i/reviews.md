# Заключения проверок: Убрать <pre> внутри <p> у формул Source и развязать render и document

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED

### Специалист по ИБ
- Статус: APPROVED

### Тестировщик
- Статус: APPROVED
- ID: QA-001
- Уровень: NOTICE
- Ссылка: AC-4
- Проблема: `contract_core_classes_appear_in_output` сверяет с выводом жёсткий список, а не весь «Только ядра»: `math-display` в контракте им не проверяется.
- Требуемый результат: проверить `math-display` в `html_classes` вывода Source (SC-001) или добавить его в список теста.

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
- Ручная проверка: класс `math-display` — константа, TeX идёт через `push_escaped` (AC-3), фолбэк MathML → `source`; новых точек входа, `unsafe`, I/O, `unwrap` нет; манифесты и lock не менялись.
- Сканеры: semgrep (`.semgrep.yml`, p/rust, p/secrets…) по изменённым исходникам — 0 находок; gitleaks по `changed-files.txt` — no leaks; `cargo deny check bans licenses sources` — ok.
- ID: SEC-001
- Уровень: NOTICE
- Ссылка: `cargo deny check`
- Проблема: проверка advisories не отработала: песочница не дала взять `~/.cargo/advisory-dbs/db.lock` (read-only). Зависимости правка не меняет, так что новых уязвимостей она внести не могла.
- Требуемый результат: `just lint` в CI или вне песочницы выполняет advisories.

### Выполнение тестов

- Статус: APPROVED
- Полный прогон: `suite-run.backend-tests.json` green, `tree` a2cd516 = текущему — не повторялся.
- Фокусно: `cargo test -p nimmd-core --test document` — 32 passed (`math_source_mode`, `display_math_has_no_pre_in_any_mode`, `dangerous_display_formulas_escaped_in_source`, `math_mathml_mode`, `class_contract`, `contract_core_classes_appear_in_output`, `rendered_serializes_lowercase`).
- Статика: diff снапшота MathML пуст; `crate::document` в `render.rs` нет; `enum MathMode` — только `render.rs:29`; строки `use` в тестах не менялись.
- `just lint` — exit 0 (вне песочницы: semgrep пишет в `~/.semgrep`; advisories ok).

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
