# Заключения проверок: Показывать файлы данных TOML, YAML, JSON, CSV и TXT

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED

- ID: ARCH-001
- Уровень: NOTICE
- Ссылка: FR-4 / AC-10
- Проблема: фичи `syntect` задаёт корневой `Cargo.toml` (строка 13: `default-syntaxes`, `html`, `regex-fancy`), а крейт берёт `workspace = true`. Фичи только добавляются, поэтому «без `default-syntaxes`» не получится, пока не правлен корневой `Cargo.toml`, а его нет в карте реализации. Кроме того, `parsing` в syntect 5.3 сам включает `dump-load` и `dump-create`.
- Требуемый результат: при реализации поправить фичи в корневом `Cargo.toml` (это правка по FR-4); `dump-create` в normal-дереве останется через `parsing`, это нормально.

- ID: ARCH-002
- Уровень: NOTICE
- Ссылка: FR-4
- Проблема: в `syntect::dumps` (5.3) сказано, что `SyntaxSet` не надо сжимать второй раз: ленивые синтаксисы в нём уже сжаты, и `load_defaults_newlines` читает его через `from_uncompressed_data`. `from_reader` по сжатому дампу почти не уменьшает размер и добавляет распаковку при первом рендере в Quick Look.
- Требуемый результат: лучше `dump_to_uncompressed_file` + `from_uncompressed_data` (без паники, возвращает `Result`); на AC не влияет.

### Специалист по ИБ
- Статус: APPROVED

### Тестировщик
- Статус: APPROVED
- Дельта (FR-4, FR-7, FR-10, FR-11, AC-6, AC-10, Границы, Данные): QA-001 и QA-002 закрыты, новых находок нет; SC-006 (CSV с `Csv` — в `Действие` AC-6 не назван) и SC-012 обновлены. `parsing` syntect 5.3 включает `dump-load`, `from_uncompressed_data` есть.
- AC-1…AC-11 проверяемы, детерминированы, в границах; `test-cases.md` — SC-001…SC-013. Сверено с кодом: `&quot;` из `push_escaped` (AC-6), `<pre class="syntax-highlighting"><code>` без класса (AC-3), дамп syntect 5.3 сериализует `HashMap` упорядоченно (`ordered_map`) — AC-10 «дамп не меняется» выполним.

- ID: QA-001
- Уровень: NOTICE
- Ссылка: FR-7, AC-8
- Проблема: при равной частоте нескольких чисел полей мода не определена; кейсы AC-8 от этого не зависят.
- Требуемый результат: при ничьей брать, например, большее число полей.

- ID: QA-002
- Уровень: NOTICE
- Ссылка: FR-6, AC-6, AC-10
- Проблема: лимит подсветки для CSV не проверяется ни одним AC; у AC-10 нет входа для `highlight` (SC-012 берёт входы AC-2).
- Требуемый результат: добавить CSV > 2 МБ в AC-6 и вход в AC-10.

### UX-ревьюер
- Статус: PENDING

## Проверка реализации

### Качество кода
- Статус: APPROVED
- Соответствие FR/AC: CONFIRMED
- Несвязанные изменения: NONE
- `deny.toml` (RUSTSEC-2024-0320, yaml-rust из dev `yaml-load`) — по FR-4/AC-10. clippy `-D warnings`, fmt, `cargo test --workspace` (снапшоты Markdown) зелёные; `cargo tree -d -e normal` пуст.

### Безопасность кода
- Статус: APPROVED
- Проверены меры ИБ: CONFIRMED
- `cargo deny check` ok; semgrep и gitleaks по файлам правки — 0 находок. Текст данных экранирован, атрибуты — константы; `yaml-rust` (RUSTSEC-2024-0320) только в dev-дереве.

### Выполнение тестов
- Статус: APPROVED
- `suite-run.backend-tests.json`: green, `tree` = текущему — полный прогон не повторён. Фокусно: `--test data/csv/highlight/text` 42/42, `--test render/document` (снапшоты) ok; `just syntaxes` — `TOML: ok`, дамп не изменился; `cargo tree -d` пуст; `git grep --untracked` пуст; `just lint` зелёный. SC-001…SC-013 — pass.

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
