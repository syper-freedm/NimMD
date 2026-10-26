# Функциональный дизайн: Обрезать большие файлы и проверить скорость рендера

## Метаданные [@for: all]
- Issue: #45
- Size: M
- Владелец: Системный аналитик

## Цель [@for: all]
- Файл 1 МБ (проза, сплошной код, JSON, CSV) рендерится ядром ≤ 100 мс на Apple Silicon (спек 8, план ядра Task 7), а порог проверяется автоматически: жёстко — `just bench-check` локально, мягко (×3) — в CI.
- Замер в release до правки: проза 349 мс, сплошной код Rust 2,18 с, JSON 328 мс, CSV 3 мс; без подсветки код 61 мс, проза 34 мс. Время уходит на разбор syntect (fancy-regex): Python ≈ 3,6 мкс/байт, Rust ≈ 2,3, YAML ≈ 0,8, TOML ≈ 0,7, JSON ≈ 0,3. `onig` ускоряет в 2–2,5 раза и порог не берёт (код 0,84 с), поэтому по решению пользователя — только лимит объёма подсветки (спек 5.1, В5), без `onig`.

## Границы [@for: all]
- Входит: FR-1…FR-7. Обрезка 10 МБ с баннером уже сделана в #43 (`text::MAX_BYTES`, `truncate_at_char_boundary`, баннер `truncated`) — здесь только проверка на реальном размере (FR-1).
- Не входит: `onig`; холодный старт (загрузка дампа и компиляция регулярных выражений при первом рендере, В7); хосты и FFI; лимит спанов редактора сверх смены значения `HIGHLIGHT_LIMIT`.

## Целевое поведение [@for: all]
- Markdown: блоки кода подсвечиваются, пока их суммарный объём укладывается в 8 КБ на документ; блок, который не помещается, показывается простым текстом, внизу документа — баннер. Блоки без языка и с нераспознанным языком подсвечиваются как раньше и лимит не тратят.
- Файлы данных: TOML/YAML/JSON подсвечиваются первые 64 КБ (было 2 МБ), CSV раскрашивается в первых 2 МБ (как было); баннер называет действующий лимит.
- Файл больше 10 МБ: показаны первые 10 МБ и баннер «Файл обрезан» (без изменений).

