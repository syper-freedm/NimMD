# Функциональный дизайн: Убрать <pre> внутри <p> у формул Source и развязать render и document

## Метаданные [@for: all]
- Issue: #89
- Size: S
- Владелец: Системный аналитик

## Цель [@for: all]
- В режиме `MathMode::Source` display-формула выводится валидным HTML: без `<pre>` внутри `<p>`, который браузер разрывает (ARCH-004 из #43).
- Зависимость модулей ядра односторонняя: `document.rs` зависит от `render.rs`, но не наоборот (ARCH-005 из #43).

## Границы [@for: all]
- Входит: вывод display-формулы в режиме `Source`; перенос `MathMode` из `crates/nimmd-core/src/document.rs` в `crates/nimmd-core/src/render.rs`; новый класс в `packages/theme/CONTRACT.md` и его стиль в `packages/theme/nimmd.css`.
- Не входит: режимы `Katex` и `MathML` (их вывод не меняется); inline-формулы `Source`; публичный путь `nimmd_core::MathMode` и его serde-представление (не меняются).

## Целевое поведение [@for: all]
- `Source`, display-формула (`$$…$$`): `<code class="math-display">` + экранированный TeX + `</code>`. Элемент строчный, поэтому допустим внутри `<p>`, в котором comrak выводит формулу; блоком его делает CSS темы.
- `Source`, inline-формула: `<code>…</code>`, как сейчас.
- Хост, использующий `nimmd_core::MathMode`, ничего не меняет: тип тот же, путь реэкспорта тот же.

## Требования [@for: all]
- **FR-1:** `source()` в `crates/nimmd-core/src/render.rs` для `display_math` выводит `<code class="math-display">{экранированный TeX}</code>`, для inline — `<code>{экранированный TeX}</code>`. Это же значение используется как запасной вывод `MathML` (формула не преобразована), чтобы `<pre>` внутри `<p>` не появлялся ни в одном режиме. Экранирование — `push_escaped`, как сейчас.
- **FR-2:** `packages/theme/CONTRACT.md`, список «Только ядра», получает `math-display`; `packages/theme/nimmd.css` — правило `.math-display` (блочное отображение, `white-space: pre-wrap`, отступ и фон как у `pre`), чтобы формула показывалась отдельным блоком.
- **FR-3:** `enum MathMode` (с doc-комментариями, `derive` и `#[serde(rename_all = "lowercase")]`) переносится из `crates/nimmd-core/src/document.rs` в `crates/nimmd-core/src/render.rs`; `document.rs` импортирует его из `render`; `crates/nimmd-core/src/lib.rs` реэкспортирует `MathMode` из `render`. В `render.rs` не остаётся `use crate::document`.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход — недоверенный TeX формулы; он остаётся экранированным (`push_escaped`), класс — константа ядра, из документа в атрибут ничего не попадает.
- Ошибок нет, API не меняется. Инварианты ядра (`forbid(unsafe_code)`, без `std::fs/net/process`, без `unwrap` в рабочем коде) сохраняются.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/render.rs`, `crates/nimmd-core/src/document.rs`, `crates/nimmd-core/src/lib.rs`, `packages/theme/CONTRACT.md`, `packages/theme/nimmd.css`; адаптация существующего теста `math_source_mode` в `crates/nimmd-core/tests/document.rs` под новый вывод.
- Frontend: не затрагивается.
- Новые тесты: в `crates/nimmd-core/tests/document.rs` — insta-снапшот `body` для `Source` (FR-1), проверка, что ни в одном режиме нет `<p><pre`, и класс `math-display` в тесте основных классов контракта (FR-2); на FR-3 — тест не нужен, его проверяют сборка и `git grep`.

## Критерии приёмки [@for: all]
### AC-1: Display-формула Source без `<pre>` в `<p>`
- **Дано:** `"$x^2$\n\n$$\\frac{a}{b}$$\n"`.
- **Действие:** `render` с `MathMode::Source`.
- **Ожидание:** `body` содержит `<code>x^2</code>` и `<p><code class="math-display">\frac{a}{b}</code></p>`; нет `<pre`; снапшот `body` зафиксирован.

### AC-2: Ни один режим не даёт `<p><pre`
- **Дано:** тот же текст и формула, которую `MathML` не преобразует (`$$\\nosuchcommand{x}$$`).
- **Действие:** `render` с `Katex`, `MathML`, `Source`.
- **Ожидание:** ни в одном `body` нет `<pre`; у непреобразованной формулы в `MathML` — `<code class="math-display">`; снапшот `document__math_mathml_mode.snap` не меняется.

### AC-3: Опасный TeX остаётся экранированным
- **Дано:** `$$\\text{<script>}$$` и `$$a<b&c$$`.
- **Действие:** `render` с `MathMode::Source`.
- **Ожидание:** нет `<script`; есть `&lt;script&gt;` и `a&lt;b&amp;c` внутри `<code class="math-display">`.

### AC-4: Класс в контракте и теме
- **Дано:** `packages/theme/CONTRACT.md`, `packages/theme/nimmd.css`, вывод AC-1.
- **Действие:** `cargo test -p nimmd-core --test document`.
- **Ожидание:** `math-display` есть в списке «Только ядра», селектор `.math-display` есть в `nimmd.css`, тест контракта (селекторы ⊆ контракт, классы ядра есть в выводе) зелёный.

### AC-5: Односторонняя зависимость модулей
- **Дано:** дерево после правки.
- **Действие:** `git grep -n "crate::document" crates/nimmd-core/src/render.rs`; `git grep -n "enum MathMode" crates/`; `just lint`; `cargo test --workspace`.
- **Ожидание:** первая команда пуста; `enum MathMode` только в `crates/nimmd-core/src/render.rs`; тесты, использующие `nimmd_core::MathMode`, компилируются без правок импорта; `just lint` и `cargo test --workspace` зелёные.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства нет; публичный API не меняется, новый класс описан в `packages/theme/CONTRACT.md`.
