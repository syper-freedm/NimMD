# Функциональный дизайн: Зафиксировать LF в рабочей копии, чтобы тесты ядра проходили на Windows

## Метаданные [@for: all]
- Issue: #91
- Size: S
- Владелец: Системный аналитик

## Цель [@for: all]
- Рабочая копия на любой ОС и при любом `core.autocrlf` получает текстовые файлы с LF: встроенные `include_str!` ресурсы (CSS, JS темы) одинаковы независимо от ОС сборки, `test (windows-latest)` в CI зелёный.
- Причина: в репозитории нет `.gitattributes`, `actions/checkout` на Windows (`core.autocrlf=true`) кладёт CRLF; падают `bom_crlf_frontmatter_and_title`, `class_contract`, `profile_matches_detect_on_fixtures` в `crates/nimmd-core/tests/document.rs`. В индексе все текстовые файлы уже с LF (`git ls-files --eol`: 152 `i/lf`, 1 бинарный — `crates/nimmd-core/assets/syntaxes.packdump`), перенормализация содержимого не меняет.

## Границы [@for: all]
- Входит: FR-1…FR-3.
- Не входит: нормализация переносов внутри ядра при `include_str!` (решается на уровне git); изменение поведения ядра на CRLF-входе; CI-матрица.

## Целевое поведение [@for: all]
- Клон с `core.autocrlf=true` (как Git for Windows по умолчанию) содержит LF во всех текстовых файлах, и `cargo test --workspace` в нём зелёный.
- Тест CRLF-варианта фикстур не зависит от того, какие переносы у самой фикстуры на диске.

## Требования [@for: all]
- **FR-1:** В корне репозитория `.gitattributes`: `* text=auto eol=lf` и `*.packdump binary` (бинарный дамп syntect не нормализуется, даже если эвристика git ошибётся), у каждого правила — комментарий-причина (ссылка на спек 9, переносимость).
- **FR-2:** `git add --renormalize .` после FR-1 не даёт изменений содержимого (подтверждение, что индекс уже в LF); если даёт — изменённые файлы входят в коммит задачи.
- **FR-3:** `profile_matches_detect_on_fixtures` в `crates/nimmd-core/tests/document.rs` строит CRLF-вариант из нормализованного текста (`normalize_newlines(&src).replace('\n', "\r\n")`), а не из сырого: фикстура с CRLF на диске больше не даёт `\r\r\n`. Ожидаемый профиль и остальные проверки теста не меняются.

## Данные, права, ошибки [@for: backend, security, qa]
- Код ядра не меняется; данных и прав нет. `binary` у дампа исключает порчу встроенного `include_bytes!` преобразованием переносов.

## Карта реализации [@for: backend, frontend, qa]
- Backend: `.gitattributes` (новый, корень; грант `--allow-path`); `crates/nimmd-core/tests/document.rs` — правка существующего теста по FR-3 (укрепление, не новый тест).
- Frontend: не затрагивается.
- Новые тесты: нет (правка конфигурации git и одного существующего теста; проверка — клоном с `core.autocrlf=true`, AC-2).

## Критерии приёмки [@for: all]
### AC-1: Атрибуты и индекс
- **Дано:** ветка после FR-1, FR-2.
- **Действие:** `git check-attr text eol -- packages/theme/nimmd.css packages/theme/CONTRACT.md crates/nimmd-core/tests/fixtures/gfm/table_tasks.md`; `git check-attr binary -- crates/nimmd-core/assets/syntaxes.packdump`; `git ls-files --eol`.
- **Ожидание:** у текстовых файлов `text: auto`, `eol: lf`; у дампа `binary: set`; в `git ls-files --eol` нет `i/crlf` и нет `i/mixed`, дамп — `i/-text`.

### AC-2: Клон с autocrlf=true
- **Дано:** закоммиченная ветка; для сравнения — `main` до правки.
- **Действие:** `git clone -c core.autocrlf=true` локального репозитория (ветки и `main`) во временный каталог; в клоне `git ls-files --eol` и `cargo test -p nimmd-core --test document`.
- **Ожидание:** клон `main` — рабочие файлы `w/crlf` и 3 упавших теста (воспроизведение дефекта); клон ветки — все рабочие текстовые файлы `w/lf`, `test result: ok`.

### AC-3: Тест CRLF устойчив к фикстуре
- **Дано:** `crates/nimmd-core/tests/document.rs` после FR-3; временная копия фикстуры `gfm/table_tasks.md`, переведённая в CRLF в рабочей копии (без коммита).
- **Действие:** `cargo test -p nimmd-core --test document profile_matches_detect_on_fixtures`; затем вернуть фикстуру (`git checkout -- <фикстура>`).
- **Ожидание:** тест зелёный и с CRLF-фикстурой; после возврата `git status --short` не содержит фикстур.

### AC-4: Проверки проекта
- **Дано:** ветка после FR-1…FR-3.
- **Действие:** `just lint`, `just test`.
- **Ожидание:** оба зелёные.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского руководства нет.
