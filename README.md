# NimMD

Быстрый лёгкий просмотрщик и редактор Markdown. По пробелу в Finder файл сразу показывается
отформатированным: NimMD сам определяет диалект Markdown и расширения. Одно Rust-ядро
обслуживает и приложение, и Quick Look, поэтому документ выглядит одинаково.

## Структура

- `crates/nimmd-core` — ядро: Markdown и файлы данных в HTML ([README](crates/nimmd-core/README.md)).
- `crates/nimmd-assets` — безопасное чтение картинок, на которые ссылается документ.
- `packages/theme` — CSS тем и подсветки, скрипты инициализации KaTeX и Mermaid.
- `packages/vendor` — сборка KaTeX и Mermaid без сети в рантайме.

## Команды

```sh
just test     # тесты Rust и скриптов
just lint     # форматирование, clippy, semgrep, лицензии, секреты, CI
just bench    # замеры рендера (criterion)
just vendor   # собрать KaTeX и Mermaid в packages/vendor/dist
```
