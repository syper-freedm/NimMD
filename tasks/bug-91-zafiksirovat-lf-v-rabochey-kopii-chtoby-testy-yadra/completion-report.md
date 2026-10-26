# Итог задачи: Зафиксировать LF в рабочей копии, чтобы тесты ядра проходили на Windows

- Issue: #91
- Итоговое состояние: все ворота пройдены, слито в `main`

## Реализованные требования
- FR-1: `.gitattributes` — `* text=auto eol=lf`, `*.packdump binary`.
- FR-2: индекс уже в LF (`git ls-files --eol`: нет `i/crlf`, `i/mixed`) — перенормализация содержимого не меняет.
- FR-3: `profile_matches_detect_on_fixtures` строит CRLF-вариант из `normalize_newlines(&src)`.

## Проверки
| Проверка | Результат | Свидетельство |
| --- | --- | --- |
| Архитектура | APPROVED | две правки строго по FR-1, FR-3 |
| Безопасность | APPROVED | `binary` сохраняет целостность `include_bytes!` дампа |
| Тестирование | APPROVED | клон `main` с `core.autocrlf=true`: 152 `w/crlf`, 4 падения (`class_contract`, `math_source_mode`, `bom_crlf_frontmatter_and_title`, `profile_matches_detect_on_fixtures`); с правкой — 153 `w/lf`, 40 passed; `just lint`, `just test` зелёные |

## Руководство пользователя
- Обновлённые страницы или причина отсутствия изменений: не требуется — пользовательского руководства нет.

## Действия оператора
- нет. Существующим клонам на Windows с CRLF в рабочей копии после обновления нужен `git add --renormalize .` или свежий checkout; окончательно подтвердит зелёная джоба `test (windows-latest)` на ближайшем пуше.

## GitHub
- Закрытые подзадачи: нет
- Закрытие основной задачи: `afd.py close-issue` после слияния в `main`
