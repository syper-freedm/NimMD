# NimMD

Быстрый лёгкий просмотрщик и редактор Markdown. По пробелу в Finder файл сразу показывается
отформатированным: NimMD сам определяет диалект Markdown и расширения. Одно Rust-ядро
обслуживает и приложение, и Quick Look, поэтому документ выглядит одинаково.

## Структура

- `crates/nimmd-core` — ядро: Markdown и файлы данных в HTML ([README](crates/nimmd-core/README.md)).
- `crates/nimmd-assets` — безопасное чтение картинок, на которые ссылается документ.
- `apps/desktop` — каркас Tauri 2: окно, режим запуска и путь из командной строки.
- `packages/theme` — CSS тем и подсветки, скрипты инициализации KaTeX и Mermaid.
- `packages/vendor` — сборка KaTeX и Mermaid без сети в рантайме.

## Команды

```sh
just test     # тесты Rust, скриптов и frontend (Vitest)
just lint     # форматирование, clippy, semgrep, лицензии, секреты, CI
just bench    # замеры рендера (criterion)
just vendor   # собрать KaTeX и Mermaid в packages/vendor/dist
just desktop-check # cargo check приложения и сборка frontend
```

## Приложение

Нужны stable Rust, Node.js 26, pnpm 12.18.0 и
[системные зависимости Tauri](https://v2.tauri.app/start/prerequisites/) для вашей ОС.
Tauri закреплён на 2.11.6 / tauri-build 2.6.3: следующие minor-версии требуют
Rust 1.90 при MSRV проекта 1.85. TypeScript 6.0.3 совместим с текущим typescript-eslint;
TypeScript 7 пока не поддерживается его проверками типов.

```sh
just desktop-deps
pnpm -C apps/desktop tauri dev -- -- file.md
pnpm -C apps/desktop tauri dev -- -- --view file.md
pnpm -C apps/desktop tauri build --debug --no-bundle
```

После сборки `target/debug/nimmd file.md` выбирает редактирование,
`target/debug/nimmd --view file.md` — просмотр. Для имени с дефисом используйте
`nimmd -- -file.md`. Без аргументов открывается пустое окно. Неизвестные параметры,
несколько файлов и пути не в UTF-8 завершают запуск с ошибкой.

Сейчас окно показывает режим и путь: чтение, редактор, рендер и сохранение файлов
добавляются в следующих задачах. `app/` запрещает unsafe; `platform/` содержит
контракт `Platform` для метаданных, признака изменений и иконки документа.
Эти три операции пока возвращают `Unsupported` и при запуске не вызываются.
WebView может только запросить параметры запуска, без доступа к файловым и сетевым плагинам.

Платформенные конфиги задают `.app` для macOS, NSIS с автономным установщиком
WebView2 для Windows и AppImage для Linux. `bundle.fileAssociations` генерирует
`CFBundleDocumentTypes`: Markdown — `Editor`, TOML/YAML/JSON/CSV/текст — `Viewer`.
Обработка открытия через файловый менеджер будет добавлена вместе с открытием файлов.
Локальный macOS-бандл: `pnpm -C apps/desktop tauri build --debug --bundles app`.
CI запускает `cargo check` приложения, Rust-тесты и Vitest на macOS, Windows и Linux.
