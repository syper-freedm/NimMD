# Функциональный дизайн: Экранировать код блока при ошибке подсветки syntect

## Метаданные [@for: all]
- Issue: #88
- Size: S
- Владелец: Системный аналитик

## Цель [@for: all]
- Текст недоверенного документа никогда не попадает в HTML неэкранированным: при ошибке подсветки syntect блок кода и YAML frontmatter выводятся экранированным текстом без подсветки (SEC-003 из #41).

## Границы [@for: all]
- Входит: собственный адаптер подсветки ядра вместо `comrak::plugins::syntect::SyntectAdapter` для блоков кода и frontmatter; прямая зависимость `syntect` той же версии, что тянет comrak.
- Не входит: свой дамп грамматик, TOML и `highlight -> Vec<Span>` (#44); изменение классов и вида подсветки; блоки `mermaid` (у них свой экранирующий рендерер).

## Целевое поведение [@for: all]
- Успешная подсветка: HTML блока кода и frontmatter байт-в-байт тот же, что сейчас (`<pre class="syntax-highlighting"><code class="language-…">` + `<span class="…">`).
- Ошибка подсветки (`Err` от syntect): содержимое `<code>` — исходный текст блока, экранированный `html::escape` comrak, без `<span>` подсветки; обёртка `<pre>`/`<code>` та же.

## Требования [@for: all]
- **FR-1:** В `crates/nimmd-core/src/render.rs` адаптер `Highlighter` (реализует `comrak::adapters::SyntaxHighlighterAdapter`) на `syntect::parsing::SyntaxSet::load_defaults_newlines()` в `OnceLock` заменяет `SyntectAdapter` и в `plugins()`, и в `highlight_yaml`. Выбор грамматики как у comrak 0.56: первое слово до `,` в `lang` → `find_syntax_by_token`, иначе `find_syntax_by_first_line(code)`, иначе Plain Text; пустой или отсутствующий `lang` — Plain Text. Подсветка — `ClassedHTMLGenerator` с `ClassStyle::Spaced` построчно по `LinesWithEndings`. `write_pre_tag` пишет `<pre class="syntax-highlighting">` без атрибутов comrak (как `SyntectAdapter`: при `github_pre_lang` язык блока в разметку не попадает), `write_code_tag` — `<code>` с атрибутами comrak через `html::write_opening_tag`.
- **FR-2:** При `Err` на любой строке весь уже подсвеченный вывод блока отбрасывается, в выход пишется `html::escape(code)`. Ветка ошибки выделена в функцию, принимающую результат подсветки (`Result<String, syntect::Error>`) и исходный текст, чтобы её можно было проверить без входа, на котором syntect падает (такого входа не найдено).
- **FR-3:** `syntect` добавляется в `[workspace.dependencies]` корневого `Cargo.toml` и в `crates/nimmd-core/Cargo.toml` с `default-features = false` и только нужными функциями (`default-syntaxes`, `html`, `regex-fancy` — тот же движок регулярных выражений, что у `comrak` с `syntect-fancy`), версия совпадает с `Cargo.lock` (5.3) — второго экземпляра крейта в дереве нет. `SyntectAdapter` в ядре больше не используется.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход — недоверенный текст блока кода и frontmatter. Ошибка подсветки не паникует и не теряет текст: блок выводится экранированным целиком.
- Инварианты ядра сохраняются: `forbid(unsafe_code)`, без `std::fs/net/process`, без `unwrap`/`expect` в рабочем коде, `render.unsafe_ = false`.
- Лицензия `syntect` (MIT) уже разрешена — крейт есть в дереве через comrak.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/render.rs`, `crates/nimmd-core/Cargo.toml`, `Cargo.toml`, `Cargo.lock`.
- Frontend: не затрагивается.
- Новые тесты: модульный `#[cfg(test)]`-тест в `crates/nimmd-core/src/render.rs` на функцию FR-2 (подставной `Err` → экранированный текст; `Ok` → текст подсветки без изменений) — функция приватная, из `crates/nimmd-core/tests/` до неё не дотянуться, а публичный API для подмены подсветки не нужен; интеграционный тест в `crates/nimmd-core/tests/render.rs`: существующий снапшот `code_highlighted_with_classes` и frontmatter-тесты `crates/nimmd-core/tests/document.rs` не меняются, плюс блок с неизвестным языком и `<script>` в тексте выводится экранированным.

## Критерии приёмки [@for: all]
### AC-1: Ошибка подсветки → экранированный текст
- **Дано:** результат подсветки `Err(syntect::Error)` (подставной) и текст блока `<script>alert(1)</script> & "x"`.
- **Действие:** вызов функции FR-2.
- **Ожидание:** вывод равен `&lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;x&quot;` — тому же, что `html::escape` comrak; ни `<script`, ни `<span` нет.

### AC-2: Успешная подсветка не меняется
- **Дано:** результат `Ok("<span class=\"source\">x</span>")`; документы существующих снапшотов и тестов frontmatter.
- **Действие:** вызов функции FR-2; `cargo test -p nimmd-core`.
- **Ожидание:** функция пишет строку `Ok` без изменений; снапшот `render__code_highlighted_with_classes.snap` и тесты frontmatter в `crates/nimmd-core/tests/document.rs` проходят без правки снапшотов.

### AC-3: Неизвестный язык и опасный текст в блоке
- **Дано:** `` ```nosuchlang\n<script>x</script>\n``` `` и `` ```\n<img src=x onerror=y>\n``` ``.
- **Действие:** `render_fragment`.
- **Ожидание:** оба блока начинаются с `<pre class="syntax-highlighting"><code>` (язык блока в разметку не попадает — `github_pre_lang`, как до правки), текст экранирован: есть `&lt;script&gt;` и `&lt;img`, нет `<script`, `<img`.

### AC-4: Адаптер comrak убран, второго syntect нет
- **Дано:** дерево после правки.
- **Действие:** `git grep -n SyntectAdapter crates/`; `cargo tree -p nimmd-core -d -e normal`.
- **Ожидание:** `SyntectAdapter` не встречается; `syntect` не дублируется; `just lint` и `cargo test --workspace` зелёные.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства нет, вывод при успешной подсветке не меняется.
