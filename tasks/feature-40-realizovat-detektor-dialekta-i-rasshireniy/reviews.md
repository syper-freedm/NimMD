# Заключения проверок: Реализовать детектор диалекта и расширений

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED

- ID: ARCH-001
- Уровень: NOTICE
- Ссылка: FR-1 / FR-2
- Проблема: не задано предусловие входа `detect`: `split_frontmatter` ждёт текст без BOM и с `\n`; при `\r\n` frontmatter не найдётся, а `\r` в конце строки ломает проверки «оканчивается на `||`/`{#id}`».
- Требуемый результат: в doc-комментарии `detect` зафиксировать предусловие (как у `split_frontmatter`) либо нормализовать вход внутри (`strip_bom` + `normalize_newlines`, `Cow` без копии при `\n`).

- ID: ARCH-002 · CLOSED · FR-9
- ID: ARCH-003 · CLOSED · FR-10, AC-14
- Дельта (FR-10, AC-14, карта): `limit_nesting` в `text.rs` — ок.

- ID: ARCH-005
- Уровень: NOTICE
- Ссылка: FR-10 / AC-14
- Проблема: предел считает маркеры одной строки, а глубина списков копится через отступ продолжения: строка k = отступ (табы) + 64 `- ` даёт 64·k уровней, 10 МБ ≈ 50 000. «Глубина не больше `MAX_NESTING`» неверно.
- Требуемый результат: AC-14(г) с такой лестницей под тем же лимитом 1 с; если медленно — считать и глубину продолжения.

### Специалист по ИБ
- Статус: APPROVED
- ID: SEC-001
- Уровень: NOTICE
- Ссылка: FR-9 / раздел «Данные, права, ошибки»
- Проблема: API4 (потребление ресурсов). Обход AST comrak для `math`/`mermaid` не оговорён; рекурсивный обход на недоверенном входе с глубокой вложенностью (тысячи `>` или вложенных списков в 10 МБ) может переполнить стек и уронить Quick Look — нарушение тотальности FR-1.
- Требуемый результат: обходить дерево итеративно (`root.descendants()` из arena_tree comrak), без собственной рекурсии; в тесты тотальности добавить глубоко вложенную цитату.
- Дельта (FR-10, AC-14): SEC-002 закрыт в ФД.

### Тестировщик
- Статус: APPROVED
- QA-001, QA-002 закрыты прошлой дельтой.
- QA-003, QA-004 закрыты: AC-14(в) — `> > > текст`, ждёт `CommonMark`, все false (согласно FR-6); `limit_nesting` публична в `text.rs`, SC-015 — integration.

### UX-ревьюер
- Статус: PENDING

## Проверка реализации

### Качество кода
- Статус: APPROVED
- Соответствие FR/AC: CONFIRMED
- Несвязанные изменения: NONE
- Дельта: `text::limit_nesting` (FR-10) перед comrak в `detect`, линейна, `Cow` без копии — ок; clippy, тесты ok.
- ID: ARCH-004
- Уровень: NOTICE
- Ссылка: deny.toml / FR-9
- Проблема: файла нет в карте; `[graph] targets` и ignore RUSTSEC-2025-0141 — следствие comrak с `syntect-fancy` из workspace.
- Требуемый результат: внести в карту.

### Безопасность кода
- Статус: APPROVED
- Проверены меры ИБ: CONFIRMED
- Сканеры: semgrep `.semgrep.yml` 0, gitleaks 0; cargo deny — ok в круге 1, повтор не запущен (песочница)
- SEC-002 · CLOSED: `limit_nesting` до comrak; debug: 60k `>` — 7 мс, 1 МБ `>` — 43 мс, 10 МБ строк по 200 `>` — 0,8 с.

### Выполнение тестов

- Статус: APPROVED
- Полный: suite-run green, tree совпал.
- `cargo test -p nimmd-core --test detect --test text`: 26+14 ok, 0 ignored.
- SC-001…013: `plain_commonmark`, `gfm_table_task_strike`, `footnote_is_gfm_not_multimarkdown`, `obsidian_unambiguous_signs`, `obsidian_weak_signs`, `multimarkdown_signs`, `bare_autolink`, `signs_inside_code_fences_are_ignored`, `issue_reference_and_heading_are_not_tags`, `math_from_ast`, `mermaid_from_ast`, `frontmatter_overrides_dialect_only`, `edge_inputs_are_total`.
- SC-014: `just clippy` ok; semgrep 0; grep пуст.
- SC-015: `deep_nesting_is_fast`, `limit_nesting_deep_prefix`, `_boundary`.
- QA-005 · CLOSED: SC-016 `fixtures_match_profiles` ok, 12 пар сверены с AC; suite-run green, tree совпал; clippy ok.

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
