# NimMD

Быстрый лёгкий просмотрщик и редактор Markdown для macOS 14+. По пробелу в Finder файл сразу показывается
отформатированным: NimMD сам определяет диалект Markdown и расширения. Одно Rust-ядро
обслуживает и приложение, и Quick Look, поэтому документ выглядит одинаково.

## Установка

1. Откройте доверенный `NimMD.dmg` для вашего Mac и перетащите NimMD в папку
   «Программы». Установочный образ можно [собрать из исходников](#сборка-из-исходников).
2. Извлеките образ и откройте NimMD из папки «Программы».
3. Сборка использует ad-hoc подпись. Если macOS блокирует первый запуск
   доверенной копии, откройте «Системные настройки → Конфиденциальность и
   безопасность», нажмите «Все равно открыть» и подтвердите запуск. [Инструкция Apple по Gatekeeper](https://support.apple.com/ru-ru/102445).

Откройте документ через «Открыть в программе → NimMD» или перетащите его на окно
приложения. Для быстрого просмотра выделите файл в Finder и нажмите пробел.
Уведомления о сторонних компонентах доступны в меню «NimMD → О NimMD → Лицензии».

### NimMD по умолчанию для .md

Выделите файл `.md` в Finder и нажмите ⌘I («Свойства»). В разделе «Открывать
в программе» выберите NimMD, нажмите «Изменить все» и подтвердите выбор для
всех файлов этого типа. После этого двойной щелчок по Markdown и кнопка
«Открыть в NimMD» в Quick Look открывают файл в редакторе.

## Сборка из исходников

Понадобятся полный Xcode, стабильный Rust через rustup, `just`, Node.js 26
и pnpm 12.21.0. Для упаковки установите cbindgen и cargo-about.

```sh
cargo install --locked cbindgen --version 0.29.4
cargo install --locked cargo-about --version 0.9.3
```

Для подписи архива передайте `TAURI_SIGNING_PRIVATE_KEY_PATH` с абсолютным
путём к закрытому ключу или `TAURI_SIGNING_PRIVATE_KEY` с самим ключом.
Для зашифрованного ключа задайте `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Для
локальной сборки и проверки создайте отдельный ключ вне репозитория.

```sh
just desktop-deps
signing_dir=$(mktemp -d)
(umask 077
 pnpm -C apps/desktop tauri signer generate --ci --password '' \
   --write-keys "$signing_dir/updater.key" > "$signing_dir/generate.log")
env -u TAURI_SIGNING_PRIVATE_KEY \
  TAURI_SIGNING_PRIVATE_KEY_PATH="$signing_dir/updater.key" \
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD='' just build-macos
```

Для проверки подписи используйте открытый ключ `$signing_dir/updater.key.pub`.
Храните закрытый ключ и `generate.log` вне репозитория.

Готовые файлы появляются в `target/macos-release/<rust-target>/`
(`aarch64-apple-darwin` для Apple Silicon или `x86_64-apple-darwin` для Intel).
Каталог содержит `NimMD.app` с Quick Look, `NimMD.app.tar.gz` и его подпись
`NimMD.app.tar.gz.sig`, а также установочный образ `NimMD.dmg`.
Порядок проверки описан в [чек-листе выпуска](docs/manual-checks.md#приёмка-выпуска-macos).

## Структура

- `crates/nimmd-core` — ядро: Markdown и файлы данных в HTML ([README](crates/nimmd-core/README.md)).
- `crates/nimmd-assets` — безопасное чтение картинок, на которые ссылается документ.
- `crates/nimmd-ffi` — интерфейс Rust-ядра для расширения Quick Look.
- `apps/desktop` — приложение Tauri 2 с режимами просмотра и редактирования.
- `apps/macos-quicklook` — расширение Quick Look на Swift.
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

Для приложения закреплены Tauri 2.12.7, tauri-build 2.7.6 и Tauri CLI 2.12.4.
Для `nimmd-desktop` объявлен MSRV 1.90, для workspace и ядра — 1.85.
Проект использует TypeScript 6.0.3, совместимый с текущим typescript-eslint.

```sh
just desktop-deps
pnpm -C apps/desktop tauri dev -- -- file.md
pnpm -C apps/desktop tauri dev -- -- --view file.md
pnpm -C apps/desktop tauri build --debug --no-bundle
```

Каждый файл получает отдельное окно с именем файла в заголовке.
Повторное открытие поднимает уже открытое окно и сохраняет его режим.

После сборки `target/debug/nimmd file.md` выбирает редактирование,
`target/debug/nimmd --view file.md` — просмотр. Для имени с дефисом используйте
`nimmd -- -file.md`. CLI принимает один файл. Без аргументов открывается или
активируется пустое окно. В том же сеансе macOS повторный вызов передаёт запрос
работающему NimMD. Относительный путь разрешается в каталоге вызывающего процесса.
Если CLI не дождался ответа после отправки запроса, открытие ещё может завершиться.
Проверьте окна NimMD перед повтором команды.

Режим просмотра форматирует Markdown (`.md`, `.markdown`, `.mdown`, `.mkd`), TOML,
YAML (`.yaml`, `.yml`), JSON и CSV и показывает `.txt` как обычный текст. Неизвестное
расширение обрабатывается как Markdown. Markdown без флага `--view`, из Finder
или при перетаскивании открывается в редакторе. Изменения сохраняются автоматически,
для немедленного сохранения нажмите ⌘S.

Штатные сборка и запуск через Tauri подготавливают KaTeX, Mermaid и шрифты по
пути `/vendor/`. Просмотр формул и диаграмм работает без сети. Просмотрщик
загружает локальные картинки из каталога документа и его вложенных каталогов.
На одну картинку отведено 5 МиБ, на уникальные картинки в одном окне — 20 МиБ.
Для удалённых и недоступных картинок показывается текст. Ссылки на сайты и
почтовые адреса открываются в системном приложении, якоря — в текущем окне.

Конфиг Tauri собирает `NimMD.app` для macOS 14 и новее. `bundle.fileAssociations` генерирует
`CFBundleDocumentTypes`: Markdown — `Editor`, TOML/YAML/JSON/CSV/текст — `Viewer`.
CI на macOS проверяет сборку приложения, тесты, производительность ядра и линтеры.
