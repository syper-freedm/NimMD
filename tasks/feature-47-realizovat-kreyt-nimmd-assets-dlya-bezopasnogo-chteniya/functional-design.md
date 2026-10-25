# Функциональный дизайн: Реализовать крейт nimmd-assets для безопасного чтения картинок

## Метаданные [@for: all]
- Issue: #47
- Size: M
- Владелец: Системный аналитик

## Цель [@for: all]
- Превью чужого Markdown не может прочитать файл вне каталога документа, не-картинку или слишком много данных: проверка пути картинки — одна функция крейта `crates/nimmd-assets`, которую позже вызовут FFI Quick Look (#49) и Tauri-обработчик `nimmd-asset://` (спек 5.3 «Безопасность превью», Б4, Р15, К-Р15, К23; спек 8).

## Границы [@for: all]
- Входит: крейт `nimmd-assets` (библиотека, только `std`): `read_asset`, `AssetBudget`, `AssetError`, лимиты; член workspace; тесты раздела 8 спека.
- Не входит: FFI `nimmd_read_asset` (#49), обработчик `nimmd-asset://` и CSP (Tauri), `http(s)`-картинки (их отсекает ядро, `remote_images = false`), MIME-тип ответа (хост знает расширение), кэширование содержимого.

## Целевое поведение [@for: all]
- Хост передаёт путь документа, `href` картинки из Markdown и состояние лимита на один рендер (окно редактора); получает байты файла или отказ. Любой отказ хост показывает плейсхолдером, причина нужна только тестам и логам.
- `![](img/a.png)` рядом с документом — байты файла; `![](../../.ssh/id_ed25519)`, `![](%2e%2e/x.png)`, `![](/etc/hosts)`, `![](~/x.png)`, `![](file:///x.png)`, `![](notes.txt)` — отказ; та же картинка повторно лимит не расходует.

## Требования [@for: all]
- **FR-1:** Крейт `crates/nimmd-assets` (`[lib] name = "nimmd_assets"`, `edition`/`license`/`rust-version` и `[lints]` из workspace, без зависимостей), добавлен в `members` корневого `Cargo.toml`; `#![forbid(unsafe_code)]`; `unwrap`/`expect`/`panic!` в рабочем коде нет. Код в `crates/nimmd-assets/src/lib.rs`.
- **FR-2:** `pub fn read_asset(doc_path: &Path, href: &str, budget: &mut AssetBudget) -> Result<Vec<u8>, AssetError>`. Порядок проверок строго такой; шаги 1–3 не обращаются к ФС:
  1. URL-decode `href` один раз: `%XX` (две шестнадцатеричные цифры, любой регистр) → байт; `%` без двух цифр или результат не UTF-8 → `BadEncoding`. Пустой результат или байт NUL → `BadEncoding`.
  2. Отказ `Scheme` — результат начинается со схемы `[A-Za-z][A-Za-z0-9+.-]*:` (`file:`, `http:`, `C:`); `Absolute` — начинается с `/` или `\`; `Home` — начинается с `~`.
  3. Лексическая проверка: компоненты пути по разделителям `/` и `\` (обратная косая — разделитель на любой ОС); `.` и пустые пропускаются; `..` снимает предыдущий компонент, `..` при пустом стеке → `Escape`.
  4. База — родительский каталог `canonicalize(doc_path)` (документ-симлинк → каталог цели); ошибка ФС или нет родителя → `Io`. Кандидат — `canonicalize` базы с добавленными по одному компонентами шага 3 (не исходной строкой: `img\a.png` открывает `img/a.png` на любой ОС), ошибка (нет файла) → `Io`; кандидат не начинается с базы (покомпонентно, `Path::starts_with`) → `Escape` (симлинк внутри каталога, ведущий наружу).
  5. Расширение канонического кандидата без учёта регистра (ASCII) ∈ `png jpg jpeg gif webp svg`, иначе `NotImage`; не обычный файл (каталог `x.png`) → `NotImage`.
  6. Размер: `metadata.len() > MAX_ASSET_BYTES` → `TooLarge` без чтения; чтение через `take(MAX_ASSET_BYTES + 1)`, прочитано больше лимита (файл вырос) → `TooLarge`; ошибка чтения → `Io`.
  7. Лимит (FR-3); успех — байты файла.
- **FR-3:** `pub const MAX_ASSET_BYTES: u64 = 5 * 1024 * 1024`, `pub const MAX_BUDGET_BYTES: u64 = 20 * 1024 * 1024`. `#[derive(Debug, Default)] pub struct AssetBudget` (поля приватные: сумма байт и множество канонических путей, учтённых в сумме) с `AssetBudget::new()` и `pub fn used(&self) -> u64`. Путь, уже учтённый в бюджете, читается снова и лимит не расходует. Новый путь: `used + размер > MAX_BUDGET_BYTES` → `BudgetExceeded` до чтения, бюджет не меняется; после успешного чтения `used += прочитано`, путь запоминается. Любой отказ бюджет не меняет. Сброс — новый `AssetBudget` (решает хост).
- **FR-4:** `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum AssetError { BadEncoding, Scheme, Absolute, Home, Escape, NotImage, TooLarge, BudgetExceeded, Io }` с doc-комментарием у каждого варианта; `impl Display` (русский текст) и `impl std::error::Error`.

## Данные, права, ошибки [@for: backend, security, qa]
- `doc_path` — доверенный путь от хоста; `href` — недоверенный текст документа. Ни один `href` не даёт прочитать файл вне канонического каталога документа, не-картинку, файл больше 5 МБ или больше 20 МБ уникальных файлов за один бюджет.
- Шаги 1–3 отказывают без системных вызовов: на несуществующем `doc_path` они возвращают свою ошибку, а не `Io`.
- Гонка «проверили — подменили» (TOCTOU) снижается чтением канонического пути с ограничением `take`; полная защита от подмены симлинка между `canonicalize` и `open` — вне объёма (хост читает файлы пользователя, не сервер).
- Платформенного кода в крейте нет; в тестах создание симлинка — `std::os::unix::fs::symlink` под `#[cfg(unix)]` (на Windows без прав администратора симлинк не создать), тесты симлинков проходят на macOS и Linux CI.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `crates/nimmd-assets/Cargo.toml`, `crates/nimmd-assets/src/lib.rs` (новые), `Cargo.toml` (`members`), `Cargo.lock`.
- Frontend: не затрагивается.
- Новые тесты: `crates/nimmd-assets/tests/assets.rs` — каталоги во временной папке `env!("CARGO_TARGET_TMPDIR")` (без новых зависимостей), по одному тесту на каждый AC; это единственный набор тестов правила (спек 8).

## Критерии приёмки [@for: all]
### AC-1: Картинка рядом с документом
- **Дано:** `docs/note.md`, `docs/img/a.png` (10 байт), `docs/b.JPEG`, `docs/c d.svg`.
- **Действие:** `read_asset(note.md, href, &mut budget)` для `img/a.png`, `./img/../img/a.png`, `img\a.png`, `b.JPEG`, `c%20d.svg`.
- **Ожидание:** байты соответствующих файлов; `budget.used()` = сумма размеров трёх разных файлов (повторный `a.png` не учтён).

### AC-2: Отказы без обращения к ФС
- **Дано:** `doc_path` = несуществующий `/nonexistent-nimmd/doc.md`.
- **Действие:** `read_asset` для `../x.png`, `a/../../x.png`, `%2e%2e/x.png`, `%2E%2E%2Fx.png`, `..\x.png`, `/etc/hosts`, `\x.png`, `~/x.png`, `file:///x.png`, `http://h/x.png`, `C:/x.png`, `%zz.png`, `%ff.png`, `%00.png`, `""`.
- **Ожидание:** `Escape` ×5, `Absolute` ×2, `Home`, `Scheme` ×3, `BadEncoding` ×4 — ни одного `Io`; бюджет не изменился.

### AC-3: Выход наружу через симлинк и документ-симлинк (unix)
- **Дано:** `base/doc.md`, `outside/secret.png`, симлинк `base/link.png → ../outside/secret.png`; симлинк `other/alias.md → base/doc.md`, `base/a.png`, `other/a.png`.
- **Действие:** `read_asset(base/doc.md, "link.png")`; `read_asset(other/alias.md, "a.png")`.
- **Ожидание:** `Escape`; байты `base/a.png` (база — каталог цели), а не `other/a.png`.

### AC-4: Только картинки
- **Дано:** `notes.txt`, `x.png.txt`, `noext`, каталог `dir.png`, симлинк `fake.png → notes.txt` (unix).
- **Действие:** `read_asset` для каждого.
- **Ожидание:** `NotImage` для всех; несуществующий `missing.png` → `Io`.

### AC-5: Лимит файла и бюджета
- **Дано:** `big.png` = `MAX_ASSET_BYTES + 1` байт, `ok.png` = `MAX_ASSET_BYTES` байт; пять разных файлов `p1.png`…`p5.png` по `4 × 1024 × 1024 + 1` байт.
- **Действие:** `read_asset` для `big.png`, `ok.png`; затем новым бюджетом `p1`…`p5` по порядку и снова `p1`.
- **Ожидание:** `big.png` → `TooLarge`, `ok.png` — `MAX_ASSET_BYTES` байт; `p1`…`p5`: пятый — `BudgetExceeded` и `used()` = `4 × (4 × 1024 × 1024 + 1)`; повторный `p1` — байты файла, `used()` не изменился.

### AC-6: Крейт и проверки
- **Дано:** дерево после правки.
- **Действие:** `git grep -n "unsafe\|unwrap()\|expect(" crates/nimmd-assets/src`; `cargo test -p nimmd-assets`; `just lint`; `cargo test --workspace`.
- **Ожидание:** только `#![forbid(unsafe_code)]`; `Display` каждого варианта `AssetError` — непустой текст на русском, у вариантов тексты разные; проверки зелёные, `cargo machete` не находит неиспользуемых зависимостей.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства нет; правила описывают doc-комментарии крейта.
