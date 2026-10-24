# Тестовые сценарии: Реализовать типы профиля и предобработку текста

## Метаданные
- Issue: #39
- Стек: unit-тесты Rust (`cargo test --workspace --all-targets`) в `crates/nimmd-core` (модули `profile`, `text`); `serde_json` — dev-зависимость из `[workspace.dependencies]`; static — `just lint` и `git grep`.
- Контроль доступа и изоляция данных неприменимы: библиотечный API без ролей и объектов по идентификатору (ФД, «Данные, права, ошибки»).

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001, SC-002, SC-003 |
| AC-2 | SC-004, SC-005 |
| AC-3 | SC-006 |
| AC-4 | SC-007, SC-010 |
| AC-5 | SC-008 |
| AC-6 | SC-009, SC-010 |
| FR-9 (без AC) | SC-011 |

## Сценарии
### SC-001: Профиль по умолчанию
- **Покрывает:** AC-1
- **Уровень:** unit
- **Предусловия:** модуль `profile` собран.
- **Шаги:** `Profile::default()`.
- **Ожидание:** `dialect == Dialect::CommonMark`; все девять полей `features` равны `false`; `Profile::default().features == Features::default()`.
- **Результат выполнения:** pass

### SC-002: JSON-имена диалектов и обратное чтение
- **Покрывает:** AC-1
- **Уровень:** unit
- **Предусловия:** `serde_json` в `[dev-dependencies]`.
- **Шаги:** `serde_json::to_string` каждого из четырёх вариантов `Dialect` и `Profile::default()`; `serde_json::from_str` результата.
- **Ожидание:** `"\"commonmark\""`, `"\"gfm\""`, `"\"multimarkdown\""`, `"\"obsidian\""`; обратное чтение равно исходному значению (и для `Profile`).
- **Результат выполнения:** pass

### SC-003: Разбор диалекта из строки
- **Покрывает:** AC-1
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `"GFM".parse::<Dialect>()`, `"MultiMarkdown"`, `"kramdown"`, `""`, `" gfm"`.
- **Ожидание:** `Ok(Gfm)`, `Ok(MultiMarkdown)`, затем три `Err(UnknownDialect)`; `UnknownDialect` форматируется через `Display` непустой строкой и приводится к `&dyn std::error::Error`.
- **Результат выполнения:** pass

### SC-004: Снятие BOM
- **Покрывает:** AC-2
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `strip_bom("\u{FEFF}# x")`, `strip_bom("# x")`.
- **Ожидание:** `(true, "# x")`, `(false, "# x")`.
- **Результат выполнения:** pass

### SC-005: Преобладающий стиль переносов
- **Покрывает:** AC-2
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `detect_line_ending` на `"a\r\nb"`, `"abc"`, `"a\r\nb\r\nc\nd"`, `"a\r\nb\n"`, `"a\rb\rc\n"`.
- **Ожидание:** `CrLf`, `Lf`, `CrLf`, `Lf`, `Lf`.
- **Результат выполнения:** pass

### SC-006: Нормализация переносов
- **Покрывает:** AC-3
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `normalize_newlines` на `"a\nb"`, `"a\r\nb\rc"`, `"\r\r\n"`.
- **Ожидание:** первая — `matches!(_, Cow::Borrowed("a\nb"))`; вторая — `"a\nb\nc"`; третья — `"\n\n"`.
- **Результат выполнения:** pass

### SC-007: Отделение frontmatter
- **Покрывает:** AC-4
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `split_frontmatter` на девяти строках из AC-4.
- **Ожидание:** `(Some("---\ntitle: x\n---\n"), "# H\n")`, `(Some("---\n---\n"), "")`, `(Some("---\na: 1\n---"), "")`; остальные шесть — `(None, src)`; для всех девяти `fm.unwrap_or("").to_owned() + body == src`.
- **Результат выполнения:** pass

### SC-008: Переопределение диалекта во frontmatter
- **Покрывает:** AC-5
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `frontmatter_dialect` на шести строках из AC-5.
- **Ожидание:** `Some(Gfm)`, `Some(Obsidian)`, затем четыре `None`.
- **Результат выполнения:** pass

### SC-009: Обрезка по границе символа
- **Покрывает:** AC-6
- **Уровень:** unit
- **Предусловия:** —
- **Шаги:** `truncate_at_char_boundary("aé", max)` для `max` = 2, 10, 0, 3; сравнить `MAX_BYTES`.
- **Ожидание:** `("a", true)`, `("aé", false)`, `("", true)`, `("aé", false)`; `MAX_BYTES == 10_485_760`.
- **Результат выполнения:** pass

### SC-010: Тотальность на произвольных строках
- **Покрывает:** AC-4, AC-6
- **Уровень:** unit
- **Предусловия:** набор строк: пустая, только `\r`, только `---`, `"---\n"`, многобайтовые символы (`é`, эмодзи), `\u{FEFF}` в середине.
- **Шаги:** на каждой строке вызвать `strip_bom`, `detect_line_ending`, `normalize_newlines`, `split_frontmatter`, `frontmatter_dialect`; `truncate_at_char_boundary` для каждого `max` от 0 до `len + 1`.
- **Ожидание:** ни одной паники; `split_frontmatter` склеивается обратно в `src`; результат обрезки — префикс `src` длиной `<= max`.
- **Результат выполнения:** pass

### SC-011: Объявление модулей и запреты рабочего кода
- **Покрывает:** FR-9
- **Уровень:** static
- **Предусловия:** реализация на диске.
- **Шаги:** `git grep -n "pub mod profile;\|pub mod text;\|forbid(unsafe_code)" crates/nimmd-core/src/lib.rs`; `just lint`; `cargo test --workspace --all-targets`.
- **Ожидание:** три совпадения; `just lint` и тесты зелёные (clippy `unwrap_used`/`expect_used`/`panic` с `-D warnings` не срабатывают).
- **Результат выполнения:** pass
