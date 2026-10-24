# Функциональный дизайн: Реализовать типы профиля и предобработку текста

## Метаданные [@for: all]
- Issue: #39
- Size: M
- Владелец: Системный аналитик

## Цель [@for: all]
- Дать ядру `nimmd-core` типы профиля документа и чистые функции предобработки текста: снять BOM, определить стиль переносов, нормализовать переносы, отделить frontmatter, прочитать переопределение диалекта и обрезать вход по границе символа. На них строятся детектор (#40) и рендер (#41, #45).
- Источники: спека `docs/superpowers/specs/2026-10-19-nimmd-design.md` разделы 5.1 (Р28), 5.4 «Кодировки», 6; план ядра Task 1.

## Границы [@for: all]
- Входит: модули `profile` и `text` крейта `nimmd-core` с публичным API ниже; зависимость `serde` (derive) в крейте.
- Не входит: `detect()` и применение переопределения `markdown:` к профилю (#40), `render()` и вызов обрезки из него (#41, #45), `+++`-frontmatter (спека 3, Р19), YAML-парсер (спека 6, М7), перекодировка не-UTF-8 (делает хост, 5.2).

## Целевое поведение [@for: all]
- Хост ядра (позже — `render`) получает из строки UTF-8: признак и срез без BOM, стиль переносов для сохранения, текст с переносами `\n`, frontmatter отдельно от тела, диалект из ключа `markdown:` и вход не длиннее лимита. Все функции чистые, без паник и аллокаций там, где их можно избежать.

## Требования [@for: all]
- **FR-1:** `profile.rs`: `enum Dialect { CommonMark, Gfm, MultiMarkdown, Obsidian }`; `struct Features` с девятью `pub bool`: `frontmatter, math, mermaid, footnotes, tables, tasklists, strikethrough, wikilinks, callouts`; `struct Profile { pub dialect: Dialect, pub features: Features }`. Все три — `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`; `Features` и `Profile` — `Default` (`CommonMark`, все флаги `false`).
- **FR-2:** `Dialect` сериализуется serde в нижнем регистре: `"commonmark"`, `"gfm"`, `"multimarkdown"`, `"obsidian"`. `impl FromStr for Dialect` принимает те же четыре строки без учёта регистра ASCII; любая другая строка (в том числе пустая и с пробелами по краям) — `Err(UnknownDialect)`, где `UnknownDialect` — публичная единичная структура с `Debug`, `Display` и `std::error::Error`.
- **FR-3:** `text::strip_bom(src) -> (bool, &str)`: если строка начинается с `U+FEFF`, отрезает ровно один BOM и возвращает `true`; иначе `(false, src)`.
- **FR-4:** `enum LineEnding { Lf, CrLf }` (`Debug, Clone, Copy, PartialEq, Eq`); `detect_line_ending(src)` считает пары `\r\n` и одиночные `\n` (без `\r` перед ним) и возвращает преобладающий стиль; при равенстве, в том числе без переносов, — `Lf`. Одиночный `\r` не считается.
- **FR-5:** `normalize_newlines(src) -> Cow<str>`: `\r\n` и одиночный `\r` → `\n`; строка без `\r` возвращается как `Cow::Borrowed` без копирования.
- **FR-6:** `split_frontmatter(src) -> (Option<&str>, &str)` по Р28. Вход уже без BOM и с переносами `\n`. Открывающая строка — первая строка, равная ровно `---`. Закрывающая — первая следующая строка, равная ровно `---` (пробелы в строке делают её обычной). Frontmatter — точный префикс `src` от начала до конца закрывающей строки вместе с её `\n`; если закрывающая строка последняя и без `\n`, frontmatter — вся строка, тело пустое. Нет открывающей или закрывающей строки, или первая строка `+++` → `(None, src)`. Всегда `frontmatter + тело == src`.
- **FR-7:** `frontmatter_dialect(frontmatter) -> Option<Dialect>`: построчно ищет первую строку вида `markdown:<пробелы><значение><пробелы>`, где ключ начинается с первого символа строки, в нижнем регистре, пробелы и табуляции допустимы перед и после `:` и в конце строки, значение — одно слово из букв, цифр и `_`. Значение разбирается по FR-2. Нет такой строки или значение не диалект → `None`. Строка с ключом не в начале (`title: markdown: gfm`, отступ) не подходит. Без регулярных выражений и YAML-парсера.
- **FR-8:** `const MAX_BYTES: usize = 10 * 1024 * 1024`; `truncate_at_char_boundary(src, max) -> (&str, bool)`: при `src.len() <= max` → `(src, false)`; иначе самый длинный префикс не длиннее `max` байт, кончающийся на `is_char_boundary`, и `true`. Не паникует при любом `max`, включая `0`.
- **FR-9:** `lib.rs` объявляет `pub mod profile; pub mod text;`. Рабочий код без `unwrap/expect/panic!`/индексации, способной паниковать; `#![forbid(unsafe_code)]` сохраняется; `just lint` и `cargo test --workspace` зелёные.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход — `&str` (валидный UTF-8 гарантирован типом); выход — срезы того же буфера или `Cow`. Ввода-вывода, сети, глобального состояния нет.
- Прав и ролей нет: библиотечный API.
- Единственная ошибка — `UnknownDialect` из `FromStr`. Остальные функции тотальны: на любой строке возвращают значение без паники (недоверенный файл в Quick Look не должен ронять превью).
- Сложность линейная по длине входа; `frontmatter_dialect` и `split_frontmatter` не копируют текст.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/profile.rs` (новый, FR-1, FR-2), `crates/nimmd-core/src/text.rs` (новый, FR-3–FR-8), `crates/nimmd-core/src/lib.rs` (FR-9), `crates/nimmd-core/Cargo.toml` (`serde = { workspace = true }`), `Cargo.lock`.
- Frontend: нет.
- Новые тесты: `crates/nimmd-core/tests/text.rs` — десять тестов из плана Task 1 Step 1 плюс случаи AC-1–AC-6 (serde/`FromStr`, `\r` одиночный, закрывающий `---` в конце без `\n`, `---` с пробелом, `max = 0`).

## Критерии приёмки [@for: all]
### AC-1: Типы профиля
- **Дано:** `Profile::default()`, каждый вариант `Dialect`.
- **Действие:** сериализовать в JSON и обратно; разобрать `"GFM"`, `"MultiMarkdown"`, `"kramdown"`, `""`, `" gfm"` через `FromStr`.
- **Ожидание:** дефолт — `CommonMark` и девять `false`; JSON-имена `"commonmark"`, `"gfm"`, `"multimarkdown"`, `"obsidian"`, обратное чтение даёт исходное значение; первые два разбора `Ok`, остальные `Err(UnknownDialect)`.

### AC-2: BOM и стиль переносов
- **Дано:** `"\u{FEFF}# x"`, `"# x"`, `"a\r\nb"`, `"abc"`, `"a\r\nb\r\nc\nd"`, `"a\r\nb\n"`, `"a\rb\rc\n"`.
- **Действие:** `strip_bom`, `detect_line_ending`.
- **Ожидание:** `(true, "# x")`, `(false, "# x")`; `CrLf`, `Lf`, `CrLf`, `Lf` (равенство), `Lf` (одиночный `\r` не считается).

### AC-3: Нормализация переносов
- **Дано:** `"a\nb"`, `"a\r\nb\rc"`, `"\r\r\n"`.
- **Действие:** `normalize_newlines`.
- **Ожидание:** первая — `Cow::Borrowed` с тем же текстом; вторая — `"a\nb\nc"`; третья — `"\n\n"`.

### AC-4: Отделение frontmatter
- **Дано:** `"---\ntitle: x\n---\n# H\n"`, `"---\n---\n"`, `"---\na: 1\n---"`, `"---\ntitle: x\n# H\n"`, `"text\n---\na\n---\n"`, `"+++\na=1\n+++\nbody"`, `"--- \na\n---\n"`, `"---\na\n--- \nb\n"`, `""`.
- **Действие:** `split_frontmatter`.
- **Ожидание:** `(Some("---\ntitle: x\n---\n"), "# H\n")`; `(Some("---\n---\n"), "")`; `(Some("---\na: 1\n---"), "")`; остальные шесть — `(None, src)`.

### AC-5: Переопределение диалекта
- **Дано:** frontmatter `"---\nmarkdown:  GFM\n---\n"`, `"---\nmarkdown : obsidian \n---\n"`, `"---\ntitle: markdown: gfm\n---\n"`, `"---\n  markdown: gfm\n---\n"`, `"---\nmarkdown: kramdown\n---\n"`, `"---\nMarkdown: gfm\n---\n"`.
- **Действие:** `frontmatter_dialect`.
- **Ожидание:** `Some(Gfm)`, `Some(Obsidian)`, остальные четыре — `None`.

### AC-6: Обрезка по границе символа
- **Дано:** `"aé"` (`é` — 2 байта), `MAX_BYTES`.
- **Действие:** `truncate_at_char_boundary` с `max` = 2, 10, 0, 3.
- **Ожидание:** `("a", true)`, `("aé", false)`, `("", true)`, `("aé", false)`; `MAX_BYTES == 10_485_760`.

## Влияние на руководство [@for: docs]
- не требуется: внутренний API ядра без пользовательского интерфейса; документация крейта — #46.
