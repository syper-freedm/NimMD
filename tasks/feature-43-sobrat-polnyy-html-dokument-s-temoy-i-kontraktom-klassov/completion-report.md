# Итог задачи: Собрать полный HTML-документ с темой и контрактом классов

- Issue: #43
- Итоговое состояние: реализовано, все ворота пройдены

## Реализованные требования
- FR-1…FR-4: `nimmd_core::render(src, &RenderOptions) -> Rendered` в `crates/nimmd-core/src/document.rs`; `Theme`, `MathMode`, `AssetMode`, `RenderOptions` (`theme`, `assets`, `math`, `remote_images`, `not_utf8`, `max_bytes`, `profile`); один разбор comrak даёт фрагмент и флаги профиля; `body` — подстрока `html`; `title` из первого заголовка уровня 1.
- FR-5: frontmatter — `<pre class="frontmatter">` с подсветкой YAML.
- FR-6: удалённые картинки (`http`/`https` при любом числе `/`, `//`) — `<span class="remote-image">` с адресом.
- FR-7: формулы `Katex` / `MathML` (`latex2mathml` с пределом рекурсии и экранированием) / `Source`.
- FR-8: KaTeX и mermaid.js по профилю (`None` / `Link` / `Inline`), инициализация в `packages/theme/katex-init.js` и `packages/theme/mermaid-init.js` (`trust: false`, `securityLevel: 'strict'`).
- FR-9, FR-10: баннеры `truncated` и `not-utf8`; `packages/theme/nimmd.css`, `packages/theme/highlight.css`, `packages/theme/CONTRACT.md`.

## Проверки
| Проверка | Результат | Свидетельство |
| --- | --- | --- |
| Архитектура | APPROVED | ФД и код-ревью; NOTICE ARCH-004/005 → #89 |
| Безопасность | APPROVED | SEC-001…003 закрыты в ФД и коде; `cargo deny check` ok; semgrep `.semgrep.yml` чист |
| Тестирование | APPROVED | `cargo test --workspace --all-targets` — 95 passed; `tests/document.rs` — 30; SC-001…SC-011 pass |

## Руководство пользователя
- Обновлённые страницы или причина отсутствия изменений: не требуется — руководства в репозитории ещё нет.

## Действия оператора
- нет

## GitHub
- Закрытые подзадачи: нет
- Follow-up: #89 — `<pre>` внутри `<p>` у формул `Source` и взаимная зависимость `render`/`document` (ARCH-004, ARCH-005)
- Закрытие основной задачи: `afd close-issue` после squash-merge в `main`
