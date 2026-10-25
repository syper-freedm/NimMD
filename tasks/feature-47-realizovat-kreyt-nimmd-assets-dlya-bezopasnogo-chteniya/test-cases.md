# Тестовые сценарии: Реализовать крейт nimmd-assets для безопасного чтения картинок

## Метаданные
- Issue: #47
- Стек: Rust, `cargo test`; интеграционные тесты — `crates/nimmd-assets/tests/*.rs`, временный каталог на тест (через `std::env::temp_dir()` + уникальное имя, если dev-зависимостей нет — FR-1).
- Ролей нет; изоляция данных — граница каталога документа (SC-002…SC-004).

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001 |
| AC-2 | SC-002 |
| AC-3 | SC-003, SC-004 |
| AC-4 | SC-005 |
| AC-5 | SC-006, SC-007 |
| AC-6 | SC-008, SC-009, SC-010 |

## Сценарии
### SC-001: Картинки рядом с документом и повторный путь
- **Покрывает:** AC-1
- **Уровень:** integration
- **Предусловия:** каталог из `Дано` AC-1, размеры `b.JPEG` и `c d.svg` различны и известны.
- **Шаги:** один `AssetBudget::new()`; `read_asset` для `img/a.png`, `./img/../img/a.png`, `img\a.png`, `b.JPEG`, `c%20d.svg`.
- **Ожидание:** каждый `Ok` равен байтам файла (`img\a.png` — байты `img/a.png` на любой ОС); `used()` = 10 + размер `b.JPEG` + размер `c d.svg`.
- **Результат выполнения:** pass

### SC-002: Отказы шагов 1–3 без ФС
- **Покрывает:** AC-2
- **Уровень:** integration
- **Предусловия:** `doc_path` = `/nonexistent-nimmd/doc.md`, новый бюджет.
- **Шаги:** таблица 15 `href` из AC-2, для каждого — ожидаемый вариант по FR-2.
- **Ожидание:** `Escape`: `../x.png`, `a/../../x.png`, `%2e%2e/x.png`, `%2E%2E%2Fx.png`, `..\x.png`; `Absolute`: `/etc/hosts`, `\x.png`; `Home`: `~/x.png`; `Scheme`: `file:///x.png`, `http://h/x.png`, `C:/x.png`; `BadEncoding`: `%zz.png`, `%ff.png`, `%00.png`, `""`. Ни одного `Io`; `used()` = 0.
- **Результат выполнения:** pass

### SC-003: Симлинк внутри каталога ведёт наружу (unix)
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** `#[cfg(unix)]`; `base/doc.md`, `outside/secret.png`, `base/link.png → ../outside/secret.png`.
- **Шаги:** `read_asset(base/doc.md, "link.png")`.
- **Ожидание:** `Err(Escape)`; `used()` = 0.
- **Результат выполнения:** pass

### SC-004: Документ-симлинк — база в каталоге цели (unix)
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** `#[cfg(unix)]`; `other/alias.md → base/doc.md`; `base/a.png` и `other/a.png` с разным содержимым.
- **Шаги:** `read_asset(other/alias.md, "a.png")`.
- **Ожидание:** байты `base/a.png`, не равны `other/a.png`.
- **Результат выполнения:** pass

### SC-005: Только картинки
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** `notes.txt`, `x.png.txt`, `noext`, каталог `dir.png`; под `#[cfg(unix)]` — `fake.png → notes.txt`.
- **Шаги:** `read_asset` для каждого и для отсутствующего `missing.png`.
- **Ожидание:** `NotImage` для всех пяти; `missing.png` → `Io`; `used()` = 0.
- **Результат выполнения:** pass

### SC-006: Лимит одного файла
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** `big.png` = `MAX_ASSET_BYTES + 1`, `ok.png` = `MAX_ASSET_BYTES` байт.
- **Шаги:** `read_asset` для `big.png`, затем `ok.png` тем же бюджетом.
- **Ожидание:** `big.png` → `TooLarge`; `ok.png` → `Ok`, длина `MAX_ASSET_BYTES`; `used()` = `MAX_ASSET_BYTES`.
- **Результат выполнения:** pass

### SC-007: Бюджет уникальных файлов
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** `p1.png`…`p5.png` по `4 × 1024 × 1024 + 1` байт, новый бюджет.
- **Шаги:** `read_asset` `p1`…`p5` по порядку, затем снова `p1`.
- **Ожидание:** `p1`…`p4` — `Ok`; `p5` → `BudgetExceeded`; `used()` = `4 × (4 × 1024 × 1024 + 1)` после `p5` и после повторного `p1`; повторный `p1` — байты файла.
- **Результат выполнения:** pass

### SC-008: Нет unsafe/unwrap/expect
- **Покрывает:** AC-6
- **Уровень:** static
- **Предусловия:** дерево после правки.
- **Шаги:** `git grep -n "unsafe\|unwrap()\|expect(" crates/nimmd-assets/src`.
- **Ожидание:** единственное совпадение — строка `#![forbid(unsafe_code)]`.
- **Результат выполнения:** pass

### SC-009: Display и Error у AssetError
- **Покрывает:** AC-6
- **Уровень:** unit
- **Предусловия:** все 9 вариантов `AssetError`.
- **Шаги:** для каждого `to_string()`; приведение к `&dyn std::error::Error`.
- **Ожидание:** строки непустые, попарно различны, содержат кириллицу; приведение компилируется.
- **Результат выполнения:** pass

### SC-010: Крейт в workspace, проверки зелёные
- **Покрывает:** AC-6
- **Уровень:** static
- **Предусловия:** `crates/nimmd-assets` в `members` корневого `Cargo.toml`.
- **Шаги:** `cargo test -p nimmd-assets`; `cargo test --workspace`; `just lint` (сеть для `vendor`/`deny`).
- **Ожидание:** все команды с кодом 0; `cargo machete` без находок.
- **Результат выполнения:** pass
