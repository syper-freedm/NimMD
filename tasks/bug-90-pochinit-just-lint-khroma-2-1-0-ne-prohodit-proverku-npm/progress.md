# Состояние разработки: Починить just lint: khroma 2.1.0 не проходит проверку npm-лицензий

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #90 (syper-freedm/NimMD)
- Размер: S; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-26T20:38:10+00:00

## Следующий шаг

Заполнить completion-report.md, `validate-completion`, закоммитить ветку, `stack.py ci` по закоммиченному дереву (exit 3 = ENV-BLOCKED → `block-human`, не сливать), влить ветку в main (squash) и `close-issue`.

## Ворота

| Ключ | Значение |
| --- | --- |
| documents.functional_design | approved |
| documents.test_cases | approved |
| documents.user_guide | not_required |
| document_reviews.architecture | approved |
| document_reviews.security | approved |
| document_reviews.qa | approved |
| document_reviews.ux | not_required |
| implementation.backend | done |
| implementation.frontend | not_required |
| implementation.backend_tests | done |
| implementation.frontend_tests | not_required |
| verification.code_review | approved |
| verification.security_review | approved |
| verification.testing | approved |

## Подзадачи

нет

## Журнал (последние события)

- 2026-10-26T20:32:46+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": [], "notes": ["reconcile() прогнан на входах AC-1…AC-5 — результаты совпадают с ожиданиями (префикс каталога добавляет check, как в ARCH-001).", "node scripts/check-npm-licenses.mjs packages/vendor — exit 0; node --test scripts/*.test.mjs — fail 0.", "Тесты reconcile в scripts/check-npm-licenses.test.mjs в диффе пока нет — их добавляет QA/тест-фаза по карте реализации."]}
- 2026-10-26T20:32:46+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": [], "notes": ["cargo deny check advisories запущен вне песочницы: в песочнице нет записи в ~/.cargo/advisory-dbs/db.lock", "semgrep запущен с SEMGREP_LOG_FILE в control/.tmp/security-code-review: в песочнице нет записи в ~/.semgrep"]}
- 2026-10-26T20:32:46+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "BLOCKED", "findings": [], "notes": ["Нужен доступ на запись к scripts/check-npm-licenses.test.mjs (context --allow-path scripts/check-npm-licenses.test.mjs). Без него тесты не написать: write_scope пакета — crates/, apps/desktop/src-tauri/, fuzz/ (test_paths области backend), а правка (scripts/check-npm-licenses.mjs) — это Node-скрипт. Существующие тесты к нему лежат в scripts/check-npm-licenses.test.mjs (node --test scripts/*.test.mjs, его вызывает just test).", "План после выдачи доступа: unit-тесты reconcile() для SC-001..SC-005 в существующем файле scripts/check-npm-licenses.test.mjs (сверка числа нарушений и окончания строк по ARCH-001/QA-001); проверка red-before-fix через git stash скрипта; затем node --test scripts/*.test.mjs, cargo test --workspace --all-targets и запись suite-run.", "SC-006..SC-008 (E2E/integration: just npm-licenses, just lint, pnpm licenses list) — это прогоны рецептов, а не тест-файлы; их выполняет QA на фазе testing.", "Ничего не менял и не запускал: файлов не трогал, suite-run.backend-tests.json не записан."]}
- 2026-10-26T20:32:47+00:00 `write_scope_extended` — {"phase": "backend-tests", "paths": ["scripts/check-npm-licenses.test.mjs"], "reason": "ФД «Карта реализации»: новые тесты в scripts/check-npm-licenses.test.mjs"}
- 2026-10-26T20:32:47+00:00 `note` — {"text": "backend-tests BLOCKED: нет доступа к scripts/*.test.mjs — выдан грант, повторный диспатч"}
- 2026-10-26T20:35:22+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Покрытие: SC-001..SC-005 — тесты «SC-001: уточнение закрывает Unknown», «SC-002: новая версия снова на проверку», «SC-003: уточнение при известной лицензии считается использованным», «SC-004: уточнение не обходит белый список», «SC-005: без уточнений поведение прежнее» в scripts/check-npm-licenses.test.mjs; плюс граничный тест «уточнение одной версии не закрывает соседнюю, пустой вывод pnpm оставляет уточнение неиспользованным».", "ARCH-001/QA-001: reconcile возвращает нарушения без префикса каталога (его добавляет check); тесты сверяют число нарушений и окончание строки через endsWith, unused — точным списком.", "Красный до правки: git stash push -- scripts/check-npm-licenses.mjs; node --test scripts/check-npm-licenses.test.mjs -> 'SyntaxError: ... does not provide an export named reconcile', fail 1; git stash pop; git diff --stat идентичен до/после. Функция новая, поэтому красный прогон без правки возможен только на отсутствующем экспорте.", "Поэтому дополнительно мутационные прогоны на копиях скрипта в control/.tmp/backend-tests (production не трогался): A) уточнения игнорируются (старое поведение) -> падают SC-001, SC-003, SC-004 и граничный, ключевая строка 'AssertionError: Expected values to be strictly deep-equal: + unused: [ khroma@2.1.0 ]'; B) уточнение обходит белый список -> падает SC-004; C) used считается только для Unknown -> падает SC-003; D) версии склеены через запятую (старый формат) -> падают SC-004 ('actual: 1, expected: 2') и граничный.", "Зелёные прогоны: node --test scripts/*.test.mjs — tests 8, pass 8, fail 0, skipped 0; cargo test --workspace --all-targets — все test result ok, 0 failed (единственный ignored — существующий render_one_mib_within_budget, не относится к правке).", "diff-lint.sh backend: no changed files (Rust не менялся); semgrep --config .semgrep.yml по тестовому файлу — 0 находок (вне sandbox-лога: SEMGREP_LOG_FILE в $TMPDIR, т.к. ~/.semgrep/semgrep.log недоступен песочнице). Полный just lint не запускался (нужна сеть и npm-реестр) — это SC-006..SC-008, за QA.", "QA-002 (CLI-путь FR-3: stderr + код 1 и текст «Уточнения без совпадения…») unit-тестами не покрыт: check не экспортирован и вызывает pnpm; косвенно — SC-002 через список unused."]}
- 2026-10-26T20:35:23+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-26T20:37:56+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный набор: control/suite-run.backend-tests.json green, tree e8e34f96e6d3cc9f6f00cf2ed7d2d4185ab3f983 совпадает с текущим; just test по поручению прогнан повторно — код 0, node --test pass 8 fail 0, cargo test все ok (1 ignored вне правки).", "SC-001…SC-005 pass: тесты `SC-001: уточнение закрывает Unknown` … `SC-005: без уточнений поведение прежнее` в scripts/check-npm-licenses.test.mjs.", "SC-006 pass: just npm-licenses вне песочницы — код 0, строк скрипта в stderr нет.", "SC-007 pass: just lint вне песочницы — код 0 (deny ok, vendor ok, npm-licenses ok, machete ok, gitleaks no leaks, actionlint/zizmor без находок); just test — код 0.", "SC-008 pass: pnpm licenses list --json в packages/vendor — khroma [\"2.1.0\"] под MIT (локальный pnpm лицензию распознаёт; ветка Unknown из CI покрыта только unit SC-001).", "Серверов не запускалось; логи — control/.tmp/testing/."]}
- 2026-10-26T20:37:56+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-26T20:37:57+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-26T20:37:57+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-26T20:37:57+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-26T20:37:57+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-26T20:37:57+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-26T20:38:10+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 90,
  "title": "Починить just lint: khroma 2.1.0 не проходит проверку npm-лицензий",
  "slug": "pochinit-just-lint-khroma-2-1-0-ne-prohodit-proverku-npm",
  "branch": "fix/90-pochinit-just-lint-khroma-2-1-0-ne",
  "size": "S",
  "areas": [
    "backend"
  ],
  "state": "completion",
  "result": "in_progress",
  "documents": {
    "functional_design": "approved",
    "test_cases": "approved",
    "user_guide": "not_required"
  },
  "document_reviews": {
    "architecture": "approved",
    "security": "approved",
    "qa": "approved",
    "ux": "not_required"
  },
  "implementation": {
    "backend": "done",
    "frontend": "not_required",
    "backend_tests": "done",
    "frontend_tests": "not_required"
  },
  "verification": {
    "code_review": "approved",
    "security_review": "approved",
    "testing": "approved"
  },
  "subtasks": {
    "required": []
  },
  "blocking_reason": null,
  "history": [
    {
      "at": "2026-10-26T20:27:05+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-26T20:27:46+00:00",
      "event": "classified",
      "size": "S",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-26T20:27:46+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-26T20:27:46+00:00",
      "event": "note",
      "text": "ФД готов (решение пользователя: deny.toml не трогать); старт ревью ФД"
    },
    {
      "at": "2026-10-26T20:28:57+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001"
      ],
      "notes": [
        "architecture.md не создан: долгоживущего решения нет, правка внутри одного infra-скрипта."
      ]
    },
    {
      "at": "2026-10-26T20:28:57+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Изменение затрагивает только dev-скрипт scripts/check-npm-licenses.mjs: новых путей атаки, прав и чувствительных данных нет; уточнение привязано к точной версии и само проходит isAllowed (AC-4), поэтому белый список deny.toml не обходится. security.md не нужен."
      ]
    },
    {
      "at": "2026-10-26T20:28:57+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "AC-1…AC-5 проверяемы unit-тестами node --test в scripts/check-npm-licenses.test.mjs (файл и функции isAllowed/allowedLicenses существуют, just test запускает scripts/*.test.mjs).",
        "QA-001 NOTICE: строки нарушений в AC без префикса каталога из FR-2; сценарии сверяют окончание строки и число нарушений.",
        "QA-002 NOTICE: AC-6 не единичен и зависит от сети, разбит на SC-006…SC-008; CLI-путь FR-3 (stderr, код 1) отдельным AC не покрыт.",
        "Контроль доступа неприменим: CLI-скрипт без ролей."
      ]
    },
    {
      "at": "2026-10-26T20:28:57+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:28:57+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:28:58+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:28:58+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:28:58+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:28:58+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:28:58+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-26T20:28:58+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:28:59+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:28:59+00:00",
      "event": "note",
      "text": "ФД одобрен (NOTICE ARCH-001, QA-001/002 разработчику); старт implementation"
    },
    {
      "at": "2026-10-26T20:30:47+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: scripts/check-npm-licenses.mjs",
        "FR-1..FR-3: OVERRIDES {packages/vendor: {khroma@2.1.0: MIT}} с комментарием о проверке; экспортируемая reconcile(byLicense, allowed, overrides) -> {violations, unused}; check печатает нарушения и 'Уточнения без совпадения в <каталог> (обновите или удалите): <ключи>' в stderr, код 1",
        "ARCH-001/QA-001: reconcile возвращает нарушения без префикса каталога, префикс '<каталог>: ' добавляет check; тесты сверяют окончание строки",
        "QA-002: путь FR-3 на уровне CLI реализован в check; отдельного AC нет, это не моя зона тестов",
        "smoke-проверка reconcile на входах AC-1..AC-5: все ожидания совпали (вручную через node -e, новые тесты не писал)",
        "just npm-licenses: exit 0; node --test scripts/*.test.mjs: pass 2 fail 0; pnpm licenses list в packages/vendor локально отдаёт khroma 2.1.0 под MIT (путь AC-3), путь Unknown из CI проверен только через reconcile",
        "semgrep --config .semgrep.yml по scripts/check-npm-licenses.mjs: exit 0 (запуск вне песочницы: в ней semgrep не может писать ~/.semgrep/semgrep.log)",
        "cargo fmt --check: ok; diff-lint backend: no changed files (только Rust-анализаторы); cargo build/clippy/test/deny не запускал — Rust-файлы не менялись; полный just lint не запускал (нужна сеть)",
        "ponytail: 'Object.hasOwn(overrides,key) ? overrides[key] : undefined' лишнее, ключи всегда с '@' и не задевают прототип — accepted, заменено на overrides[key]",
        "ponytail: 1/1 accepted"
      ]
    },
    {
      "at": "2026-10-26T20:30:47+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-26T20:30:47+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-26T20:30:48+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 1,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "backend-tests",
        "documentation"
      ]
    },
    {
      "at": "2026-10-26T20:30:57+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:30:58+00:00",
      "event": "note",
      "text": "route пропустил backend-tests (scripts/*.mjs не исходник rust), но ФД требует node-тест — фазу backend-tests запускаю вопреки skip; старт post-checks"
    },
    {
      "at": "2026-10-26T20:32:46+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "reconcile() прогнан на входах AC-1…AC-5 — результаты совпадают с ожиданиями (префикс каталога добавляет check, как в ARCH-001).",
        "node scripts/check-npm-licenses.mjs packages/vendor — exit 0; node --test scripts/*.test.mjs — fail 0.",
        "Тесты reconcile в scripts/check-npm-licenses.test.mjs в диффе пока нет — их добавляет QA/тест-фаза по карте реализации."
      ]
    },
    {
      "at": "2026-10-26T20:32:46+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "cargo deny check advisories запущен вне песочницы: в песочнице нет записи в ~/.cargo/advisory-dbs/db.lock",
        "semgrep запущен с SEMGREP_LOG_FILE в control/.tmp/security-code-review: в песочнице нет записи в ~/.semgrep"
      ]
    },
    {
      "at": "2026-10-26T20:32:46+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "BLOCKED",
      "findings": [],
      "notes": [
        "Нужен доступ на запись к scripts/check-npm-licenses.test.mjs (context --allow-path scripts/check-npm-licenses.test.mjs). Без него тесты не написать: write_scope пакета — crates/, apps/desktop/src-tauri/, fuzz/ (test_paths области backend), а правка (scripts/check-npm-licenses.mjs) — это Node-скрипт. Существующие тесты к нему лежат в scripts/check-npm-licenses.test.mjs (node --test scripts/*.test.mjs, его вызывает just test).",
        "План после выдачи доступа: unit-тесты reconcile() для SC-001..SC-005 в существующем файле scripts/check-npm-licenses.test.mjs (сверка числа нарушений и окончания строк по ARCH-001/QA-001); проверка red-before-fix через git stash скрипта; затем node --test scripts/*.test.mjs, cargo test --workspace --all-targets и запись suite-run.",
        "SC-006..SC-008 (E2E/integration: just npm-licenses, just lint, pnpm licenses list) — это прогоны рецептов, а не тест-файлы; их выполняет QA на фазе testing.",
        "Ничего не менял и не запускал: файлов не трогал, suite-run.backend-tests.json не записан."
      ]
    },
    {
      "at": "2026-10-26T20:32:47+00:00",
      "event": "write_scope_extended",
      "phase": "backend-tests",
      "paths": [
        "scripts/check-npm-licenses.test.mjs"
      ],
      "reason": "ФД «Карта реализации»: новые тесты в scripts/check-npm-licenses.test.mjs"
    },
    {
      "at": "2026-10-26T20:32:47+00:00",
      "event": "note",
      "text": "backend-tests BLOCKED: нет доступа к scripts/*.test.mjs — выдан грант, повторный диспатч"
    },
    {
      "at": "2026-10-26T20:35:22+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Покрытие: SC-001..SC-005 — тесты «SC-001: уточнение закрывает Unknown», «SC-002: новая версия снова на проверку», «SC-003: уточнение при известной лицензии считается использованным», «SC-004: уточнение не обходит белый список», «SC-005: без уточнений поведение прежнее» в scripts/check-npm-licenses.test.mjs; плюс граничный тест «уточнение одной версии не закрывает соседнюю, пустой вывод pnpm оставляет уточнение неиспользованным».",
        "ARCH-001/QA-001: reconcile возвращает нарушения без префикса каталога (его добавляет check); тесты сверяют число нарушений и окончание строки через endsWith, unused — точным списком.",
        "Красный до правки: git stash push -- scripts/check-npm-licenses.mjs; node --test scripts/check-npm-licenses.test.mjs -> 'SyntaxError: ... does not provide an export named reconcile', fail 1; git stash pop; git diff --stat идентичен до/после. Функция новая, поэтому красный прогон без правки возможен только на отсутствующем экспорте.",
        "Поэтому дополнительно мутационные прогоны на копиях скрипта в control/.tmp/backend-tests (production не трогался): A) уточнения игнорируются (старое поведение) -> падают SC-001, SC-003, SC-004 и граничный, ключевая строка 'AssertionError: Expected values to be strictly deep-equal: + unused: [ khroma@2.1.0 ]'; B) уточнение обходит белый список -> падает SC-004; C) used считается только для Unknown -> падает SC-003; D) версии склеены через запятую (старый формат) -> падают SC-004 ('actual: 1, expected: 2') и граничный.",
        "Зелёные прогоны: node --test scripts/*.test.mjs — tests 8, pass 8, fail 0, skipped 0; cargo test --workspace --all-targets — все test result ok, 0 failed (единственный ignored — существующий render_one_mib_within_budget, не относится к правке).",
        "diff-lint.sh backend: no changed files (Rust не менялся); semgrep --config .semgrep.yml по тестовому файлу — 0 находок (вне sandbox-лога: SEMGREP_LOG_FILE в $TMPDIR, т.к. ~/.semgrep/semgrep.log недоступен песочнице). Полный just lint не запускался (нужна сеть и npm-реестр) — это SC-006..SC-008, за QA.",
        "QA-002 (CLI-путь FR-3: stderr + код 1 и текст «Уточнения без совпадения…») unit-тестами не покрыт: check не экспортирован и вызывает pnpm; косвенно — SC-002 через список unused."
      ]
    },
    {
      "at": "2026-10-26T20:35:23+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-26T20:37:56+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный набор: control/suite-run.backend-tests.json green, tree e8e34f96e6d3cc9f6f00cf2ed7d2d4185ab3f983 совпадает с текущим; just test по поручению прогнан повторно — код 0, node --test pass 8 fail 0, cargo test все ok (1 ignored вне правки).",
        "SC-001…SC-005 pass: тесты `SC-001: уточнение закрывает Unknown` … `SC-005: без уточнений поведение прежнее` в scripts/check-npm-licenses.test.mjs.",
        "SC-006 pass: just npm-licenses вне песочницы — код 0, строк скрипта в stderr нет.",
        "SC-007 pass: just lint вне песочницы — код 0 (deny ok, vendor ok, npm-licenses ok, machete ok, gitleaks no leaks, actionlint/zizmor без находок); just test — код 0.",
        "SC-008 pass: pnpm licenses list --json в packages/vendor — khroma [\"2.1.0\"] под MIT (локальный pnpm лицензию распознаёт; ветка Unknown из CI покрыта только unit SC-001).",
        "Серверов не запускалось; логи — control/.tmp/testing/."
      ]
    },
    {
      "at": "2026-10-26T20:37:56+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:37:57+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:37:57+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-26T20:37:57+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:37:57+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-26T20:37:57+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-26T20:38:10+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "39e0ba844b3af2ff75b1a901c270358a771a378419336db8b1a208a77b6dc109",
      "sections": {
        "Метаданные": "f5b85cef6a96be9d0de77d857b7eae2b898299b3442a988b6accb1c4544d077e",
        "Цель": "8fc9a4c2dcfc14f79c935360a98ac6f1392bd5625661dd2a0bd2341e29252584",
        "Границы": "13e8927adff43fa1ecf25f5bf391cc521e4bdd8d64930686ca3bf9d653bff4c5",
        "Целевое поведение": "c8d3bf673b073823940316c0514e2b0e83300444a06167145f10e4b9ab538c8c",
        "Требования": "53f691aecb2d538423ee4d25e87b0d87d46c6f75e489f407346cadf3d6dd580c",
        "Данные, права, ошибки": "ea960cf4299c93e307b62db05c1a8701c0ff7a3ab09fc225d935c329e0384274",
        "Карта реализации": "fcbfbc2d85ba7ef47858f9b6c44be29bb150ba96278f717530407d4df0de13e7",
        "Критерии приёмки": "ffc901c26d98972404d8785a7bd52dee0f48e4ebf667d1743f138aa58e654614",
        "Влияние на руководство": "45254709581e4cea05801255d560d1c3d9d7c8826e124e3185c7c9e73b9bd3c1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "27dfa5fdefd37a4c1f4c34402a59bc07ba033ca4a78bfc1e7986100db52639c0"
        },
        "Цель": {
          "Цель": "cb1dedec6ebb0b56c5c2f4a2ef65a8b9462f5e6fa9a2d1d72ca3f3b49e6f9632"
        },
        "Границы": {
          "Границы": "6b996ae2e3eb180508528798a81f557120d887295bd71ede585494e4f02ede0a"
        },
        "Целевое поведение": {
          "Целевое поведение": "53af70bbf4a64ed9743968c54db111a9b507fb614847a1947bfddcc8def6334b"
        },
        "Требования": {
          "FR-1": "78cab962669b9bdb5f699306259f0269739873d1a201e5362a6ebb521bbc92d5",
          "FR-2": "431b33e4c2a194344e1bac436f693678d3371f0be58e824db951df81bd7030bd",
          "FR-3": "c5aadfbae83e72e68e435884d4116b8d973816051bcd350207aab8281e403fc1"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "107bcc5c942c922af9e544c77dd6e376b089c15685bcb1abc3e67a6f2bb70a94"
        },
        "Карта реализации": {
          "Карта реализации": "0ed043bf7a12919576e357d65af896fa7fde08f59a5f3ea016f18696c0f7ddc7"
        },
        "Критерии приёмки": {
          "AC-1": "fd047c6b6b26e2fe40129bbe0c7eec20c5968f5ef01ee3031f60c248ae30cbaa",
          "AC-2": "7583d73f6425346ccdb3d62c34591dddc085fcfd57306a9dd1d6329c893bb133",
          "AC-3": "3eae48d77aa452603f32392475f300e8b10f7a13ed38a390ba297a701a577bae",
          "AC-4": "3dac01cbe21d20ef86bc151eb030a609eceb0faa45bfa99dbf6f60aefa5f0e02",
          "AC-5": "f1c57ada3859efd49d20dd54069972c886619c3eb39e24f800aeb6aacf54f4c5",
          "AC-6": "545bec811d277bb7805df6a4bcd8d9226a456bc7de0439c6b541c4e4d769e546"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "efb622532b2cab39ff51c18c89847a902865c77629f70dcbca4707029013a196"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "39e0ba844b3af2ff75b1a901c270358a771a378419336db8b1a208a77b6dc109",
      "sections": {
        "Метаданные": "f5b85cef6a96be9d0de77d857b7eae2b898299b3442a988b6accb1c4544d077e",
        "Цель": "8fc9a4c2dcfc14f79c935360a98ac6f1392bd5625661dd2a0bd2341e29252584",
        "Границы": "13e8927adff43fa1ecf25f5bf391cc521e4bdd8d64930686ca3bf9d653bff4c5",
        "Целевое поведение": "c8d3bf673b073823940316c0514e2b0e83300444a06167145f10e4b9ab538c8c",
        "Требования": "53f691aecb2d538423ee4d25e87b0d87d46c6f75e489f407346cadf3d6dd580c",
        "Данные, права, ошибки": "ea960cf4299c93e307b62db05c1a8701c0ff7a3ab09fc225d935c329e0384274",
        "Карта реализации": "fcbfbc2d85ba7ef47858f9b6c44be29bb150ba96278f717530407d4df0de13e7",
        "Критерии приёмки": "ffc901c26d98972404d8785a7bd52dee0f48e4ebf667d1743f138aa58e654614",
        "Влияние на руководство": "45254709581e4cea05801255d560d1c3d9d7c8826e124e3185c7c9e73b9bd3c1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "27dfa5fdefd37a4c1f4c34402a59bc07ba033ca4a78bfc1e7986100db52639c0"
        },
        "Цель": {
          "Цель": "cb1dedec6ebb0b56c5c2f4a2ef65a8b9462f5e6fa9a2d1d72ca3f3b49e6f9632"
        },
        "Границы": {
          "Границы": "6b996ae2e3eb180508528798a81f557120d887295bd71ede585494e4f02ede0a"
        },
        "Целевое поведение": {
          "Целевое поведение": "53af70bbf4a64ed9743968c54db111a9b507fb614847a1947bfddcc8def6334b"
        },
        "Требования": {
          "FR-1": "78cab962669b9bdb5f699306259f0269739873d1a201e5362a6ebb521bbc92d5",
          "FR-2": "431b33e4c2a194344e1bac436f693678d3371f0be58e824db951df81bd7030bd",
          "FR-3": "c5aadfbae83e72e68e435884d4116b8d973816051bcd350207aab8281e403fc1"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "107bcc5c942c922af9e544c77dd6e376b089c15685bcb1abc3e67a6f2bb70a94"
        },
        "Карта реализации": {
          "Карта реализации": "0ed043bf7a12919576e357d65af896fa7fde08f59a5f3ea016f18696c0f7ddc7"
        },
        "Критерии приёмки": {
          "AC-1": "fd047c6b6b26e2fe40129bbe0c7eec20c5968f5ef01ee3031f60c248ae30cbaa",
          "AC-2": "7583d73f6425346ccdb3d62c34591dddc085fcfd57306a9dd1d6329c893bb133",
          "AC-3": "3eae48d77aa452603f32392475f300e8b10f7a13ed38a390ba297a701a577bae",
          "AC-4": "3dac01cbe21d20ef86bc151eb030a609eceb0faa45bfa99dbf6f60aefa5f0e02",
          "AC-5": "f1c57ada3859efd49d20dd54069972c886619c3eb39e24f800aeb6aacf54f4c5",
          "AC-6": "545bec811d277bb7805df6a4bcd8d9226a456bc7de0439c6b541c4e4d769e546"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "efb622532b2cab39ff51c18c89847a902865c77629f70dcbca4707029013a196"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "39e0ba844b3af2ff75b1a901c270358a771a378419336db8b1a208a77b6dc109",
      "sections": {
        "Метаданные": "f5b85cef6a96be9d0de77d857b7eae2b898299b3442a988b6accb1c4544d077e",
        "Цель": "8fc9a4c2dcfc14f79c935360a98ac6f1392bd5625661dd2a0bd2341e29252584",
        "Границы": "13e8927adff43fa1ecf25f5bf391cc521e4bdd8d64930686ca3bf9d653bff4c5",
        "Целевое поведение": "c8d3bf673b073823940316c0514e2b0e83300444a06167145f10e4b9ab538c8c",
        "Требования": "53f691aecb2d538423ee4d25e87b0d87d46c6f75e489f407346cadf3d6dd580c",
        "Данные, права, ошибки": "ea960cf4299c93e307b62db05c1a8701c0ff7a3ab09fc225d935c329e0384274",
        "Карта реализации": "fcbfbc2d85ba7ef47858f9b6c44be29bb150ba96278f717530407d4df0de13e7",
        "Критерии приёмки": "ffc901c26d98972404d8785a7bd52dee0f48e4ebf667d1743f138aa58e654614",
        "Влияние на руководство": "45254709581e4cea05801255d560d1c3d9d7c8826e124e3185c7c9e73b9bd3c1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "27dfa5fdefd37a4c1f4c34402a59bc07ba033ca4a78bfc1e7986100db52639c0"
        },
        "Цель": {
          "Цель": "cb1dedec6ebb0b56c5c2f4a2ef65a8b9462f5e6fa9a2d1d72ca3f3b49e6f9632"
        },
        "Границы": {
          "Границы": "6b996ae2e3eb180508528798a81f557120d887295bd71ede585494e4f02ede0a"
        },
        "Целевое поведение": {
          "Целевое поведение": "53af70bbf4a64ed9743968c54db111a9b507fb614847a1947bfddcc8def6334b"
        },
        "Требования": {
          "FR-1": "78cab962669b9bdb5f699306259f0269739873d1a201e5362a6ebb521bbc92d5",
          "FR-2": "431b33e4c2a194344e1bac436f693678d3371f0be58e824db951df81bd7030bd",
          "FR-3": "c5aadfbae83e72e68e435884d4116b8d973816051bcd350207aab8281e403fc1"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "107bcc5c942c922af9e544c77dd6e376b089c15685bcb1abc3e67a6f2bb70a94"
        },
        "Карта реализации": {
          "Карта реализации": "0ed043bf7a12919576e357d65af896fa7fde08f59a5f3ea016f18696c0f7ddc7"
        },
        "Критерии приёмки": {
          "AC-1": "fd047c6b6b26e2fe40129bbe0c7eec20c5968f5ef01ee3031f60c248ae30cbaa",
          "AC-2": "7583d73f6425346ccdb3d62c34591dddc085fcfd57306a9dd1d6329c893bb133",
          "AC-3": "3eae48d77aa452603f32392475f300e8b10f7a13ed38a390ba297a701a577bae",
          "AC-4": "3dac01cbe21d20ef86bc151eb030a609eceb0faa45bfa99dbf6f60aefa5f0e02",
          "AC-5": "f1c57ada3859efd49d20dd54069972c886619c3eb39e24f800aeb6aacf54f4c5",
          "AC-6": "545bec811d277bb7805df6a4bcd8d9226a456bc7de0439c6b541c4e4d769e546"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "efb622532b2cab39ff51c18c89847a902865c77629f70dcbca4707029013a196"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "39e0ba844b3af2ff75b1a901c270358a771a378419336db8b1a208a77b6dc109",
    "backend_tests": "39e0ba844b3af2ff75b1a901c270358a771a378419336db8b1a208a77b6dc109"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства нет.",
    "backend-tests": "в изменениях области нет исходников её тулчейна (\\.rs$, ^packages/theme/.*\\.css$): scripts/check-npm-licenses.mjs"
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "843d56c6e0990085e7f48507a9d0a47068de76d4fc90abec412fcb1b8bec3f89",
      "files": [
        "scripts/check-npm-licenses.mjs"
      ],
      "production_hash": "843d56c6e0990085e7f48507a9d0a47068de76d4fc90abec412fcb1b8bec3f89",
      "production_files": [
        "scripts/check-npm-licenses.mjs"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "843d56c6e0990085e7f48507a9d0a47068de76d4fc90abec412fcb1b8bec3f89",
      "files": [
        "scripts/check-npm-licenses.mjs"
      ],
      "production_hash": "843d56c6e0990085e7f48507a9d0a47068de76d4fc90abec412fcb1b8bec3f89",
      "production_files": [
        "scripts/check-npm-licenses.mjs"
      ]
    }
  },
  "write_scope_extensions": {
    "backend-tests": [
      {
        "path": "scripts/check-npm-licenses.test.mjs",
        "reason": "ФД «Карта реализации»: новые тесты в scripts/check-npm-licenses.test.mjs"
      }
    ]
  },
  "accepted_code_hash": {
    "backend-tests": "f622b7a16f3b4bf79075e0380ddebb227de3903f1f459d2e2c950bf3f85b5c9a",
    "testing": "f622b7a16f3b4bf79075e0380ddebb227de3903f1f459d2e2c950bf3f85b5c9a"
  }
}
```
<!-- afd-state-snapshot:end -->
