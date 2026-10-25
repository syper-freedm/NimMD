# Итог задачи: Реализовать рендер Markdown через comrak

- Issue: #41
- Итоговое состояние: реализовано, все ворота пройдены

## Реализованные требования
- FR-1…FR-6: `nimmd_core::render_fragment(body) -> String` в `crates/nimmd-core/src/render.rs`: comrak в режиме надмножества, `render.unsafe_ = false` и `render.escape = true`, подсветка syntect CSS-классами, `<pre class="mermaid">` (регистр info-строки не важен), алерты пяти типов, задачи, сноски, wikilinks, формулы.
- FR-2: общие опции `parse::options()` без `front_matter_delimiter`; `detect` разбирает тело после `split_frontmatter`.
- FR-7: `limit_nesting` — `\` перед `.`/`)` нумерованного маркера, ведущий отступ не шире 256 колонок (закрыт ARCH-005 из #40).
- insta-снапшоты в `crates/nimmd-core/tests/snapshots/`, тесты в `crates/nimmd-core/tests/render.rs`.

## Проверки
| Проверка | Результат | Свидетельство |
| --- | --- | --- |
| Архитектура | APPROVED | ФД и код-ревью; NOTICE ARCH-002 — лестница AC-10(в) 0,9 с в debug, тест держит порог 5 с |
| Безопасность | APPROVED | опасные схемы и сырой HTML проверены; `cargo deny check` ok; SEC-003 → #88 |
| Тестирование | APPROVED | `cargo test --workspace --all-targets` зелёный (render 17, text 14, detect 27); SC-001…SC-016 pass |

## Руководство пользователя
- Обновлённые страницы или причина отсутствия изменений: не требуется — руководства в репозитории ещё нет.

## Действия оператора
- нет

## GitHub
- Закрытые подзадачи: нет
- Follow-up: #88 — экранирующий fallback при ошибке подсветки syntect (SEC-003)
- Закрытие основной задачи: `afd close-issue` после squash-merge в `main`
