# Итог задачи: Убрать <pre> внутри <p> у формул Source и развязать render и document

- Issue: #89
- Итоговое состояние: реализовано, все ворота пройдены

## Реализованные требования
- FR-1: display-формула `MathMode::Source` (и запасной вывод `MathML`) — `<code class="math-display">`, без `<pre>` внутри `<p>`.
- FR-2: класс `math-display` в списке «Только ядра» `packages/theme/CONTRACT.md` и правило `code.math-display` в `packages/theme/nimmd.css`.
- FR-3: `MathMode` перенесён в `crates/nimmd-core/src/render.rs`; `render.rs` больше не импортирует `document`; путь `nimmd_core::MathMode` прежний.

## Проверки
| Проверка | Результат | Свидетельство |
| --- | --- | --- |
| Архитектура | APPROVED | ФД и код-ревью без замечаний |
| Безопасность | APPROVED | ФД и код-ревью; SEC-001 (NOTICE): `cargo deny advisories` в песочнице не запустился, вне песочницы — ok |
| Тестирование | APPROVED | новый снапшот `document__math_source_mode.snap`, `cargo test --workspace --all-targets` зелёный, `just lint` exit 0, красный прогон до правки — 3 падения |

## Руководство пользователя
- Обновлённые страницы или причина отсутствия изменений: не требуется — руководства нет, публичный API не меняется.

## Действия оператора
- нет

## GitHub
- Закрытые подзадачи: нет
- Закрытие основной задачи: коммит в `main` с `Closes #89`
