# NimMD

Быстрый лёгкий просмотрщик и редактор Markdown для macOS 14+. По пробелу в Finder файл сразу показывается
отформатированным: NimMD сам определяет диалект Markdown и расширения. Одно Rust-ядро
обслуживает и приложение, и Quick Look, поэтому документ выглядит одинаково.

## Структура

- `crates/nimmd-core` — ядро: Markdown и файлы данных в HTML ([README](crates/nimmd-core/README.md)).
- `crates/nimmd-assets` — безопасное чтение картинок, на которые ссылается документ.
- `apps/desktop` — приложение Tauri 2 с режимом просмотра и каркасом редактора.
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

Для приложения нужны Rust 1.90 или новее, Node.js 26, pnpm 12.21.0 и
[инструменты сборки Tauri для macOS](https://v2.tauri.app/start/prerequisites/#macos).
Закреплены Tauri 2.12.7 и tauri-build 2.7.6. Для workspace, ядра и nimmd-assets
сохранён объявленный MSRV 1.85. TypeScript 6.0.3 совместим с текущим typescript-eslint;
TypeScript 7 пока не поддерживается его проверками типов.

```sh
just desktop-deps
pnpm -C apps/desktop tauri dev -- -- file.md
pnpm -C apps/desktop tauri dev -- -- --view file.md
pnpm -C apps/desktop tauri build --debug --no-bundle
```

На macOS откройте файл в Finder через «Открыть в программе» → NimMD или перетащите
его на окно приложения. Каждый файл получает отдельное окно с именем файла
в заголовке. Повторное открытие поднимает уже открытое окно и сохраняет его режим.

После сборки `target/debug/nimmd file.md` выбирает редактирование,
`target/debug/nimmd --view file.md` — просмотр. Для имени с дефисом используйте
`nimmd -- -file.md`. CLI принимает один файл. Без аргументов открывается или
активируется пустое окно. В том же сеансе macOS повторный вызов передаёт запрос
работающему NimMD. Относительный путь разрешается в каталоге вызывающего процесса.
Если CLI не дождался ответа после отправки запроса, открытие ещё может завершиться.
Проверьте окна NimMD перед повтором команды.

Режим просмотра форматирует Markdown (`.md`, `.markdown`, `.mdown`, `.mkd`), TOML,
YAML (`.yaml`, `.yml`), JSON и CSV и показывает `.txt` как обычный текст. Неизвестное
расширение обрабатывается как Markdown. Файлы без флага `--view`, из Finder или
перетаскивания открываются в каркасе редактора, который показывает режим и путь.

Штатные сборка и запуск через Tauri подготавливают KaTeX, Mermaid и шрифты по
пути `/vendor/`. Просмотр формул и диаграмм работает без сети. Просмотрщик
загружает локальные картинки из каталога документа и его вложенных каталогов.
На одну картинку отведено 5 МиБ, на уникальные картинки в одном окне — 20 МиБ.
Для удалённых и недоступных картинок показывается текст. Ссылки на сайты и
почтовые адреса открываются в системном приложении, якоря — в текущем окне.

Конфиг Tauri собирает `NimMD.app` для macOS 14 и новее. `bundle.fileAssociations` генерирует
`CFBundleDocumentTypes`: Markdown — `Editor`, TOML/YAML/JSON/CSV/текст — `Viewer`.
Локальный macOS-бандл: `pnpm -C apps/desktop tauri build --debug --bundles app`.
CI на macOS проверяет сборку приложения, тесты, производительность ядра и линтеры.
