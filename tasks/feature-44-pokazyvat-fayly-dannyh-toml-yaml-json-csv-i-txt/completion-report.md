# Итог задачи: Показывать файлы данных TOML, YAML, JSON, CSV и TXT

- Issue: #44
- Итоговое состояние: реализовано, все ворота пройдены

## Реализованные требования
- FR-1–FR-3: `FileKind` с `from_extension`, `RenderOptions.kind`, `Rendered.highlight_limited`; документ данных `<main class="nimmd nimmd-data" data-kind="…"><pre class="data">` без профиля, frontmatter и скриптов.
- FR-4–FR-5: свой дамп syntect (`crates/nimmd-core/assets/syntaxes.packdump`, `just syntaxes`) — стандартный набор + TOML из sublimehq/Packages@72065294; общий для файлов данных, блоков кода (` ```toml ` теперь подсвечен) и frontmatter; без паникующих методов syntect.
- FR-6: подсветка первых 2 МБ, хвост простым текстом, баннер.
- FR-7–FR-8: CSV — свой токенизатор RFC 4180, `csv::detect_delimiter`, раскраска `c0`…`c7`/`d` без потери текста.
- FR-9: `packages/theme/data.css`, контраст ≥ 4.5:1 в обеих темах.
- FR-10–FR-11: `highlight(lang, code) -> Vec<Span>` в UTF-16; `text::decode_lossy`.

## Проверки
| Проверка | Результат | Свидетельство |
| --- | --- | --- |
| Архитектура | APPROVED | ФД (NOTICE ARCH-001/002 учтены в ФД) и код-ревью без замечаний |
| Безопасность | APPROVED | ФД и код-ревью; `deny.toml`: RUSTSEC-2024-0320 (yaml-rust) исключён — только dev-сборка дампа |
| Тестирование | APPROVED | 131 тест, 0 пропущенных; дифференциальные тесты CSV и спанов; мутации; снапшоты Markdown без изменений; `just lint` зелёный |

## Руководство пользователя
- Обновлённые страницы или причина отсутствия изменений: не требуется — руководства нет, API описывают doc-комментарии (#46).

## Действия оператора
- нет

## GitHub
- Закрытые подзадачи: нет
- Закрытие основной задачи: коммит в `main` с `Closes #44`