## Требования [@for: all]
- **FR-1:** Обрезка (К-Р11) не меняется: `RenderOptions::default().max_bytes == text::MAX_BYTES` (10 МБ); вход длиннее режется по границе символа, `Rendered.truncated = true`, в конце `<main>` — `<div class="banner truncated">Файл обрезан: показаны первые 10 МБ</div>`.
- **FR-2:** В `crates/nimmd-core/src/highlight.rs` три публичные константы (реэкспорт из `lib.rs`): `HIGHLIGHT_LIMIT = 64 * 1024` — голова файла TOML/YAML/JSON и спаны `highlight`; `CSV_COLOR_LIMIT = 2 * 1024 * 1024` — голова раскраски CSV; `MARKDOWN_HIGHLIGHT_LIMIT = 8 * 1024` — суммарный объём подсвечиваемого кода одного документа Markdown. Правило головы (`split_head`: до последнего `\n` включительно, без `\n` — до границы символа не дальше лимита) — прежнее, лимит — параметр.
- **FR-3:** Баннер лимита данных — `<div class="banner">Подсветка отключена после N</div>`, где `N` берётся из действующего лимита: `64 КБ` для TOML/YAML/JSON, `2 МБ` для CSV (меньше 1 МиБ — `{байт / 1024} КБ`, иначе `{байт / 1 МиБ} МБ`). Остальное поведение данных (#44 FR-6) не меняется.
- **FR-4:** Лимит Markdown. В один вызов разбора тела (`render_body`, а значит `render` и `render_fragment`) — свой счётчик, глобального изменяемого состояния нет (рендеры в разных потоках не влияют друг на друга). Подсвечиваемые фрагменты идут в порядке документа: frontmatter (YAML), затем блоки кода. Фрагмент длиной `n` байт, чья грамматика (правило выбора #44 FR-5) не Plain Text, подсвечивается, если `использовано + n ≤ MARKDOWN_HIGHLIGHT_LIMIT` (тогда `использовано += n`), иначе выводится экранированным текстом в той же обёртке (`<pre class="syntax-highlighting"><code …>` для блока, `<pre class="frontmatter">` для frontmatter), и документ помечается ограниченным; следующие фрагменты проверяются тем же правилом. Фрагменты Plain Text подсвечиваются как раньше и счётчик не меняют; блоки `mermaid` идут мимо подсветки, как раньше.
- **FR-5:** Документ Markdown, где хоть один фрагмент вывелся без подсветки по FR-4: `Rendered.highlight_limited = true`, после фрагмента и до баннера `truncated` — `<div class="banner">Часть кода показана без подсветки: лимит 8 КБ на документ</div>`. `render_fragment` баннер не добавляет. Документ без превышения — HTML байт в байт как до правки.
- **FR-6:** Бенчмарк `crates/nimmd-core/benches/render.rs` (criterion, `[[bench]] name = "render", harness = false`; `criterion` из `[workspace.dependencies]` в dev-зависимости ядра) — четыре входа ≥ 1 МиБ, собранные в коде повторением фрагмента: проза (заголовки, абзацы с выделением и ссылками, список, таблица, блок ` ```rust `), сплошной код (блоки ` ```rust `, ` ```python `, ` ```js `, ` ```bash ` по кругу), JSON (`kind: Json`), CSV с `;` и десятичной запятой (`kind: Csv`); `RenderOptions::default()` с нужным `kind`. Генераторы входов — один файл, общий для бенчмарка и FR-7 (`#[path]`). `just bench` запускает бенчмарк.
- **FR-7:** Порог: `crates/nimmd-core/tests/perf.rs`, тест `#[ignore]` (в `cargo test` без `--ignored` не идёт): на каждом входе FR-6 один прогрев, затем медиана 5 рендеров ≤ `100 мс × F`, где `F` — целое из переменной окружения `NIMMD_BENCH_FACTOR` (нет или не число — 1). При провале сообщение называет вход, медиану и порог; при успехе тест печатает медианы всех входов. Рецепт `just bench-check` = `cargo test --release -p nimmd-core --test perf -- --ignored --nocapture`. CI (`.github/workflows/ci.yml`, джоба `test`): после `just test` шаг `just bench-check` с `NIMMD_BENCH_FACTOR: "3"` только на `macos-latest` (Apple Silicon, `if: runner.os == 'macOS'`): на общих x86-раннерах Linux и Windows ×3 от порога Apple Silicon ненадёжен.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход недоверенный: код без подсветки идёт в HTML только экранированным (как в #88), текст баннеров — константы ядра. Функции тотальны, `unwrap`/`expect` в рабочем коде нет; счётчик лимита считает с насыщением.
- Лимиты в байтах, а не во времени: вывод детерминирован и не зависит от машины.
- `criterion` и его зависимости — только dev; лицензии проверяет `cargo deny` (`just lint`). Новые `crates.io`-зависимости скачиваются из сети.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-core/src/highlight.rs` (константы, `split_head` с лимитом, счётчик Markdown в адаптере), `crates/nimmd-core/src/render.rs` (плагины comrak на вызов вместо глобального `OnceLock`, флаг ограничения в `Body`, frontmatter под лимитом), `crates/nimmd-core/src/data.rs` (лимит по типу, текст баннера), `crates/nimmd-core/src/document.rs` (баннер и `highlight_limited` для Markdown, doc-комментарий поля), `crates/nimmd-core/src/lib.rs`, `crates/nimmd-core/Cargo.toml`, `Cargo.lock`; `crates/nimmd-core/benches/render.rs`, `crates/nimmd-core/benches/inputs.rs` (новые); infra: `justfile` (`bench-check`), `.github/workflows/ci.yml`; `CLAUDE.md` (строка порога и команда `just bench-check` вместо «бенчмарков пока нет»).
- Frontend: не затрагивается.
- Новые тесты: `crates/nimmd-core/tests/perf.rs` (FR-7); `crates/nimmd-core/tests/limits.rs` — FR-1, FR-4, FR-5 (AC-1…AC-4); правка существующих `crates/nimmd-core/tests/data.rs`, `crates/nimmd-core/tests/highlight.rs` под новые значения констант и текст баннера.

## Критерии приёмки [@for: all]
### AC-1: Обрезка 10 МБ
- **Дано:** текст `kind: Text` = `"a"` + `"ё"` × `MAX_BYTES / 2` (длина `MAX_BYTES + 1` байт, граница `MAX_BYTES` внутри символа) и текст ровно `MAX_BYTES` байт ASCII.
- **Действие:** `render` с `RenderOptions::default()` и нужным `kind`.
- **Ожидание:** первый — `truncated`, без паники, есть `<div class="banner truncated">Файл обрезан: показаны первые 10 МБ</div>`, видимый текст `<pre>` — префикс входа длиной `MAX_BYTES - 1` байт; второй — `truncated == false`, баннера нет.

### AC-2: Лимит подсветки Markdown
- **Дано:** Markdown: блок ` ```rust ` 5 КБ, блок ` ```python ` 5 КБ, блок ` ```rust ` 2 КБ, блок без языка 20 КБ, блок ` ```mermaid `.
- **Действие:** `render` с `RenderOptions::default()`.
- **Ожидание:** первый и третий блоки содержат `<span class="source rust`, второй — `<pre class="syntax-highlighting"><code>` (обёртка та же, что у подсвеченных блоков: `github_pre_lang` не даёт `<code>` класса языка) с экранированным текстом без `<span`; блок без языка содержит `<span class="text plain`; mermaid — `<pre class="mermaid">`; `highlight_limited == true`; баннер `Часть кода показана без подсветки: лимит 8 КБ на документ` стоит после фрагмента и перед `</main>`.

### AC-3: Ровно на лимите, frontmatter, независимость вызовов
- **Дано:** (а) один блок ` ```rust ` ровно `MARKDOWN_HIGHLIGHT_LIMIT` байт; (б) frontmatter YAML 9 КБ и блок ` ```rust ` 1 КБ; (в) документ (а) и тот же документ с блоком на 1 байт длиннее, отрендеренные по очереди и затем снова (а).
- **Действие:** `render`.
- **Ожидание:** (а) подсвечен, `highlight_limited == false`, баннера нет; (б) frontmatter без `<span`, блок rust подсвечен, `highlight_limited == true`; (в) повторный (а) совпадает с первым байт в байт.

### AC-4: Документы без превышения не меняются
- **Дано:** существующие снапшоты `crates/nimmd-core/tests/snapshots/` и фикстуры `crates/nimmd-core/tests/fixtures/`.
- **Действие:** `cargo test --workspace`.
- **Ожидание:** снапшоты не изменились (`git status --short crates/nimmd-core/tests/snapshots` пуст).

### AC-5: Лимиты данных
- **Дано:** JSON `{"k": 1}\n` × (`HIGHLIGHT_LIMIT / 9 + 100`); YAML и TOML того же устройства; CSV `a;b\n` × (`HIGHLIGHT_LIMIT / 4 + 100`); CSV `a;b\n` × (`CSV_COLOR_LIMIT / 4 + 100`).
- **Действие:** `render` с соответствующим `kind`.
- **Ожидание:** JSON/YAML/TOML — `highlight_limited`, баннер `Подсветка отключена после 64 КБ`, видимый текст `<pre>` равен входу; первый CSV — `highlight_limited == false`, последний `a;b` раскрашен; второй CSV — `highlight_limited`, баннер `Подсветка отключена после 2 МБ`; `HIGHLIGHT_LIMIT == 65536`, `CSV_COLOR_LIMIT == 2097152`, `MARKDOWN_HIGHLIGHT_LIMIT == 8192`.

### AC-6: Порог скорости
- **Дано:** входы FR-6, release-профиль, Apple Silicon.
- **Действие:** `just bench-check`; `NIMMD_BENCH_FACTOR=3 just bench-check`; `just bench`; `cargo test --workspace`.
- **Ожидание:** `just bench-check` зелёный, напечатаны медианы четырёх входов, каждая ≤ 100 мс; с фактором 3 — зелёный; `just bench` выполняет четыре группы; в `cargo test --workspace` тест порога значится `ignored`.

### AC-7: Порог в CI и проверки проекта
- **Дано:** `.github/workflows/ci.yml`, `justfile`, `CLAUDE.md`.
- **Действие:** `just lint`; чтение файлов.
- **Ожидание:** `just lint` зелёный (`actionlint`, `zizmor`, `cargo deny` с `criterion`); в джобе `test` шаг `just bench-check` с `NIMMD_BENCH_FACTOR: "3"` и `if: runner.os == 'macOS'` после `just test`; `CLAUDE.md` называет `just bench-check` и не говорит «бенчмарков пока нет».

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства нет; поведение описывают doc-комментарии констант и `Rendered.highlight_limited`.
