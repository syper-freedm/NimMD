# Заключения проверок: Зафиксировать LF в рабочей копии, чтобы тесты ядра проходили на Windows

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED

### Специалист по ИБ
- Статус: APPROVED

### Тестировщик
- Статус: APPROVED
- ID: QA-001
- Уровень: NOTICE
- Ссылка: AC-2
- Проблема: два поведения в одном AC (воспроизведение на `main` и результат ветки); точное «3 упавших теста» — свойство базы, а не правки; «закоммиченная ветка» на этапе тестирования может не существовать.
- Требуемый результат: считать клон `main` базой (≥1 падение, в т. ч. `profile_matches_detect_on_fixtures`); без коммита — клон `main` + перенос правки (SC-004).
- ID: QA-002
- Уровень: NOTICE
- Ссылка: AC-2
- Проблема: «все рабочие текстовые файлы `w/lf`» — файлы без переноса дают `w/none`.
- Требуемый результат: проверять отсутствие `w/crlf` и `w/mixed`.

### UX-ревьюер
- Статус: PENDING

## Проверка реализации

### Качество кода
- Статус: APPROVED
- Соответствие FR/AC: CONFIRMED
- Несвязанные изменения: NONE

### Безопасность кода
- Статус: APPROVED
- Проверены меры ИБ: CONFIRMED
- Правка: `.gitattributes` (`* text=auto eol=lf`, `*.packdump binary`) и тест `document.rs`; код ядра, точки входа, данные и зависимости не меняются — authz/scope/injection неприменимы. `syntaxes.packdump` — `i/-text`, `binary: set`: `include_bytes!` не портится; других `-text` файлов в индексе нет.
- gitleaks по файлам правки: no leaks found. Semgrep (`.semgrep.yml`, p/rust, p/secrets) по `document.rs`: 0 находок; производственных исходников в правке нет.
- `cargo deny check` не отработал в песочнице (lock базы RustSec read-only); Cargo.lock правкой не затронут — новых уязвимостей быть не может.

### Выполнение тестов
- Статус: APPROVED
- suite-run записей нет — прогнал сам: `cargo test --workspace --all-targets`, `just test` (40/40 в `document`, node 8/8), `just lint` — exit 0.
- SC-003/SC-004: клоны с `core.autocrlf=true` в `tmp_dir`; ветка не закоммичена — правка закоммичена в клон `main`, затем он склонирован заново. `main`: 4 failed (вкл. `profile_matches_detect_on_fixtures`); правка: `w/crlf|w/mixed` нет, 40 passed.
- SC-005: CRLF-фикстура `table_tasks.md` — 1 passed; после `git checkout` фикстур в `git status` нет.

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
