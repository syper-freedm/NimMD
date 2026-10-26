# Тестовые сценарии: Обрезать большие файлы и проверить скорость рендера

## Метаданные
- Issue: #45
- Стек: `cargo test` (`crates/nimmd-core/tests/*.rs`), `just bench`, `just bench-check`. Ролей нет (библиотека) — сценарии доступа неприменимы.
- «Видимый текст `<pre>`» — `<pre>` без тегов, сущности раскрыты. Длина фрагмента — текст блока между оградами с последним `\n`.

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001, SC-002 |
| AC-2 | SC-003 |
| AC-3 | SC-004, SC-005, SC-006 |
| AC-4 | SC-007 |
| AC-5 | SC-008, SC-009 |
| AC-6 | SC-010, SC-011 |
| AC-7 | SC-012, SC-013 |

## Сценарии
### SC-001: Граница 10 МБ внутри символа
- **Покрывает:** AC-1
- **Уровень:** integration
- **Предусловия:** `src = "a" + "ё".repeat(MAX_BYTES / 2)`, `src.len() == MAX_BYTES + 1`.
- **Шаги:** `render(src, kind: Text)`.
- **Ожидание:** без паники; `truncated`; `body` кончается на `<div class="banner truncated">Файл обрезан: показаны первые 10 МБ</div></main>`; видимый текст `<pre>` == `src[..MAX_BYTES - 1]`.
- **Результат выполнения:** pass

### SC-002: Ровно 10 МБ не режется
- **Покрывает:** AC-1
- **Уровень:** integration
- **Предусловия:** `"a".repeat(MAX_BYTES)`; `RenderOptions::default().max_bytes == MAX_BYTES`.
- **Шаги:** `render(…, kind: Text)`.
- **Ожидание:** `truncated == false`, нет `banner truncated`.
- **Результат выполнения:** pass

### SC-003: Лимит 8 КБ на блоки кода
- **Покрывает:** AC-2
- **Уровень:** integration
- **Предусловия:** Markdown из AC-2.
- **Шаги:** `render` с `RenderOptions::default()`.
- **Ожидание:** блоки 1 и 3 — `<span class="source rust`; блок 2 — `<pre class="syntax-highlighting"><code>` (как у подсвеченных), без `<span`, видимый текст == коду; блок без языка — `<span class="text plain`; `<pre class="mermaid">`; `highlight_limited`; баннер `Часть кода показана без подсветки: лимит 8 КБ на документ` ровно один, после блока 2, перед `</main>`.
- **Результат выполнения:** pass

### SC-004: Ровно на лимите
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** один ` ```rust `, текст == `MARKDOWN_HIGHLIGHT_LIMIT` байт.
- **Шаги:** `render`.
- **Ожидание:** есть `<span class="source rust`; `highlight_limited == false`; баннера лимита нет.
- **Результат выполнения:** pass

### SC-005: Frontmatter тратит лимит первым
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** AC-3 (б).
- **Шаги:** `render`.
- **Ожидание:** `<pre class="frontmatter">` без `<span`; блок rust с `<span class="source rust`; `highlight_limited`.
- **Результат выполнения:** pass

### SC-006: Счётчик не переживает вызов
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** A — документ SC-004, B — блок на 1 байт длиннее.
- **Шаги:** `render(A)`, `render(B)`, `render(A)`.
- **Ожидание:** B — `highlight_limited`; третий `html` == первому байт в байт.
- **Результат выполнения:** pass

### SC-007: Снапшоты не меняются
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** нет
- **Шаги:** `cargo test --workspace`; `git status --short crates/nimmd-core/tests/snapshots`.
- **Ожидание:** тесты зелёные; `git status` пуст.
- **Результат выполнения:** pass

### SC-008: Лимит 64 КБ для JSON/YAML/TOML
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** строки `{"k": 1}\n`, `k: 1\n`, `k = 1\n`, каждая × (`HIGHLIGHT_LIMIT / len + 100`).
- **Шаги:** `render` с `kind` Json/Yaml/Toml.
- **Ожидание:** каждый — `highlight_limited`, баннер `Подсветка отключена после 64 КБ`, видимый текст `<pre>` == входу.
- **Результат выполнения:** pass

### SC-009: CSV 2 МБ и значения констант
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** `a;b\n` × (`HIGHLIGHT_LIMIT / 4 + 100`) и × (`CSV_COLOR_LIMIT / 4 + 100`).
- **Шаги:** `render` с `kind: Csv`.
- **Ожидание:** первый — `highlight_limited == false`, последняя строка `<span class="c0">a</span><span class="d">;</span><span class="c1">b</span>`; второй — `highlight_limited`, баннер `Подсветка отключена после 2 МБ`; `HIGHLIGHT_LIMIT == 65536`, `CSV_COLOR_LIMIT == 2097152`, `MARKDOWN_HIGHLIGHT_LIMIT == 8192`.
- **Результат выполнения:** pass

### SC-010: Порог 100 мс
- **Покрывает:** AC-6
- **Уровень:** E2E
- **Предусловия:** Apple Silicon (`uname -m` = `arm64`).
- **Шаги:** `just bench-check`; `NIMMD_BENCH_FACTOR=3 just bench-check`.
- **Ожидание:** оба зелёные; напечатаны 4 медианы, каждая ≤ 100 мс.
- **Результат выполнения:** pass

### SC-011: Бенчмарк и ignored
- **Покрывает:** AC-6
- **Уровень:** E2E
- **Предусловия:** нет
- **Шаги:** `just bench`; `cargo test --workspace`.
- **Ожидание:** criterion выполняет 4 группы; тест из `tests/perf.rs` — `ignored`.
- **Результат выполнения:** pass

### SC-012: CI и CLAUDE.md
- **Покрывает:** AC-7
- **Уровень:** static
- **Предусловия:** нет
- **Шаги:** прочитать `ci.yml`, `justfile`, `CLAUDE.md`.
- **Ожидание:** в джобе `test` после `just test` шаг `just bench-check` с `NIMMD_BENCH_FACTOR: "3"` и `if: runner.os == 'macOS'`; рецепт `bench-check` по FR-7; `CLAUDE.md` называет `just bench-check`, фразы «бенчмарков пока нет» нет.
- **Результат выполнения:** pass

### SC-013: Проверки проекта
- **Покрывает:** AC-7
- **Уровень:** static
- **Предусловия:** сеть.
- **Шаги:** `just lint`.
- **Ожидание:** зелёный, включая `actionlint`, `zizmor`, `cargo deny` с `criterion`.
- **Результат выполнения:** pass
