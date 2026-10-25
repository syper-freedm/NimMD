# Функциональный дизайн: Реализовать рендер Markdown через comrak

## Метаданные [@for: all]
- Issue: #41
- Size: M
- Владелец: Системный аналитик

## Цель [@for: all]
- Ядро превращает тело Markdown в HTML-фрагмент со всеми расширениями v1 (спек 3 «Поддерживаемая разметка», 5.1): хосты (#43 — полный документ, позже — живое превью редактора) показывают документ отформатированным, а не разметкой.

## Границы [@for: all]
- Входит: `render_fragment`, опции comrak режима надмножества, подсветка кода syntect CSS-классами, mermaid-блок с классом, ограничение глубины вложенности (спек 5.2) до разбора, insta-снапшоты.
- Не входит: полный HTML-документ, `Rendered`, `RenderOptions`, тема и CSS, плейсхолдер http(s)-картинок, MathML, frontmatter-блок (#43); семантика Obsidian сверх отрисовки (переход по `[[ссылке]]`); списки определений и `{#id}` заголовков (вынесено за MVP, спек 3); бенчмарк 1 МБ (Task 7 плана).
- Решения, оставленные планом исполнителю (mermaid через `codefence_renderers` или пост-обработку), остаются за ним: ФД фиксирует только выход.

## Целевое поведение [@for: all]
- Вызывающий отделяет frontmatter (`split_frontmatter`) и передаёт тело без BOM, с переносами `\n`. `render_fragment` возвращает HTML-фрагмент без `<html>`/`<head>`/`<body>`, одинаковый для любого диалекта (от `Profile` не зависит, спек 6).
- Сырой HTML документа показывается как текст, опасные адреса ссылок не попадают в `href`/`src`: документ недоверенный.
- Функция не паникует ни на каком входе и укладывается в линейное время по длине входа, включая злонамеренную вложенность.

## Требования [@for: all]
- **FR-1:** В `nimmd-core` есть `pub fn render_fragment(body: &str) -> String` (`crates/nimmd-core/src/render.rs`, реэкспорт из `lib.rs`). Тотальна: без `unwrap`/`expect`/`panic!` в рабочем коде, без `std::fs/net/process`.
- **FR-2:** Опции разбора — общая функция `parse::options()` (её же использует `detect`): `autolink`, `table`, `tasklist`, `strikethrough`, `footnotes`, `alerts`, `multiline_block_quotes`, `math_dollars`, `wikilinks_title_after_pipe` включены; `description_lists` и `header_ids`/атрибуты заголовков выключены; `front_matter_delimiter` **не** задан — иначе блок `---…---` в начале тела пропал бы из вывода. `detect` разбирает comrak тело после `split_frontmatter` (а не весь текст), чтобы результат не изменился. Опции рендера: `render.unsafe_ = false`, `render.escape = true` (без неё comrak заменяет сырой HTML на `<!-- raw HTML omitted -->`, а не показывает его текстом), `render.github_pre_lang = true`, `render.tasklist_classes = true`. Опции и адаптер подсветки создаются один раз (`std::sync::OnceLock`).
- **FR-3:** Сырой HTML (блочный и инлайновый) показывается экранированным текстом (`render.escape = true`); в `href` ссылок, автоссылок и wikilinks и в `src` картинок не попадают адреса `javascript:`, `vbscript:`, `file:` и `data:` кроме `data:image/png|gif|jpeg|webp` — проверка `dangerous_url` comrak при `unsafe_ = false`, зафиксированная тестами.
- **FR-4:** Блоки кода с языком подсвечиваются syntect через адаптер comrak в режиме CSS-классов: в выводе блока есть `class="…"` у токенов и нет атрибута `style=`. Блок без языка или с неизвестным языком — `<pre><code>` с экранированным текстом без ошибки.
- **FR-5:** Fenced-блок, первое слово info-строки которого — `mermaid` (без учёта регистра), выводится как `<pre class="mermaid">` + экранированный текст диаграммы, без подсветки; незакрытый блок до конца текста — так же.
- **FR-6:** Алерты пяти типов GitHub (`NOTE`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`, без учёта регистра) — классы comrak `markdown-alert markdown-alert-<type>` и `markdown-alert-title`; прочие callouts Obsidian — обычная цитата с текстом (спек 5.1, К16). Задачи — `<input type="checkbox" … disabled>` с классами `task-list-item`; сноски — ссылка и раздел сносок comrak; формулы — `data-math-style="inline|display"` без изменений; wikilinks `[[Цель|текст]]` — `<a href="Цель"…>текст</a>`; таблицы — с `align` выравнивания.
- **FR-7:** Перед разбором тело проходит `text::limit_nesting` (#40). Функция дополняется двумя правилами: (а) для маркеров `N.`/`N)` сверх `MAX_NESTING` `\` вставляется перед `.`/`)`, а не перед цифрой — `\` не виден в рендере (CommonMark-экранирование пунктуации); (б) ведущие пробелы и табуляции строки шириной больше `4 * MAX_NESTING` колонок (табуляция — до следующей кратной 4 колонки) сокращаются до этой ширины — так глубину не наращивает и «лестница» вложенности через отступ продолжения (ARCH-005 из #40: 424 КБ — 4,5 с). Инвариант `limit_nesting` меняется: без потерь, кроме ведущих пробелов и табуляций сверх `4 * MAX_NESTING` колонок; удаление вставленных `\` и восстановление этих пробелов даёт исходник — doc-комментарий и тест `limit_nesting_is_lossless_and_idempotent` обновляются под него. `limit_nesting` остаётся `Cow::Borrowed` для текста без таких строк.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход — недоверенная строка до `MAX_BYTES`. Ошибок нет: результат всегда строка HTML.
- Сложность линейная: `limit_nesting`, разбор comrak с ограниченной глубиной, рендер с подсветкой. Порог 1 МБ ≤ 100 мс проверит бенчмарк Task 7 плана, здесь не меряется.
- Новые зависимости: `insta` (dev, из workspace; Apache-2.0 — в белом списке). Новых прав, сетевого и файлового доступа нет.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/render.rs` (новый: `render_fragment`, адаптер syntect, mermaid), `crates/nimmd-core/src/parse.rs` (опции FR-2), `crates/nimmd-core/src/detect.rs` (разбор тела вместо всего текста), `crates/nimmd-core/src/text.rs` (`limit_nesting` FR-7), `crates/nimmd-core/src/lib.rs`, `crates/nimmd-core/Cargo.toml` (`insta` в dev-dependencies), `Cargo.lock`.
- Frontend: не затрагивается.
- Новые тесты: `crates/nimmd-core/tests/render_fragment.rs` + insta-снапшоты в `crates/nimmd-core/tests/snapshots/` (таблица и задачи, сноски, алерт, подсветка); дополнения `crates/nimmd-core/tests/text.rs` для FR-7; существующие тесты `detect` должны остаться зелёными.

## Критерии приёмки [@for: all]
### AC-1: Таблица с выравниванием и задачи
- **Дано:** `| a | b |\n|---|:-:|\n| 1 | 2 |\n\n- [x] done\n- [ ] todo\n`.
- **Действие:** `render_fragment`.
- **Ожидание:** снапшот; в нём `<th align="center">`, два `<input type="checkbox"` (один `checked`), оба `disabled`, класс `task-list-item`.

### AC-2: Подсветка классами
- **Дано:** ` ```rust\nfn main() {}\n``` `.
- **Действие:** `render_fragment`.
- **Ожидание:** есть `<pre`, `class="` у токенов, нет `style=`; снапшот.

### AC-3: Блок без языка и с неизвестным языком
- **Дано:** (а) ` ```\n<b>x</b>\n``` `; (б) ` ```nosuchlang\nx\n``` `.
- **Действие:** `render_fragment`.
- **Ожидание:** оба — `<pre` + `<code`, в (а) `&lt;b&gt;`, без паники.

### AC-4: Алерты пяти типов и чужой callout
- **Дано:** по алерту `> [!NOTE]`, `> [!tip]`, `> [!IMPORTANT]`, `> [!warning]`, `> [!CAUTION]` с текстом; отдельно `> [!abstract] заголовок`.
- **Действие:** `render_fragment` для каждого.
- **Ожидание:** классы `markdown-alert-note|tip|important|warning|caution` и `markdown-alert-title`; `[!abstract]` — `<blockquote>` с текстом `[!abstract] заголовок`, без класса `markdown-alert`.

### AC-5: Формулы и цены
- **Дано:** (а) `$$E=mc^2$$`; (б) `$a+b$`; (в) `цена $5 и $10`.
- **Действие:** `render_fragment`.
- **Ожидание:** (а) `data-math-style="display"`; (б) `data-math-style="inline"`; (в) нет `data-math-style`.

### AC-6: Mermaid
- **Дано:** (а) ` ```mermaid\ngraph TD; A-->B\n``` `; (б) то же без закрывающего ограждения; (в) ` ```Mermaid ` с закрытием.
- **Действие:** `render_fragment`.
- **Ожидание:** все — `<pre class="mermaid">`, текст `graph TD; A--&gt;B`, без `class=` токенов подсветки внутри; (б) без паники.

### AC-7: Сырой HTML и опасные адреса
- **Дано:** (а) `<script>alert(1)</script>`; (б) `<img src=x onerror=alert(1)>` в абзаце; (в) `[a](javascript:alert(1))`, `[[javascript:alert(1)]]`, `![i](data:text/html,x)`, `[v](vbscript:x)`, `[f](file:///etc/passwd)`, `<javascript:alert(1)>`; (г) `![p](data:image/png;base64,AAAA)`.
- **Действие:** `render_fragment`.
- **Ожидание:** (а), (б) — нет `<script` и `<img src=x`, есть `&lt;`; (в) — ни в одном `href`/`src` нет `javascript:`, `vbscript:`, `file:` и `data:text/html`; (г) — `src="data:image/png;base64,AAAA"`.

### AC-8: Wikilink и сноски
- **Дано:** (а) `[[Note|текст]]` без других признаков Obsidian; (б) `текст[^1]\n\n[^1]: сноска\n`.
- **Действие:** `render_fragment`.
- **Ожидание:** (а) `<a href="Note"` и текст `текст`; (б) снапшот со ссылкой на сноску и разделом сносок.

### AC-9: Блок `---` в начале тела не теряется
- **Дано:** тело `---\nтекст\n---\n` (после `split_frontmatter` настоящего frontmatter).
- **Действие:** `render_fragment`; и `detect` для `---\nmarkdown: gfm\n---\n---\nтекст\n---\n`.
- **Ожидание:** в HTML есть `текст` и `<hr`; `detect` — `Gfm`, `frontmatter` = true; все тесты `detect` из #40 зелёные.

### AC-10: Глубокая вложенность не роняет и не тормозит рендер
- **Дано:** (а) `>` × 60 000 + `x`; (б) `1. ` × 100 + `x`; (в) «лестница» из 800 строк: строка k — k табуляций + `- ` × 64 + `x`; (г) строка из 300 пробелов + `- x`.
- **Действие:** `render_fragment` и `limit_nesting`.
- **Ожидание:** (а)–(в) — `render_fragment` без паники быстрее 1 с в debug-сборке, `x` есть в выводе; (б) — после `limit_nesting` строка содержит `\.` после 64-го маркера, в HTML нет `\`; (г) — после `limit_nesting` ведущих пробелов ровно 256, остальное без изменений; текст без превышений — `Cow::Borrowed`.
### AC-11: Выключенные и включённые расширения разбора
- **Дано:** (а) `T\n: d`; (б) `# H {#id}`; (в) `>>>\nx\n>>>`.
- **Действие:** `render_fragment`.
- **Ожидание:** (а) нет `<dl`; (б) нет `id="id"`, текст `{#id}` виден; (в) `<blockquote>` с `x`.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства в репозитории ещё нет, фрагмент виден только через хосты (#43).
