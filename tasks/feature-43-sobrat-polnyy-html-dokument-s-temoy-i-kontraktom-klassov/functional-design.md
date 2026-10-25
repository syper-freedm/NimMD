# Функциональный дизайн: Собрать полный HTML-документ с темой и контрактом классов

## Метаданные [@for: all]
- Issue: #43
- Size: M
- Владелец: Системный аналитик

## Цель [@for: all]
- Хост (Quick Look, `nimmd --view`, редактор) получает от ядра одним вызовом `render` самодостаточный HTML-документ с темой light/dark или только фрагмент `<main>`, и документ выглядит одинаково везде (спек 5.1, 5.3, 5.4 «Контракт классов»).

## Границы [@for: all]
- Входит: `render`, `RenderOptions`, `Rendered`, `Theme`, `MathMode`, `AssetMode`; CSS темы и подсветки в `packages/theme/`, `packages/theme/CONTRACT.md`; плейсхолдер `http(s)`-картинок; frontmatter блоком YAML; баннеры «файл обрезан» и «не UTF-8»; подключение KaTeX и mermaid.js по профилю.
- Не входит: `FileKind` и файлы данных (#44), `highlight -> Vec<Span>` для редактора, FFI и хосты, проверка путей локальных картинок (`nimmd-assets`), CSP хостов, бенчмарк (Task 7), пример `examples/render.rs`.
- Размер: в GitHub задача помечена `L`, по объёму это один модуль ядра + CSS без миграций и новых слоёв — классифицирована `M` без декомпозиции.

## Целевое поведение [@for: all]
- Хост передаёт исходный текст файла (UTF-8, возможно с BOM и `\r\n`) и опции; ядро возвращает `Rendered`: полный документ `html`, фрагмент `body`, профиль, frontmatter, стиль переносов, признак BOM, признак обрезки, заголовок.
- Профиль меняет только подключаемые скрипты и поле `profile`; `<main>` от профиля не зависит (спек 6).
- Удалённые картинки не загружаются: вместо них — плейсхолдер с адресом. Ничего из документа не исполняется как скрипт.

## Требования [@for: all]
- **FR-1:** Публичные типы в `lib.rs` (реализация — `crates/nimmd-core/src/document.rs`): `enum Theme { Auto, Light, Dark }`, `enum MathMode { Katex, MathML, Source }` (serde, lowercase); `enum AssetMode { None, Link { base_url: String }, Inline { katex_css: String, katex_js: String, mermaid_js: String } }`; `struct RenderOptions { theme, assets, math, remote_images: bool, not_utf8: bool, max_bytes: usize, profile: Option<Profile> }` с `Default` = `Auto`, `None`, `Katex`, `false`, `false`, `text::MAX_BYTES`, `None`; `struct Rendered { html, body, profile, frontmatter: Option<String>, line_ending: LineEnding, had_bom: bool, truncated: bool, title: Option<String> }` (`Serialize`; `LineEnding` получает `Serialize`). `pub fn render(src: &str, opts: &RenderOptions) -> Rendered` — тотальна, без `std::fs/net/process`.
- **FR-2:** Конвейер: `strip_bom` → `detect_line_ending` → `normalize_newlines` → `truncate_at_char_boundary(max_bytes)` → `split_frontmatter` → разбор тела comrak **один раз** (те же опции и `limit_nesting`, что у `render_fragment`): одно дерево даёт и HTML фрагмента, и флаги `math`/`mermaid` профиля. Профиль — `opts.profile`, иначе диалект и текстовые признаки `detect` + флаги из этого дерева (результат равен `detect` по тому же тексту). `render_fragment` и `detect` сохраняют поведение.
- **FR-3:** Документ: `<!doctype html><html data-theme="auto|light|dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">` + `<title>` (если есть заголовок) + один `<style>` темы с `nimmd.css` и `highlight.css` (единственный, кроме `<style>` KaTeX при `Inline`) (`include_str!` из `packages/theme/`) + ресурсы FR-8 + `</head><body>` + `body` + `</body></html>`. `Rendered.body` — `<main class="nimmd">…</main>`, байт-в-байт та же подстрока, что в `html`. Порядок внутри `<main>`: баннер «не UTF-8», frontmatter, фрагмент, баннер «обрезан». Пустой вход — `<main class="nimmd"></main>`.
- **FR-4:** `title` — текст первого заголовка уровня 1 (ATX `#` или setext `===`), склеенный из текстовых узлов и кода без разметки; нет — `None`. В `<title>` экранирован.
- **FR-5:** Frontmatter: `Rendered.frontmatter` — срез `split_frontmatter` (с разделителями, переносы `\n`). В `<main>` — `<pre class="frontmatter">` с YAML между разделителями, подсвеченным syntect CSS-классами (как блок кода `yaml`), текст экранирован.
- **FR-6:** При `remote_images = false` картинка (inline и reference-style), чей адрес (поле `url` узла после декодирования comrak) имеет схему `http` или `https` без учёта регистра при любом числе `/` после `:` (`https:x/y.png`, `HTTP:/x`) или начинается с `//`, выводится как `<span class="remote-image">` с адресом экранированным текстом; `<img>` для неё нет. Прочие картинки и все ссылки не меняются. При `true` — обычный `<img>`.
- **FR-7:** `MathMode::Katex` — `<span data-math-style>` comrak без изменений. `MathML` — TeX каждой формулы → MathML (`latex2mathml`, inline/display); ошибка преобразования, формула длиннее 10 000 байт или с вложенностью `{`/`\left` глубже `MAX_NESTING` — как `Source` (до вызова `latex2mathml`, чтобы его рекурсия не переполнила стек). `Source` — экранированный TeX: inline в `<code>`, display в `<pre><code>`. Ни в одном режиме текст формулы не попадает в HTML неэкранированным (в том числе `\text{<script>}`, `a<b`, `&`, `\mathrm{<img src=x onerror=y>}`).
- **FR-8:** Ресурсы `<head>` по профилю: при `features.math` и `math = Katex` — KaTeX (CSS + JS + инлайн-скрипт: для каждого `[data-math-style]` `katex.render(el.textContent, el, {displayMode, trust: false, throwOnError: false})`); при `features.mermaid` — mermaid.js + инлайн-скрипт (замена `layout: elk` на `dagre` в тексте `pre.mermaid`, `mermaid.initialize({startOnLoad: false, layout: 'dagre', securityLevel: 'strict', theme})` с темой по `data-theme`/`prefers-color-scheme`, `mermaid.run({querySelector: 'pre.mermaid'})`). Инлайн-скрипты обёрнуты в `DOMContentLoaded` и лежат файлами `packages/theme/katex-init.js` и `packages/theme/mermaid-init.js` (`include_str!`), чтобы их видели правила `.semgrep.yml`. `Link { base_url }` — `<link href="{base}/katex.min.css">`, `<script defer src="{base}/katex.min.js">`, `<script src="{base}/mermaid.min.js">`, `base_url` экранирован как значение атрибута. `Inline` — содержимое в `<style>`/`<script>`, последовательности `</` внутри заменяются на `<\/` (иначе `</script>` в бандле закрыл бы тег). `None` — ни `<script>`, ни `<link>`.
- **FR-9:** Баннеры: `truncated` → `<div class="banner truncated">Файл обрезан: показаны первые N МБ</div>` (N — `max_bytes / 1 048 576`, при `max_bytes` меньше 1 МБ — «первые N байт»); `not_utf8` → `<div class="banner not-utf8">Файл не в UTF-8: нераспознанные байты заменены знаком �</div>`.
- **FR-10:** `packages/theme/nimmd.css`: цвета — переменные на `:root`; тёмная палитра — в `@media (prefers-color-scheme: dark)` только для `[data-theme="auto"]` и принудительно для `[data-theme="dark"]`; палитра алертов GitHub для обоих режимов; системные шрифты; ширина текста ≤ 46rem по центру. `highlight.css` — цвета классов syntect для обоих режимов. `packages/theme/CONTRACT.md` — два списка (спек 5.4 В6): общие с редактором (`markdown-alert`, `markdown-alert-title`, `markdown-alert-note|tip|important|warning|caution`, `contains-task-list`, `task-list-item`, `task-list-item-checkbox`, `footnote-ref`, `footnotes`, `frontmatter`, `mermaid`) и только ядра (`footnote-backref`, `nimmd`, `nimmd-data`, `data`, `banner`, `truncated`, `not-utf8`, `remote-image`), имена сверены с выводом ядра. Классовые селекторы `nimmd.css` — только из `CONTRACT.md`.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход — недоверенный документ; опции (`base_url`, содержимое `Inline`) — от хоста, доверенные, но экранируются (FR-8). Ошибок нет: результат всегда `Rendered`.
- Безопасность: сохраняются `unsafe_ = false`, `escape = true`, `limit_nesting`; KaTeX `trust: false`, Mermaid `securityLevel: 'strict'` (правила `.semgrep.yml`); текст документа в инлайн-скрипты не попадает. Сеть ядро не трогает: удалённые картинки — плейсхолдер.
- Новая зависимость — `latex2mathml` из `[workspace.dependencies]` (MIT).

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/document.rs` (новый), `crates/nimmd-core/src/render.rs` (общий разбор FR-2, картинки FR-6, формулы FR-7, frontmatter FR-5), `crates/nimmd-core/src/detect.rs` (текстовые признаки отдельно от разбора), `crates/nimmd-core/src/text.rs` (`Serialize` у `LineEnding`), `crates/nimmd-core/src/lib.rs`, `crates/nimmd-core/Cargo.toml`, `Cargo.lock`, `packages/theme/nimmd.css`, `packages/theme/highlight.css`, `packages/theme/CONTRACT.md`, `packages/theme/katex-init.js`, `packages/theme/mermaid-init.js` (новые).
- Frontend: не затрагивается.
- Новые тесты: `crates/nimmd-core/tests/document.rs` (+ insta-снапшоты `math_mathml`, документ целиком), тест контракта: классы общего списка `CONTRACT.md` встречаются в выводе ядра на эталонном документе, классовые селекторы `nimmd.css` ⊆ `CONTRACT.md`.

## Критерии приёмки [@for: all]
### AC-1: Пустой и пробельный вход
- **Дано:** `""` и `"  \n\n "`.
- **Действие:** `render` с `RenderOptions::default()`.
- **Ожидание:** `html` начинается с `<!doctype html>`, содержит `<main class="nimmd"></main>`; `title` = `None`, `truncated` = false, без паники.

### AC-2: BOM, CRLF, frontmatter и заголовок
- **Дано:** `"\u{FEFF}---\r\ntitle: X\r\n---\r\n# Заголовок\r\n"`.
- **Действие:** `render`.
- **Ожидание:** `frontmatter` = `"---\ntitle: X\n---\n"`, `had_bom`, `line_ending` = `CrLf`, в `<main>` есть `<pre class="frontmatter">` с `title` и классами подсветки, `title` = `"Заголовок"`, `<title>Заголовок</title>`.

### AC-3: Тема
- **Дано:** `"x"`.
- **Действие:** `render` с `Theme::Dark`, `Light` и по умолчанию.
- **Ожидание:** `data-theme="dark"`, `"light"`, `"auto"`; ровно один `<style>`, в нём `prefers-color-scheme: dark`.

### AC-4: `body` — подстрока `html`, профиль не меняет `<main>`
- **Дано:** `"[[Note]] и $a+b$\n\n- [x] done\n"`.
- **Действие:** `render` с `profile` = `Profile::default()` и с Obsidian + `wikilinks`, `math`.
- **Ожидание:** в обоих `html` содержит `body`, `body` начинается с `<main class="nimmd">`; `body` двух вызовов равны; `Rendered.profile` = переданному.

### AC-5: Профиль из одного разбора равен `detect`
- **Дано:** фикстуры `crates/nimmd-core/tests/fixtures/**/*.md`.
- **Действие:** `render` без `profile`.
- **Ожидание:** `Rendered.profile` = `detect` по тому же тексту (после BOM и нормализации переносов) для каждой фикстуры.

### AC-6: Удалённые картинки
- **Дано:** `![a](https://x/y.png)`, `![b](HTTP://x/z.png)`, `![c](//x/w.png)`, `![d][r]\n\n[r]: https://x/r.png`, `![f](https:x/y.png)`, `![g](HTTP:/x/y.png)`, `![e](local.png)`, `[l](https://x)`.
- **Действие:** `render` с `remote_images` = false и = true.
- **Ожидание:** false — для a…d, f, g нет `<img`, есть `class="remote-image"` с адресом; `local.png` — `<img`; ссылка `<a href="https://x"`; true — все картинки `<img`.

### AC-7: Режимы формул
- **Дано:** `"$x^2$\n\n$$\\frac{a}{b}$$\n"`; опасные `$\\text{<script>}$`, `$a<b$`, `$a&b$`, `$\\mathrm{<img src=x onerror=y>}$`; `$` + `{` × 10 000 + `x` + `}` × 10 000 + `$`.
- **Действие:** `render` с `Katex`, `MathML`, `Source`.
- **Ожидание:** `Katex` — `data-math-style`; `MathML` — `<math`, снапшот `body`; `Source` — `<code>x^2</code>` и `<pre><code>`; во всех режимах в выводе опасных формул нет `<script`, `<img` и неэкранированных `<`/`&` из TeX; формула с вложенностью 10 000 в `MathML` — без паники, как `Source`.

### AC-8: Ресурсы по профилю
- **Дано:** `$$x$$`, `plain`, ` ```mermaid\nA-->B\n``` `.
- **Действие:** `render` с `AssetMode::None`, `Link { base_url: "assets" }`, `Inline` с `katex_js = "a</script>b"`.
- **Ожидание:** `None` — нет `<script`/`<link`; `Link` — `assets/katex.min.js` только при формуле, `assets/mermaid.min.js` только при mermaid, `plain` без них; инлайн-скрипты содержат `trust: false`, `securityLevel: 'strict'`, `DOMContentLoaded`; `Inline` — `a<\/script>b`; `Link { base_url: "a\"><x" }` — кавычка экранирована.

### AC-9: Баннеры
- **Дано:** текст 2 МБ и `max_bytes` = 1 МБ; `not_utf8` = true.
- **Действие:** `render`.
- **Ожидание:** `truncated` = true, `<div class="banner truncated">` с «1 МБ» после фрагмента; `<div class="banner not-utf8">` в начале `<main>`.

### AC-10: Контракт классов
- **Дано:** эталонный документ с алертами всех пяти видов, задачей, сноской, frontmatter и mermaid; `nimmd.css`, `CONTRACT.md`.
- **Действие:** `render`; разбор селекторов CSS.
- **Ожидание:** каждый класс общего списка `CONTRACT.md` есть в `html`; каждый классовый селектор `nimmd.css` есть в `CONTRACT.md`.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства в репозитории ещё нет; API ядра описывает doc-комментарий и `CONTRACT.md`.
