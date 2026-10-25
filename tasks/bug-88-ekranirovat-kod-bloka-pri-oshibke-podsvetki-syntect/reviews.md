# Заключения проверок: Экранировать код блока при ошибке подсветки syntect

## Проверка функционального дизайна

### Архитектор решения
- Статус: APPROVED
- ID: ARCH-001
- Уровень: NOTICE
- Ссылка: FR-3 / AC-4
- Проблема: после замены адаптера фича `syntect-fancy` у `comrak` в `[workspace.dependencies]` нужна только для неиспользуемого `SyntectAdapter` (трейт `comrak::adapters` от фичи не зависит) и тянет в syntect `default-themes`.
- Требуемый результат: на усмотрение реализатора убрать `features = ["syntect-fancy"]` у comrak — syntect останется одним экземпляром через прямую зависимость; оставить тоже допустимо.

### Специалист по ИБ
- Статус: APPROVED

- ID: SEC-001
- Уровень: NOTICE
- Ссылка: FR-1 / AC-3
- Проблема: API3 (вывод недоверенного свойства). `lang` из info-строки документа попадает в `class` у `<code>`; ФД говорит «атрибуты comrak», но не требует экранирования значения. Сейчас comrak пишет его через `html::write_opening_tag` (экранирует); ручной `write!` открыл бы XSS через `"`.
- Требуемый результат: `write_code_tag`/`write_pre_tag` — только через `comrak::html::write_opening_tag`; в AC-3 добавить блок с `` ```x"onmouseover=y `` и проверить, что `"` в атрибуте экранирован.

### Тестировщик
- Статус: APPROVED
- ID: QA-001
- Уровень: NOTICE
- Ссылка: AC-1, AC-2, FR-2
- Проблема: функция FR-2 закрыта внутри `render.rs`, а тесты ядра по соглашению — интеграционные в `tests/`, откуда её не вызвать.
- Требуемый результат: проверять FR-2 unit-тестом `#[cfg(test)]` в `render.rs` (так в SC-001/SC-002) или явно открыть её как `#[doc(hidden)] pub`.
- ID: QA-002
- Уровень: NOTICE
- Ссылка: AC-4
- Проблема: в одном AC четыре проверки (grep, `cargo tree`, `just lint`, `cargo test`); `just lint` требует сети.
- Требуемый результат: разделено на SC-006 (static) и SC-007; падение SC-007 из-за окружения — это ENV, а не дефект.

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
- Вывод: текст в `ClassedHTMLGenerator` экранирует `Escape`; при `Err` частичный HTML отбрасывается, код идёт через `html::escape`; класс языка у `<code>` пишет `write_opening_tag` с экранированием значения (SEC-001 закрыт). Нет `unsafe`/IO/`unwrap`.
- Зависимости: в `Cargo.lock` только удаления (plist, yaml-rust, time…), версия syntect та же; `cargo deny check` — advisories/bans/licenses/sources ok. gitleaks по 5 файлам: no leaks. semgrep `.semgrep.yml` по `render.rs`: 0 находок.

- ID: SEC-002
- Уровень: NOTICE
- Ссылка: SAST
- Проблема: наборы `p/rust`, `p/secrets` и др. не скачались (semgrep.dev недоступен) — прогнаны только правила проекта.
- Требуемый результат: пробел инструментов; повторить при наличии сети, не блокирует.

### Выполнение тестов
- Статус: APPROVED
- Полный прогон: `suite-run.backend-tests.json` green, tree `cc704cb` = текущему.
- SC-001…005 pass: `highlight_error_writes_escaped_code`, `highlight_ok_written_unchanged`, `--test render` 21/21, `--test document` 30/30; diff снапшотов пуст.
- SC-006 pass: grep пуст, дублей нет, syntect 5.3.0 один.
- SC-007 pass: `just lint` по рецептам, `deny`/`vendor` вне песочницы.

<!-- Формат замечания: ID, уровень BLOCKER/NOTICE, ссылка на FR/AC/файл, проблема, требуемый результат. Без пересказа документа. -->
