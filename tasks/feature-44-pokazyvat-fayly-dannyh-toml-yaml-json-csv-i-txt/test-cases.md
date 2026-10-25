# Тестовые сценарии: Показывать файлы данных TOML, YAML, JSON, CSV и TXT

## Метаданные
- Issue: #44
- Стек: `cargo test` (`crates/nimmd-core/tests/*.rs`), static — `just`/`git`. Доступа и изоляции данных в ядре нет — объектных сценариев нет.
- «Видимый текст» — `<pre class="data">` без тегов, сущности раскрыты.

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001 |
| AC-2 | SC-002 |
| AC-3 | SC-003 |
| AC-4 | SC-004 |
| AC-5 | SC-005 |
| AC-6 | SC-006 |
| AC-7 | SC-007 |
| AC-8 | SC-008 |
| AC-9 | SC-009, SC-010 |
| AC-10 | SC-011, SC-012 |
| AC-11 | SC-013 |

## Сценарии
### SC-001: FileKind по расширению
- **Покрывает:** AC-1
- **Уровень:** unit
- **Предусловия:** входы AC-1.
- **Шаги:** `from_extension`; `FileKind::default()`; serde `Json`.
- **Ожидание:** значения AC-1 по порядку; default `Markdown`; `"json"` ↔ `Json`.
- **Результат выполнения:** pass

### SC-002: TOML/YAML/JSON — подсветка и обвязка
- **Покрывает:** AC-2
- **Уровень:** integration
- **Предусловия:** входы AC-2.
- **Шаги:** `render` с каждым `kind`.
- **Ожидание:** `body` начинается с `<main class="nimmd nimmd-data" data-kind="…">`; в `<pre class="data">` есть `<span class="`, у TOML — `source toml`; видимый текст = вход; в `html` один `<style`, в нём `.c0`.
- **Результат выполнения:** pass

### SC-003: классы JSON-файла = блока ```json```
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** входы AC-3.
- **Шаги:** `render` с `Json` и `Markdown`, множества `class` внутри `<pre>`; `cargo insta test -p nimmd-core`; `git status --short crates/nimmd-core/tests/snapshots`.
- **Ожидание:** множества равны; блок toml содержит `<span class="source toml`; `.snap.new` нет, статус пуст.
- **Результат выполнения:** pass

### SC-004: TXT экранирован
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** вход AC-4.
- **Шаги:** `render` с `Text`; прочитать `data.css`.
- **Ожидание:** `data-kind="text"`; в `body` есть `&lt;b&gt;не тег&lt;/b&gt;`, нет `<b>` и `<span`; `[data-kind="text"]` с `white-space: pre-wrap`.
- **Результат выполнения:** pass

### SC-005: без профиля, frontmatter и скриптов
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** YAML и `assets` AC-5.
- **Шаги:** `render` с `Yaml`.
- **Ожидание:** `profile == Profile::default()`, `frontmatter`, `title` — `None`; в `html` нет `<script`, `<link`.
- **Результат выполнения:** pass

### SC-006: лимит подсветки
- **Покрывает:** AC-6
- **Уровень:** integration
- **Предусловия:** входы AC-6 (1–5: JSON, CSV, `{}`, TXT, длинная строка).
- **Шаги:** `render`: 1, 3, 5 — `Json`, 2 — `Csv`, 4 — `Text`.
- **Ожидание:** 1 — `highlight_limited`, баннер после `</pre>`, `body` содержит `\n{&quot;k&quot;: 1}\n</pre>`, текст = вход; 2 — `highlight_limited`, баннер, `body` содержит `\na,b\n</pre>`; 3, 4 — `false`, баннера нет; 5 — без паники, текст = вход.
- **Результат выполнения:** pass

### SC-007: decode_lossy, BOM, CRLF, обрезка
- **Покрывает:** AC-7
- **Уровень:** integration
- **Предусловия:** входы AC-7.
- **Шаги:** `decode_lossy`; `render` с `Toml`: `not_utf8`; BOM+CRLF; 3 МБ при `max_bytes` 1 МБ.
- **Ожидание:** `("ok \u{FFFD}\n", true)`, `("ok\n", false)`; сразу после `<main …>` — `<div class="banner not-utf8">`; `had_bom`, `CrLf`; `truncated`, `banner truncated`.
- **Результат выполнения:** pass

### SC-008: разделитель CSV
- **Покрывает:** AC-8
- **Уровень:** unit
- **Предусловия:** входы AC-8.
- **Шаги:** `csv::detect_delimiter`.
- **Ожидание:** байты AC-8 по порядку.
- **Результат выполнения:** pass

### SC-009: раскраска CSV
- **Покрывает:** AC-9
- **Уровень:** integration
- **Предусловия:** строки AC-9.
- **Шаги:** `render` с `Csv`.
- **Ожидание:** фрагменты AC-9; у `a,,b` нет `class="c1"`; нет `<x>`; без паники; текст = вход.
- **Результат выполнения:** pass

### SC-010: фикстуры CSV
- **Покрывает:** AC-9
- **Уровень:** integration
- **Предусловия:** `tests/fixtures/data/*.csv`, есть `semicolon.csv`.
- **Шаги:** `render` с `Csv` каждой.
- **Ожидание:** текст = вход после нормализации; в `semicolon.csv` есть `<span class="d">;</span>`.
- **Результат выполнения:** pass

### SC-011: дамп и лицензия
- **Покрывает:** AC-10
- **Уровень:** static
- **Предусловия:** чистое дерево.
- **Шаги:** `just syntaxes`; `git status --short crates/nimmd-core/assets`; первая строка `LICENSE`; `cargo tree -p nimmd-core -d -e normal`; `git grep` AC-10.
- **Ожидание:** `TOML: ok`, статус пуст; в строке хэш FR-4; `syntect` не дублирован; `git grep` пуст.
- **Результат выполнения:** pass

### SC-012: highlight данных, линтеры, тесты
- **Покрывает:** AC-10
- **Уровень:** integration
- **Предусловия:** входы AC-10.
- **Шаги:** `highlight("toml", "a = 1")`, `highlight("yaml", "a: 1")`, `highlight("json", "{}")`; `just lint`; `cargo test --workspace`.
- **Ожидание:** `Vec` непусты; команды — код 0.
- **Результат выполнения:** pass

### SC-013: спаны редактора
- **Покрывает:** AC-11
- **Уровень:** unit
- **Предусловия:** входы AC-11; 3 МБ — ASCII.
- **Шаги:** `highlight` каждого.
- **Ожидание:** первые три непусты; `start < end`, порядок, без пересечений, `end ≤` длины UTF-16; во втором последний `end ≤ 12`; спан с `start == 0` — `keyword` или `storage`; `nosuchlang` — `text plain`; пустой код — пустой; 3 МБ — без паники, `end ≤ 2097152`.
- **Результат выполнения:** pass
