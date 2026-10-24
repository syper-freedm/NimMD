# Заключения проверок: Реализовать типы профиля и предобработку текста

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED
- ID: ARCH-001
- Уровень: NOTICE
- Ссылка: AC-1 / Карта реализации
- Проблема: JSON-круг AC-1 требует `serde_json`, а в карте для `Cargo.toml` указан только `serde`.
- Требуемый результат: `serde_json = { workspace = true }` в `[dev-dependencies]` `nimmd-core` (уже есть в `[workspace.dependencies]`).
- ID: ARCH-002
- Уровень: NOTICE
- Ссылка: FR-1
- Проблема: `#[derive(Default)]` для `Profile` требует `Default` у `Dialect`, которого FR-1 не перечисляет.
- Требуемый результат: `#[derive(Default)]` + `#[default] CommonMark` у `Dialect` либо ручной `impl Default for Profile`; оба варианта допустимы.

### Специалист по ИБ
- Статус: APPROVED

- ID: SEC-001
- Уровень: NOTICE
- Ссылка: FR-8 / FR-9
- Проблема: API4 (отказ в обслуживании): панику от индексации срезов запрещает только текст FR-9; в `Cargo.toml` нет `clippy::indexing_slicing` и `clippy::string_slice`, `just lint` её не поймает.
- Требуемый результат: в `text.rs` резать строки через `get`/`split_at_checked`/`char_indices`, при code review проверить отсутствие `s[..n]`; по желанию — `#![deny(clippy::indexing_slicing, clippy::string_slice)]` в модуле.

- ID: SEC-002
- Уровень: NOTICE
- Ссылка: FR-1
- Проблема: проверка ввода: `Deserialize` для `Features` и `Profile` без `#[serde(deny_unknown_fields)]` молча принимает лишние поля, если профиль придёт извне (настройки хоста).
- Требуемый результат: добавить `#[serde(deny_unknown_fields)]` к `Features` и `Profile`; на AC-1 не влияет.

### Тестировщик
- Статус: APPROVED
- AC-1…AC-6 наблюдаемы, детерминированы и в границах; ожидания сверены с FR-2…FR-8 вручную (в т. ч. `"aé"` с `max` = 2 → `"a"`, `"\r\r\n"` → `"\n\n"`). Сценарии — `test-cases.md`, SC-001…SC-011.

- ID: QA-001
- Уровень: NOTICE
- Ссылка: FR-9, FR-2, «Данные, права, ошибки»
- Проблема: нет AC на объявление модулей и запрет `unwrap/expect/panic!` (FR-9), на `Display`/`Error` у `UnknownDialect` (FR-2) и на тотальность функций на произвольной строке.
- Требуемый результат: правка ФД не нужна; проверяются сценариями SC-003, SC-010, SC-011 — реализации и тестам их не пропускать.

- ID: QA-002
- Уровень: NOTICE
- Ссылка: AC-1, AC-2
- Проблема: в каждом AC несколько независимых поведений (дефолт + serde + `FromStr`; `strip_bom` + `detect_line_ending`).
- Требуемый результат: правка ФД не нужна; в `test-cases.md` они разнесены по отдельным сценариям (SC-001…SC-005).

### UX-ревьюер
- Статус: PENDING

## Проверка реализации

### Качество кода
- Статус: APPROVED
- Соответствие FR/AC: CONFIRMED
- Несвязанные изменения: NONE
- ID: ARCH-003
- Уровень: NOTICE
- Ссылка: `.gitignore`
- Проблема: правка `tasks/*/control/` вне ФД и `changed-files.txt`.
- Требуемый результат: не включать в коммит фичи.

### Безопасность кода
- Статус: APPROVED
- Проверены меры ИБ: CONFIRMED
- Без I/O и unsafe; срезы через get/split_at_checked под deny(indexing_slicing, string_slice), deny_unknown_fields есть (SEC-001/002 учтены).
- cargo deny check: ok; semgrep: 0 findings; gitleaks: no leaks.

### Выполнение тестов
- Статус: APPROVED
- `suite-run.backend-tests.json` green, tree `e416b03c` совпал — полный прогон не повторён.
- `cargo test -p nimmd-core --test profile --test text`: 7 + 10 passed; SC-001…SC-010 pass.
- SC-011: `git grep` — 3 совпадения; `just lint` зелёный (`cargo deny` — вне песочницы).

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
