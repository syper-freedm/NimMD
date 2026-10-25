# Функциональный дизайн: Показывать файлы данных TOML, YAML, JSON, CSV и TXT

## Метаданные [@for: all]
- Issue: #44
- Size: M
- Владелец: Системный аналитик

## Цель [@for: all]
- Хост одним вызовом `render` с типом файла получает HTML файла данных той же палитрой, что блоки кода в Markdown (спек 5.7, план ядра Task 5).
- Редактор получает спаны `highlight` (К14).

## Границы [@for: all]
- Входит: FR-1…FR-11; `decode_lossy` — интерфейс плана Task 5 для FFI.
- Не входит: хосты и FFI; бенчмарк и порог 2 МБ (#45); `examples/render.rs`; редактирование данных.
- Размер: метка `L`, по объёму — модули одного крейта без новых слоёв: `M` без декомпозиции (как #43).

## Целевое поведение [@for: all]
- Файл JSON/TOML/YAML показывается подсвеченным текстом как есть, CSV — исходным текстом с цветом по колонкам, TXT — текстом с переносом строк; больше 2 МБ — голова подсвечена, хвост простым текстом, с баннером.
- ` ```toml ` в Markdown подсвечивается; прочие блоки кода и frontmatter выглядят как раньше.

## Требования [@for: all]
- **FR-1:** `pub enum FileKind { Markdown, Toml, Yaml, Json, Csv, Text }` (`Serialize`/`Deserialize`, lowercase, `Default` = `Markdown`, `Copy`, `Eq`) в `crates/nimmd-core/src/data.rs`, реэкспорт из `lib.rs`. `FileKind::from_extension(ext: &str) -> Option<FileKind>` — расширение без точки, без учёта регистра (ASCII): `md|markdown|mdown|mkd` → `Markdown`, `toml`, `yaml|yml`, `json`, `csv`, `txt`; иначе `None`.
- **FR-2:** `RenderOptions.kind: FileKind` (по умолчанию `Markdown`); `Rendered.highlight_limited: bool`. Для `Markdown` поведение `render` не меняется. Для остальных: `strip_bom` → `detect_line_ending` → `normalize_newlines` → `truncate_at_char_boundary(max_bytes)` → рендер по `kind`; `split_frontmatter` и `detect` не выполняются: `profile = Profile::default()`, `frontmatter = None`, `title = None`; KaTeX/Mermaid не подключаются при любом `assets`.
- **FR-3:** Документ данных — та же обвязка `<head>`, что у Markdown (тема, один `<style>`), в `<style>` дополнительно `packages/theme/data.css` (`include_str!`, только для не-Markdown). `body` = `<main class="nimmd nimmd-data" data-kind="toml|yaml|json|csv|text">` + баннер «не UTF-8» + `<pre class="data">…</pre>` + баннер лимита (FR-6) + баннер «обрезан» + `</main>`; баннеры — как у Markdown.
- **FR-4:** Свой набор синтаксисов в `crates/nimmd-core/src/highlight.rs`: `SyntaxSet` один раз (`OnceLock`) из `include_bytes!("../assets/syntaxes.packdump")` через `syntect::dumps::from_uncompressed_data` (дамп `dump_to_uncompressed_file`: ленивые синтаксисы уже сжаты); ошибка чтения — пустой `SyntaxSet::new()`, без паники. Дамп = стандартный набор syntect 5.3 (`load_defaults_newlines`) + `crates/nimmd-core/assets/syntaxes/TOML/TOML.sublime-syntax` из sublimehq/Packages коммита `72065294da5dfb1862cd43177df4a50e29349135`, рядом `LICENSE` sublimehq/Packages с источником и хэшем в первой строке. Дамп собирает `crates/nimmd-core/examples/build_syntaxes.rs` (рецепт `just syntaxes`), печатает `TOML: ok` или падает; дамп коммитится. Набор общий для адаптера comrak (`Highlighter` переносится из `render.rs`), frontmatter, файлов данных и `highlight`. Зависимость `syntect` (фичи в `[workspace.dependencies]` корневого `Cargo.toml`): обычная — `default-features = false`, `html`, `parsing`, `regex-fancy` (без `default-syntaxes`); dev — дополнительно `default-syntaxes`, `yaml-load`; версия 5.3, один экземпляр в дереве.
- **FR-5:** Выбор грамматики везде одинаков (как в #88): первое слово до `,` → `find_syntax_by_token`, иначе `find_syntax_by_first_line`, иначе `find_syntax_by_name("Plain Text")`; пустой язык — Plain Text. Паникующие `find_syntax_plain_text`/`from_binary` не вызываются: нет грамматики или ошибка syntect — экранированный текст без подсветки (FR-2 #88). Файл TOML/YAML/JSON подсвечивается грамматикой `toml`/`yaml`/`json` тем же `ClassedHTMLGenerator` + `ClassStyle::Spaced`, что блок кода. TXT — экранированный текст без подсветки; `data.css` даёт `[data-kind="text"] pre.data` перенос `white-space: pre-wrap`.
- **FR-6:** `pub const HIGHLIGHT_LIMIT: usize = 2 * 1024 * 1024`. Для TOML/YAML/JSON/CSV длиннее лимита подсвечивается голова — до последнего `\n` (включительно) в первых `HIGHLIGHT_LIMIT` байтах, без `\n` — до границы символа не дальше лимита; хвост дописывается экранированным текстом в тот же `<pre>`; `highlight_limited = true`, после `</pre>` — `<div class="banner">Подсветка отключена после 2 МБ</div>`. TXT и Markdown лимит не применяют (`highlight_limited = false`).
- **FR-7:** CSV — `crates/nimmd-core/src/csv.rs`, публичный модуль `nimmd_core::csv`, без crate `csv`. Токенизатор RFC 4180 (конечный автомат): поле, начинающееся с `"`, — в кавычках до `"`, за которой не следует `"` (`""` — экранированная кавычка); вне кавычек разделитель завершает поле, `\n` — запись; незакрытая кавычка тянет поле до конца текста. `pub fn detect_delimiter(src: &str) -> u8`: кандидаты `,` `;` `\t` `|`; для каждого — число полей в каждой из первых 50 записей; у кандидата берётся самое частое число полей (мода; при равной частоте — большее число полей) и сколько записей его имеют; побеждает кандидат с модой > 1 и наибольшим числом таких записей, при равенстве — порядок `,` `;` `\t` `|`; ни у кого мода не больше 1 — `,`.
- **FR-8:** Вывод CSV: поле → `<span class="c{N mod 8}">{экранированный исходник поля вместе с кавычками}</span>` (пустое поле — без `<span>`), разделитель → `<span class="d">{разделитель}</span>`, `\n` — как есть. `N` — номер поля в записи с 0, сбрасывается в каждой записи; лишние поля продолжают нумерацию. Видимый текст (без тегов, сущности раскрыты) равен входу после нормализации переносов.
- **FR-9:** `packages/theme/data.css`: `.c0`…`.c7` — 8 цветов для светлой и тёмной темы (переменные и тёмная палитра — как в `highlight.css`), контраст каждого цвета к фону темы ≥ 4.5:1, значения контраста — в комментарии; `.d` — приглушённый цвет (контраст ≥ 3:1). `c0`…`c7`, `d` в `CONTRACT.md` не вносятся (спек 5.4).
- **FR-10:** `pub fn highlight(lang: &str, code: &str) -> Vec<Span>` и `pub struct Span { pub start: u32, pub end: u32, pub class: String }` (`Serialize`) в `highlight.rs`, реэкспорт из `lib.rs`. Грамматика — FR-5. Span — максимальный непустой участок текста с неизменным стеком scope syntect; `class` — атомы всех scope стека снаружи внутрь через пробел, без повторов; участки с пустым стеком не выводятся. Смещения — в UTF-16 от начала `code`, спаны упорядочены и не пересекаются. Подсвечиваются первые `HIGHLIGHT_LIMIT` байт по правилу FR-6, дальше спанов нет. Ошибка syntect или нет грамматики — пустой `Vec`, без паники.
- **FR-11:** `pub fn text::decode_lossy(bytes: &[u8]) -> (Cow<'_, str>, bool)` — `String::from_utf8_lossy`; `true`, если были невалидные байты.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход недоверенный: текст попадает в HTML только экранированным, атрибуты — константы ядра. Функции тотальны, `unwrap`/`expect` в рабочем коде нет.
- `std::fs` — только в `examples/build_syntaxes.rs` (вне `src/`); ядро читает только встроенный дамп (`bincode`, исключение RUSTSEC-2025-0141 в `deny.toml`).
- Лицензия TOML-синтаксиса разрешительная (аудит лицензий, стр. 106–107).

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/highlight.rs`, `crates/nimmd-core/src/data.rs`, `crates/nimmd-core/src/csv.rs` (новые); `crates/nimmd-core/src/render.rs` (адаптер уходит в `highlight.rs`), `crates/nimmd-core/src/document.rs` (ветка по `kind`, `highlight_limited`), `crates/nimmd-core/src/text.rs` (`decode_lossy`), `crates/nimmd-core/src/lib.rs`, `crates/nimmd-core/Cargo.toml`, `Cargo.toml`, `Cargo.lock`; `crates/nimmd-core/assets/syntaxes/TOML/TOML.sublime-syntax`, `crates/nimmd-core/assets/syntaxes/LICENSE`, `crates/nimmd-core/assets/syntaxes.packdump`, `crates/nimmd-core/examples/build_syntaxes.rs` (новые; файлы из sublimehq/Packages скачиваются `gh api repos/sublimehq/Packages/contents/<путь>?ref=<коммит>`); `packages/theme/data.css` (новый); `justfile` (рецепт `syntaxes`).
- Frontend: не затрагивается.
- Новые тесты: `crates/nimmd-core/tests/data.rs`, `crates/nimmd-core/tests/csv.rs`, `crates/nimmd-core/tests/highlight.rs`, фикстуры `crates/nimmd-core/tests/fixtures/data/` (`config.toml`, `compose.yaml`, `package.json`, `notes.txt`, `comma.csv`, `semicolon.csv`, `tab.csv`, `quoted.csv`, `ragged.csv`, `single.csv`), insta-снапшоты документов фикстур TOML/YAML/JSON/TXT/`quoted.csv`. Существующие снапшоты Markdown не должны меняться.

## Критерии приёмки [@for: all]
### AC-1: Тип по расширению
- **Дано:** `"YML"`, `"markdown"`, `"Json"`, `"txt"`, `"exe"`, `""`, `".md"`.
- **Действие:** `FileKind::from_extension`.
- **Ожидание:** `Yaml`, `Markdown`, `Json`, `Text`, `None`, `None`, `None`; `FileKind::default()` = `Markdown`; serde: `FileKind::Json` ↔ `"json"`.

### AC-2: TOML, YAML и JSON подсвечены, обвязка данных
- **Дано:** `[server]\nport = 8080\n` (TOML), `a: &x 1\nb: *x\n` (YAML), `{"a": [1, true, null]}\n` (JSON).
- **Действие:** `render` с соответствующим `kind`.
- **Ожидание:** `body` начинается с `<main class="nimmd nimmd-data" data-kind="toml">` (`yaml`, `json`), внутри `<pre class="data">` есть `<span class="`; у TOML есть класс `source toml`; видимый текст `<pre>` равен входу; ровно один `<style>`, в нём правила `data.css` (`.c0`).

### AC-3: Классы файла и блока кода совпадают; TOML в Markdown
- **Дано:** JSON `{"a": [1, true, null]}\n`; Markdown с тем же текстом в ` ```json ` и ` ```toml\na = 1\n``` `.
- **Действие:** `render` с `Json` и с `Markdown`.
- **Ожидание:** множества значений `class="…"` внутри `<pre>` у файла JSON и блока ` ```json ` равны; блок ` ```toml ` содержит `<span class="source toml`; снапшоты Markdown из `crates/nimmd-core/tests/snapshots/` не меняются.

### AC-4: TXT — экранированный текст без подсветки
- **Дано:** `<b>не тег</b>\tтаб\n` + строка 250 символов.
- **Действие:** `render` с `Text`.
- **Ожидание:** `data-kind="text"`, есть `&lt;b&gt;не тег&lt;/b&gt;`, нет `<b>` и `<span`; `data.css` содержит `white-space: pre-wrap` для `[data-kind="text"]`.

### AC-5: Данные без профиля, frontmatter и скриптов
- **Дано:** YAML `---\nmarkdown: obsidian\nmath: $$x$$\n`, `assets = Link { base_url: "assets" }`.
- **Действие:** `render` с `Yaml`.
- **Ожидание:** `profile == Profile::default()`, `frontmatter == None`, `title == None`, в `html` нет `<script` и `<link`.

### AC-6: Лимит подсветки
- **Дано:** JSON `{"k": 1}\n` × (`HIGHLIGHT_LIMIT / 9 + 1000`); CSV `a,b\n` × (`HIGHLIGHT_LIMIT / 4 + 1000`); JSON `{}\n`; TXT того же размера, что первый; одна строка JSON длиннее 2 МБ без `\n`.
- **Действие:** `render` с `Json` / `Text`.
- **Ожидание:** первый — `highlight_limited`, есть «Подсветка отключена после 2 МБ», видимый текст `<pre>` равен входу, хвост выведен без подсветки — `body` содержит `\n{&quot;k&quot;: 1}\n</pre>`; CSV — `highlight_limited`, баннер, `body` содержит `\na,b\n</pre>`; `{}` и TXT — `highlight_limited == false`, баннера нет; длинная строка — без паники, видимый текст равен входу.

### AC-7: Кодировка, BOM, CRLF и обрезка для данных
- **Дано:** `text::decode_lossy(b"ok \xff\n")` и `decode_lossy(b"ok\n")`; `"\u{FEFF}a = 1\r\n"` (TOML); TOML 3 МБ с `max_bytes = 1 МБ`.
- **Действие:** `decode_lossy`; `render` с `not_utf8 = true` / с BOM / с `max_bytes`.
- **Ожидание:** `(…\u{FFFD}…, true)` и `("ok\n", false)`; баннер `not-utf8` в начале `<main>` данных; `had_bom`, `line_ending == CrLf`; `truncated`, баннер `truncated`.

### AC-8: Разделитель CSV
- **Дано:** `a,b,c\n1,2,3\n`; `имя;цена\nчай;1,5\nкофе;2,5\n`; `a\tb\n1\t2\n`; `a|b\n1|2\n`; `"a;b",c\n"1;2",3\n`; `один\nдва\n`; `""`.
- **Действие:** `csv::detect_delimiter`.
- **Ожидание:** `b','`, `b';'`, `b'\t'`, `b'|'`, `b','`, `b','`, `b','`.

### AC-9: Раскраска CSV и сохранение текста
- **Дано:** `a,b,c,d,e,f,g,h,i\n`; `a,"x\ny",c\n`; `a,"q""q",b\n`; `a,b\n1,2,3,4,5\nx\n`; `a,"b\nc,d\n`; `a,,b\n`; `<x>,&\n`; фикстуры `crates/nimmd-core/tests/fixtures/data/*.csv`.
- **Действие:** `render` с `Csv`.
- **Ожидание:** `<span class="c0">a</span>`, `<span class="c7">h</span>`, `<span class="c0">i</span>`; `<span class="c1">&quot;x\ny&quot;</span>` и `<span class="c2">c</span>`; `<span class="c1">&quot;q&quot;&quot;q&quot;</span>`; разделитель `<span class="d">,</span>`; пустое поле без `<span class="c1">`; `&lt;x&gt;`; без паники; видимый текст равен входу для каждого случая и каждой фикстуры; `semicolon.csv` раскрашен по `;`.

### AC-10: Дамп синтаксисов
- **Дано:** дамп и `LICENSE` из FR-4.
- **Действие:** `just syntaxes`, `git status --short crates/nimmd-core/assets`; `highlight("toml", "a = 1")`, `highlight("yaml", "a: 1")`, `highlight("json", "{}")`; `cargo tree -p nimmd-core -d -e normal`; `git grep -n "find_syntax_plain_text\|from_binary\|SyntectAdapter" crates/nimmd-core/src`; `just lint`, `cargo test --workspace`.
- **Ожидание:** `TOML: ok`, дамп в git не меняется; первая строка `LICENSE` содержит хэш коммита FR-4; `Vec` непустые; дублей `syntect` нет; `git grep` пуст; проверки зелёные.

### AC-11: Спаны редактора
- **Дано:** `highlight("rust", "let x = 1;")`, `highlight("rust", "let ё = \"ж\";")`, `highlight("nosuchlang", "x")`, `highlight("", "")`, код 3 МБ.
- **Действие:** `highlight`.
- **Ожидание:** непустые `Vec`; у всех спанов `start < end`, спаны упорядочены и не пересекаются, `end ≤` длины кода в UTF-16; во втором случае `end` последнего спана ≤ 12 (длина в UTF-16), а не в байтах (14); для `let` класс содержит `keyword` или `storage`; `nosuchlang` — спаны с `text plain`; пустой код — пустой `Vec`; 3 МБ — без паники, нет спана с `end` за пределами первых 2 МБ в UTF-16.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства нет; API описывают doc-комментарии (документация крейта — #46).
