# Тестовые сценарии: Зафиксировать LF в рабочей копии, чтобы тесты ядра проходили на Windows

## Метаданные
- Issue: #91
- Стек: git (static), `cargo test` (integration, `crates/nimmd-core/tests/document.rs`), `just lint` / `just test`. Контроля доступа нет — сценарии объектного доступа неприменимы.
- Клоны и логи — только в `tmp_dir` фазы; `CARGO_TARGET_DIR` клонов — там же.

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001, SC-002 |
| AC-2 | SC-003, SC-004 |
| AC-3 | SC-005 |
| AC-4 | SC-006, SC-007 |

## Сценарии
### SC-001: Атрибуты текстовых файлов и дампа
- **Покрывает:** AC-1
- **Уровень:** static
- **Предусловия:** `.gitattributes` в корне (FR-1).
- **Шаги:** `git check-attr text eol -- packages/theme/nimmd.css packages/theme/CONTRACT.md crates/nimmd-core/tests/fixtures/gfm/table_tasks.md`; `git check-attr binary text -- crates/nimmd-core/assets/syntaxes.packdump`.
- **Ожидание:** у трёх файлов `text: auto` и `eol: lf`; у дампа `binary: set`, `text: unset`.
- **Результат выполнения:** pass

### SC-002: Индекс в LF
- **Покрывает:** AC-1
- **Уровень:** static
- **Предусловия:** FR-1, FR-2 применены (`git add` выполнен).
- **Шаги:** `git ls-files --eol | grep -E 'i/(crlf|mixed)'`; `git ls-files --eol -- crates/nimmd-core/assets/syntaxes.packdump`.
- **Ожидание:** первая команда пуста; дамп — `i/-text`.
- **Результат выполнения:** pass

### SC-003: Воспроизведение дефекта на main
- **Покрывает:** AC-2
- **Уровень:** E2E
- **Предусловия:** локальный `main` до правки.
- **Шаги:** `git clone -c core.autocrlf=true -b main <repo> <tmp>/main`; в клоне `git ls-files --eol`; `cargo test -p nimmd-core --test document`.
- **Ожидание:** рабочие текстовые файлы `w/crlf`; тест-ран красный, 3 failed (в т. ч. `profile_matches_detect_on_fixtures`).
- **Результат выполнения:** pass: 152 `w/crlf`, 4 failed (база, QA-001)

### SC-004: Клон ветки с autocrlf=true
- **Покрывает:** AC-2
- **Уровень:** E2E
- **Предусловия:** ветка закоммичена; если нет — в клоне `main` скопировать `.gitattributes`, `document.rs` и файлы FR-2 из рабочей копии, закоммитить, затем `git rm -rq --cached . && git reset -q --hard`.
- **Шаги:** `git clone -c core.autocrlf=true -b <ветка> <repo> <tmp>/fix`; `git ls-files --eol | grep -E 'w/(crlf|mixed)'`; `cargo test -p nimmd-core --test document`.
- **Ожидание:** grep пуст (текстовые файлы `w/lf`, файлы без переноса — `w/none`); `test result: ok`.
- **Результат выполнения:** pass: 153 `w/lf`, 40 passed

### SC-005: CRLF-фикстура не ломает тест профиля
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** FR-3 в `document.rs`.
- **Шаги:** перевести `crates/nimmd-core/tests/fixtures/gfm/table_tasks.md` в CRLF (`perl -pi -e 's/\n/\r\n/'`), проверить `file` → CRLF; `cargo test -p nimmd-core --test document profile_matches_detect_on_fixtures`; `git checkout -- <фикстура>`; `git status --short`.
- **Ожидание:** `test result: ok. 1 passed`; в `git status --short` нет путей `tests/fixtures/`.
- **Результат выполнения:** pass

### SC-006: Линтеры проекта
- **Покрывает:** AC-4
- **Уровень:** static
- **Предусловия:** FR-1…FR-3; сеть для `deny`/`vendor`.
- **Шаги:** `just lint`.
- **Ожидание:** код выхода 0.
- **Результат выполнения:** pass

### SC-007: Тесты проекта
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** FR-1…FR-3.
- **Шаги:** `just test`.
- **Ожидание:** код выхода 0, во всех `test result` — `ok`.
- **Результат выполнения:** pass
