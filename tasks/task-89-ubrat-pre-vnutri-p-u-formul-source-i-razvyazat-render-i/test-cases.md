# Тестовые сценарии: Убрать <pre> внутри <p> у формул Source и развязать render и document

## Метаданные
- Issue: #89
- Стек: `cargo test` (интеграционные тесты `crates/nimmd-core/tests/document.rs`, снапшоты `insta`), `git grep`/`git diff`, `just lint`.
- Контроль доступа: нет (библиотека, вход — строка); объектные сценарии неприменимы.

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001 |
| AC-2 | SC-002, SC-003 |
| AC-3 | SC-004 |
| AC-4 | SC-005 |
| AC-5 | SC-006, SC-007 |

## Сценарии
### SC-001: Source — display как `<code class="math-display">`
- **Покрывает:** AC-1
- **Уровень:** unit
- **Предусловия:** `FORMULAS` = `"$x^2$\n\n$$\\frac{a}{b}$$\n"`.
- **Шаги:** `render(FORMULAS, Source)`; тест `math_source_mode`.
- **Ожидание:** `body` содержит `<code>x^2</code>` и `<p><code class="math-display">\frac{a}{b}</code></p>`; нет `<pre`; `html_classes(body)` содержит `math-display`; снапшот `document__math_source_mode.snap` принят и совпадает.
- **Результат выполнения:** pass

### SC-002: Ни в одном режиме нет `<pre`
- **Покрывает:** AC-2
- **Уровень:** unit
- **Предусловия:** `FORMULAS` и `"$$\\nosuchcommand{x}$$\n"`.
- **Шаги:** `render` обоих входов с `Katex`, `MathML`, `Source`.
- **Ожидание:** ни в одном из 6 `body` нет `<pre`; `MathML`-`body` второго входа содержит `<code class="math-display">\nosuchcommand{x}</code>`.
- **Результат выполнения:** pass

### SC-003: Снапшот MathML не изменён
- **Покрывает:** AC-2
- **Уровень:** static
- **Предусловия:** ветка задачи.
- **Шаги:** `git diff --exit-code main -- crates/nimmd-core/tests/snapshots/document__math_mathml_mode.snap`; `cargo test -p nimmd-core --test document math_mathml_mode`.
- **Ожидание:** diff пуст (код 0), тест зелёный.
- **Результат выполнения:** pass

### SC-004: Опасный TeX экранирован
- **Покрывает:** AC-3
- **Уровень:** unit
- **Предусловия:** `"$$\\text{<script>}$$"`, `"$$a<b&c$$"`.
- **Шаги:** `render` каждого с `Source`.
- **Ожидание:** нет `<script`; есть `<code class="math-display">\text{&lt;script&gt;}</code>` и `<code class="math-display">a&lt;b&amp;c</code>`.
- **Результат выполнения:** pass

### SC-005: Класс в контракте и теме
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** правка `CONTRACT.md` и `nimmd.css`.
- **Шаги:** `git grep -n "math-display" packages/theme/`; `cargo test -p nimmd-core --test document`.
- **Ожидание:** `- \`math-display\`` стоит в разделе «Только ядра»; в `nimmd.css` есть селектор `.math-display`; `class_contract`, `contract_core_classes_appear_in_output` и SC-001 зелёные.
- **Результат выполнения:** pass

### SC-006: `MathMode` живёт в `render.rs`
- **Покрывает:** AC-5
- **Уровень:** static
- **Предусловия:** дерево после правки.
- **Шаги:** `git grep -n "crate::document" crates/nimmd-core/src/render.rs`; `git grep -n "enum MathMode" crates/`.
- **Ожидание:** первая — пусто (код 1); вторая — одна строка в `crates/nimmd-core/src/render.rs`.
- **Результат выполнения:** pass

### SC-007: Публичный путь и полный прогон
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** ветка задачи.
- **Шаги:** `git diff main -- crates/nimmd-core/tests/ | grep "^[-+]use "`; `cargo test --workspace`; `just lint`.
- **Ожидание:** строки `use nimmd_core::…` не менялись; `rendered_serializes_lowercase` и весь `cargo test --workspace` зелёные; `just lint` зелёный.
- **Результат выполнения:** pass
