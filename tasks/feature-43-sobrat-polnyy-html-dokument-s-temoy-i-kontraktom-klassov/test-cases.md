# Тестовые сценарии: Собрать полный HTML-документ с темой и контрактом классов

## Метаданные
- Issue: #43
- Стек: `cargo test -p nimmd-core --test document` (`crates/nimmd-core/tests/document.rs`), снапшоты `insta`. Ролей и изоляции данных нет (библиотека) — сценарии доступа к объектам неприменимы.

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
| AC-8 | SC-008, SC-009 |
| AC-9 | SC-010 |
| AC-10 | SC-011 |

## Сценарии
### SC-001: Пустой и пробельный вход
- **Покрывает:** AC-1
- **Уровень:** integration
- **Предусловия:** `RenderOptions::default()`.
- **Шаги:** `render("")`, `render("  \n\n ")`.
- **Ожидание:** оба: `html` начинается с `<!doctype html>`, содержит `<main class="nimmd"></main>`, `title == None`, `!truncated`.
- **Результат выполнения:** pass

### SC-002: BOM, CRLF, frontmatter, заголовок
- **Покрывает:** AC-2
- **Уровень:** integration
- **Предусловия:** `default()`.
- **Шаги:** `render("\u{FEFF}---\r\ntitle: X\r\n---\r\n# Заголовок\r\n")`.
- **Ожидание:** `frontmatter == Some("---\ntitle: X\n---\n")`, `had_bom`, `line_ending == CrLf`; `body` содержит `<pre class="frontmatter">` с `title` и `class="` внутри; `title == Some("Заголовок")`; `html` содержит `<title>Заголовок</title>`.
- **Результат выполнения:** pass

### SC-003: Тема
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** вход `"x"`.
- **Шаги:** `render` с `Theme::Dark`, `Theme::Light`, `default()`.
- **Ожидание:** `data-theme="dark"` / `"light"` / `"auto"`; в каждом `html.matches("<style").count() == 1`, `<style>` содержит `prefers-color-scheme: dark`.
- **Результат выполнения:** pass

### SC-004: `body` — подстрока `html`, не зависит от профиля
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** вход `"[[Note]] и $a+b$\n\n- [x] done\n"`; профили `Profile::default()` и Obsidian + `wikilinks`, `math`.
- **Шаги:** `render` с каждым `profile`.
- **Ожидание:** `html.contains(&body)`, `body.starts_with("<main class=\"nimmd\">")`; `body` равны; `Rendered.profile` — переданный.
- **Результат выполнения:** pass

### SC-005: Профиль равен `detect` на фикстурах
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** фикстуры `tests/fixtures/**/*.md`.
- **Шаги:** `render(src, default)` и `detect(normalize_newlines(strip_bom(src).1))`.
- **Ожидание:** профили равны на каждой.
- **Результат выполнения:** pass

### SC-006: Удалённые картинки
- **Покрывает:** AC-6
- **Уровень:** integration
- **Предусловия:** документ из восьми элементов `Дано` AC-6.
- **Шаги:** `render` с `remote_images` = false и = true.
- **Ожидание:** false — `<img` ровно один (`local.png`), `class="remote-image"` × 6 с адресами a…d, f, g, `<a href="https://x"`; true — `<img` × 7, `remote-image` нет.
- **Результат выполнения:** pass

### SC-007: Режимы формул
- **Покрывает:** AC-7
- **Уровень:** integration
- **Предусловия:** `F` = `"$x^2$\n\n$$\\frac{a}{b}$$\n"`; опасные формулы `Дано` AC-7; `D` = `$`+`{`×10 000+`x`+`}`×10 000+`$`, `N` — то же с ×65 (> `MAX_NESTING`).
- **Шаги:** `render` каждого входа с `Katex`, `MathML`, `Source`.
- **Ожидание:** `F`: `Katex` — `data-math-style`; `MathML` — `<math`, `insta::assert_snapshot!(body)`; `Source` — `<code>x^2</code>` и `<pre><code>`. Опасные, все режимы: за `<` в `body` — буква или `/`, теги — `main p span code pre` и MathML (без `script`, `img`, `b`); `&` — только сущности; `Katex`/`Source` — `a&lt;b`, `a&amp;b`. `D`, `N` в `MathML` — без паники, `body` как у `Source`.
- **Результат выполнения:** pass

### SC-008: Ресурсы по профилю — `None` и `Link`
- **Покрывает:** AC-8
- **Уровень:** integration
- **Предусловия:** входы `$$x$$`, `plain`, ` ```mermaid\nA-->B\n``` `.
- **Шаги:** `render` с `AssetMode::None`, `Link { base_url: "assets" }`.
- **Ожидание:** `None` — нет `<script`/`<link`; `Link`: `assets/katex.min.js` только у формулы, `assets/mermaid.min.js` только у mermaid; инлайн-скрипт формулы — `trust: false`, mermaid — `securityLevel: 'strict'`, оба в `DOMContentLoaded`.
- **Результат выполнения:** pass

### SC-009: Экранирование ресурсов хоста
- **Покрывает:** AC-8
- **Уровень:** integration
- **Предусловия:** вход `$$x$$`.
- **Шаги:** `render` с `Inline { katex_js: "a</script>b", .. }` и `Link { base_url: "a\"><x" }`.
- **Ожидание:** `Inline` — `html` содержит `a<\/script>b`, нет `a</script>b`; `Link` — нет `a"><x`, кавычка как `&quot;`.
- **Результат выполнения:** pass

### SC-010: Баннеры
- **Покрывает:** AC-9
- **Уровень:** integration
- **Предусловия:** текст 2 МБ (`"a\n".repeat(..)`), `max_bytes = 1 048 576`.
- **Шаги:** `render`; `render("x", not_utf8: true)`.
- **Ожидание:** `truncated`, `body` кончается на `<div class="banner truncated">…1 МБ…</div></main>`; второй — `body` с `<main class="nimmd"><div class="banner not-utf8">`.
- **Результат выполнения:** pass

### SC-011: Контракт классов
- **Покрывает:** AC-10
- **Уровень:** integration
- **Предусловия:** эталон `Дано` AC-10; `include_str!` `packages/theme/nimmd.css`, `CONTRACT.md`.
- **Шаги:** `render`; классовые селекторы `.имя` из CSS; два списка `CONTRACT.md`.
- **Ожидание:** каждый класс общего списка есть в `html` в `class`; каждый классовый селектор `nimmd.css` — в списках `CONTRACT.md`.
- **Результат выполнения:** pass
