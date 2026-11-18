# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Что это

NimMD — быстрый лёгкий просмотрщик и редактор Markdown для macOS. Цель — одно Rust-ядро `nimmd-core`
для двух хостов: `NimMD.app` (Tauri 2 + редактор на TS/CodeMirror 6, ядро в том же процессе)
и Quick Look-расширение на Swift (через C ABI `nimmd-ffi`). В репозитории есть ядро,
`nimmd-assets`, каркас `apps/desktop`, тема и сборка KaTeX и Mermaid.

Источники истины (локальные, `docs/` в `.gitignore`):
- спек: `docs/superpowers/specs/2026-10-19-nimmd-design.md` — в коде и комментариях на него
  ссылаются как «спек 5.1», «спек 6» и т. п.; раздел 7 — целевая структура репозитория;
- план ядра: `docs/superpowers/plans/2026-10-19-nimmd-core.md` (Task 0…N, «Global Constraints»);
- ADR: `docs/adr/`.

Артефакты задач конвейера `autonomous-feature-delivery` лежат в `tasks/<тип>-<N>-<slug>/`;
стек и команды по областям — в `.claude/stack.json`.

## Команды

```sh
just test          # cargo test --workspace + node --test scripts/*.test.mjs
just lint          # всё: fmt-check clippy doc semgrep deny vendor npm-licenses unused-deps secrets workflows
just clippy        # cargo clippy --workspace --all-targets -- -D warnings
just snap          # принять снапшоты insta (cargo install cargo-insta); insta подключат к ядру в Task 3 плана
just vendor        # собрать KaTeX и Mermaid в packages/vendor/dist (нужен npm-реестр)
cargo test -p nimmd-core --test text strip_bom   # один тест: файл tests/text.rs, фильтр по имени
cargo fmt --all
```

`just lint` требует установленных `semgrep`, `cargo-deny`, `cargo-machete`, `gitleaks`,
`actionlint`, `zizmor`, `pnpm` и сети (`vendor` ставит пакеты из npm, `deny` качает базу RustSec).
В CI проверка приложения, тесты, бенчмарк и линтеры запускаются на macOS.

## Инварианты (ловятся линтерами — не обходить)

- `nimmd-core`: `#![forbid(unsafe_code)]`, никакого `std::fs/net/process` (правило
  `nimmd-core-no-io`): строка на входе, результат на выходе. Ресурсы (KaTeX, mermaid.js) — без сети.
- Паника в ядре роняет Quick Look: `unwrap`/`expect`/`panic!`/`todo!` в рабочем коде запрещены
  workspace-линтами (в тестах разрешены через `clippy.toml`). В `text.rs` дополнительно
  запрещены индексация и срезы строк — функции тотальны на любом входе.
- Документ недоверенный: comrak только с `render.unsafe_ = false`; Mermaid только с
  `securityLevel: 'strict'`; KaTeX только с `trust: false`; в DOM — без `innerHTML` и подобных;
  CSP WebView не ослабляется; WebView не получает плагины `fs/shell/process/http`. Все правила —
  в `.semgrep.yml`.
- `unsafe` — только в будущих `nimmd-ffi` и `apps/desktop/src-tauri/src/platform/`, с `// SAFETY:`.
  Интеграция с macOS живёт в `apps/desktop/src-tauri/src/platform/`,
  `apps/macos-quicklook/` и `scripts/*-macos.sh`.
- clippy `pedantic` + `-D warnings`. Правило ослабляется только точечным `#[allow(...)]` или
  `nosemgrep: <id>` с причиной рядом.
- Лицензии: белый список в `deny.toml` общий для crates и npm (`scripts/check-npm-licenses.mjs`).
  В `packages/vendor` elkjs исключён override'ом в `pnpm-workspace.yaml`; `ignore-scripts=true`.

## Поведение ядра (план, «Global Constraints»)

- Вход — UTF-8; BOM отрезается, стиль переносов (`\n`/`\r\n`) определяется и возвращается хосту.
- Больше 10 МБ (`text::MAX_BYTES`) рендерятся первые 10 МБ по границе символа, `truncated = true`.
- Ключ `markdown:` во frontmatter побеждает эвристику детектора диалекта.
- HTML-фрагмент не зависит от `Profile`: от профиля зависят только подключаемые скрипты и
  поле `Rendered.profile`, поэтому ошибка детектора не портит отображение.
- Порог: рендер файла 1 МБ ≤ 100 мс на Apple Silicon: `just bench-check` (тест `tests/perf.rs`,
  в CI на macOS с `NIMMD_BENCH_FACTOR=3`), замеры — `just bench` (criterion).

## Соглашения

- Тексты, комментарии, doc-комментарии, сообщения коммитов — на русском (идентификаторы — нет).
- Коммиты: `feat(#N): …`, `chore(#N): …`, где `#N` — issue в `syper-freedm/NimMD`.
- Тесты ядра — интеграционные, в `crates/nimmd-core/tests/<модуль>.rs`.
- Rust 2024 edition, stable toolchain, MSRV 1.85.
