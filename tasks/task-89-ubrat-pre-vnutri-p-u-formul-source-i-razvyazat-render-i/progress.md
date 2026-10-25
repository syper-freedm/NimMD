# Состояние разработки: Убрать <pre> внутри <p> у формул Source и развязать render и document

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #89 (syper-freedm/NimMD)
- Размер: S; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T22:22:10+00:00

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

- 2026-10-25T22:15:27+00:00 `routed` — {"post_checks_parallel": true, "files_count": 6, "has_migrations": false, "touches_isolation": false, "skip": ["documentation"]}
- 2026-10-25T22:15:32+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-25T22:15:33+00:00 `note` — {"text": "диспатч post-checks"}
- 2026-10-25T22:21:53+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": [], "notes": ["Все 6 файлов diff совпадают с control/changed-files.txt и картой реализации; FR-1..FR-3 реализованы, запасной вывод MathML идёт через source().", "AC-5: git grep 'crate::document' в render.rs пуст, 'enum MathMode' только в render.rs; cargo test -p nimmd-core --test document — 30 passed, clippy -D warnings чистый.", "Новые тесты из карты (insta-снапшот Source, проверка всех режимов на <pre, math-display в тесте контракта) в diff пока нет — их добавляет этап тестов, это не ревьюилось."]}
- 2026-10-25T22:21:53+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": ["SEC-001"], "notes": ["semgrep по изменённым исходникам: 0 находок; gitleaks по changed-files.txt: no leaks; cargo deny bans/licenses/sources: ok", "SEC-001 NOTICE: cargo deny advisories не отработал (песочница: read-only lock ~/.cargo/advisory-dbs/db.lock); зависимости не менялись"]}
- 2026-10-25T22:21:53+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Coverage: SC-001 -> math_source_mode (assertions + html_classes contains math-display + insta snapshot document__math_source_mode.snap); SC-002 -> display_math_has_no_pre_in_any_mode; SC-004 -> dangerous_display_formulas_escaped_in_source; SC-005 -> class_contract, contract_core_classes_appear_in_output, math_source_mode; SC-007 -> rendered_serializes_lowercase + full suite", "QA-001 handled: math_source_mode asserts contract_list(\"Только ядра\") contains math-display and html_classes(body) of Source output contains it", "Red-before-fix: git stash push -- crates/nimmd-core/src/{render,document,lib}.rs; cargo test -p nimmd-core --test document -> FAILED 3: math_source_mode (document.rs:355, contains <p><code class=\"math-display\">), display_math_has_no_pre_in_any_mode (document.rs:374, !body.contains(\"<pre\")), dangerous_display_formulas_escaped_in_source (document.rs:395, contains expected math-display code); 29 passed; stash popped, git diff --stat identical", "Static: SC-003 git diff --exit-code main -- document__math_mathml_mode.snap rc=0; SC-006 git grep crate::document render.rs rc=1, enum MathMode only crates/nimmd-core/src/render.rs:29; SC-007 no changed use lines in tests (grep rc=1)", "Checks green: cargo fmt --check, cargo build --workspace --all-targets, cargo clippy --workspace --all-targets -D warnings, diff-lint backend (exit 0, clean), just lint (exit 0), cargo test --workspace --all-targets (exit 0, 0 ignored)", "diff-lint and just lint run outside the sandbox: semgrep cannot write ~/.semgrep/semgrep.log in the sandbox (PermissionError), just lint needs network for vendor/deny"]}
- 2026-10-25T22:21:53+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный cargo test --workspace --all-targets не повторялся: suite-run.backend-tests.json green, tree a2cd516b98a04a8dfdbd1ffd6498b17789374831 совпадает с tree-hash.sh", "cargo test -p nimmd-core --test document: 32 passed, 0 failed (SC-001, SC-002, SC-004, SC-005, SC-007)", "SC-003: git diff --exit-code main -- document__math_mathml_mode.snap — exit 0; SC-006: crate::document в render.rs нет (exit 1), enum MathMode только render.rs:29", "just lint в песочнице упал на PermissionError: ~/.semgrep/semgrep.log (песочница); повтор вне песочницы — exit 0, cargo deny: advisories ok, bans ok, licenses ok, sources ok"]}
- 2026-10-25T22:21:53+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-25T22:21:54+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T22:21:54+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T22:21:54+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T22:21:54+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T22:21:54+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T22:21:54+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T22:22:10+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 89,
  "title": "Убрать <pre> внутри <p> у формул Source и развязать render и document",
  "slug": "ubrat-pre-vnutri-p-u-formul-source-i-razvyazat-render-i",
  "branch": "chore/89-ubrat-pre-vnutri-p-u-formul-source-i",
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
      "at": "2026-10-25T22:09:51+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T22:10:59+00:00",
      "event": "note",
      "text": "шаг 1 делегирован: развилка «класс vs снять <p>» решена по issue — строчный <code class=math-display> (допустим в <p>, текст рядом с формулой не ломается)"
    },
    {
      "at": "2026-10-25T22:10:59+00:00",
      "event": "classified",
      "size": "S",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T22:11:00+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T22:11:00+00:00",
      "event": "note",
      "text": "диспатч fd-review"
    },
    {
      "at": "2026-10-25T22:12:46+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "render.rs импортирует MathMode из crate::document (единственная обратная связь); перенос enum и реэкспорт в lib.rs сохраняют путь nimmd_core::MathMode",
        "Display-формула comrak — inline-узел в <p>: <code class=\"math-display\"> внутри <p> валиден; MathML-fallback идёт через тот же source()",
        "Тест контракта class_contract проверяет селекторы nimmd.css ⊆ CONTRACT.md — math-display нужно добавить в оба файла, как и требует FR-2",
        "architecture.md не создан: долговременного решения нет"
      ]
    },
    {
      "at": "2026-10-25T22:12:46+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Сверено с кодом: source() в render.rs экранирует TeX через push_escaped (comrak html::escape), MathML откатывается на source(); класс math-display — константа ядра, данные документа в атрибут не попадают.",
        "Новой поверхности атаки нет: API, права, данные и ошибки не меняются; security.md не нужен."
      ]
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001"
      ],
      "notes": [
        "Все AC-1..AC-5 наблюдаемы и детерминированы; утверждения ФД сверены с render.rs:19,140, lib.rs:12, CONTRACT.md, tests/document.rs (math_source_mode, class_contract, contract_core_classes_appear_in_output).",
        "QA-001 (NOTICE): contract_core_classes_appear_in_output проверяет жёсткий список, math-display нужно проверить явно (SC-001).",
        "Контроля доступа нет (библиотека) — объектные сценарии неприменимы.",
        "just lint в SC-007 требует сети (vendor, deny) — возможен ENV-сбой."
      ]
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:12:47+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T22:12:48+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:12:48+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:12:48+00:00",
      "event": "note",
      "text": "FD APPROVED (QA-001 NOTICE → тестам); диспатч backend"
    },
    {
      "at": "2026-10-25T22:15:26+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: render.rs, document.rs, lib.rs (nimmd-core/src), tests/document.rs (adapted math_source_mode only), packages/theme/CONTRACT.md, packages/theme/nimmd.css",
        "FR-1: source() outputs <code class=\"math-display\"> for display, <code> for inline; MathML fallback uses the same function",
        "FR-2: math-display added to «Только ядра»; nimmd.css rule code.math-display {display:block; white-space:pre-wrap; padding/radius/overflow as pre; background inherited from code}",
        "FR-3: enum MathMode moved to render.rs; document.rs imports crate::render::MathMode; lib.rs re-exports render::{MathMode, render_fragment}; git grep crate::document in render.rs is empty; enum MathMode only in render.rs",
        "checks: cargo fmt --check, build, clippy -D warnings green; cargo test --workspace --all-targets: all suites ok (2+27+30+7+21+14 passed); cargo deny: advisories/bans/licenses/sources ok; just lint rc=0; diff-lint: clippy-changed clean, machete clean, semgrep clean",
        "MathML snapshot document__math_mathml_mode.snap unchanged (no snapshot diffs)",
        "cargo deny / diff-lint / just lint ran outside sandbox: sandbox blocks write to ~/.cargo/advisory-dbs lock and ~/.semgrep/semgrep.log",
        "NOTICE QA-001 (math-display in contract test / html_classes of Source output) is a test addition: left to afd-test-developer on backend-tests",
        "migrations: none; isolation: n/a; arch_map: n/a",
        "ponytail: 0/0 accepted"
      ]
    },
    {
      "at": "2026-10-25T22:15:26+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T22:15:27+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T22:15:27+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 6,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ]
    },
    {
      "at": "2026-10-25T22:15:32+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:15:33+00:00",
      "event": "note",
      "text": "диспатч post-checks"
    },
    {
      "at": "2026-10-25T22:21:53+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Все 6 файлов diff совпадают с control/changed-files.txt и картой реализации; FR-1..FR-3 реализованы, запасной вывод MathML идёт через source().",
        "AC-5: git grep 'crate::document' в render.rs пуст, 'enum MathMode' только в render.rs; cargo test -p nimmd-core --test document — 30 passed, clippy -D warnings чистый.",
        "Новые тесты из карты (insta-снапшот Source, проверка всех режимов на <pre, math-display в тесте контракта) в diff пока нет — их добавляет этап тестов, это не ревьюилось."
      ]
    },
    {
      "at": "2026-10-25T22:21:53+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001"
      ],
      "notes": [
        "semgrep по изменённым исходникам: 0 находок; gitleaks по changed-files.txt: no leaks; cargo deny bans/licenses/sources: ok",
        "SEC-001 NOTICE: cargo deny advisories не отработал (песочница: read-only lock ~/.cargo/advisory-dbs/db.lock); зависимости не менялись"
      ]
    },
    {
      "at": "2026-10-25T22:21:53+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Coverage: SC-001 -> math_source_mode (assertions + html_classes contains math-display + insta snapshot document__math_source_mode.snap); SC-002 -> display_math_has_no_pre_in_any_mode; SC-004 -> dangerous_display_formulas_escaped_in_source; SC-005 -> class_contract, contract_core_classes_appear_in_output, math_source_mode; SC-007 -> rendered_serializes_lowercase + full suite",
        "QA-001 handled: math_source_mode asserts contract_list(\"Только ядра\") contains math-display and html_classes(body) of Source output contains it",
        "Red-before-fix: git stash push -- crates/nimmd-core/src/{render,document,lib}.rs; cargo test -p nimmd-core --test document -> FAILED 3: math_source_mode (document.rs:355, contains <p><code class=\"math-display\">), display_math_has_no_pre_in_any_mode (document.rs:374, !body.contains(\"<pre\")), dangerous_display_formulas_escaped_in_source (document.rs:395, contains expected math-display code); 29 passed; stash popped, git diff --stat identical",
        "Static: SC-003 git diff --exit-code main -- document__math_mathml_mode.snap rc=0; SC-006 git grep crate::document render.rs rc=1, enum MathMode only crates/nimmd-core/src/render.rs:29; SC-007 no changed use lines in tests (grep rc=1)",
        "Checks green: cargo fmt --check, cargo build --workspace --all-targets, cargo clippy --workspace --all-targets -D warnings, diff-lint backend (exit 0, clean), just lint (exit 0), cargo test --workspace --all-targets (exit 0, 0 ignored)",
        "diff-lint and just lint run outside the sandbox: semgrep cannot write ~/.semgrep/semgrep.log in the sandbox (PermissionError), just lint needs network for vendor/deny"
      ]
    },
    {
      "at": "2026-10-25T22:21:53+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный cargo test --workspace --all-targets не повторялся: suite-run.backend-tests.json green, tree a2cd516b98a04a8dfdbd1ffd6498b17789374831 совпадает с tree-hash.sh",
        "cargo test -p nimmd-core --test document: 32 passed, 0 failed (SC-001, SC-002, SC-004, SC-005, SC-007)",
        "SC-003: git diff --exit-code main -- document__math_mathml_mode.snap — exit 0; SC-006: crate::document в render.rs нет (exit 1), enum MathMode только render.rs:29",
        "just lint в песочнице упал на PermissionError: ~/.semgrep/semgrep.log (песочница); повтор вне песочницы — exit 0, cargo deny: advisories ok, bans ok, licenses ok, sources ok"
      ]
    },
    {
      "at": "2026-10-25T22:21:53+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-25T22:21:54+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:21:54+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:21:54+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T22:21:54+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:21:54+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T22:21:54+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T22:22:10+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "ace50b26695ee98338fff27dfd09629840da0f3ebc116c0c33df911b5c446595",
      "sections": {
        "Метаданные": "58ddaf7d7b3fe638c3bb2b294c15f7398fe65aa736d1889e26d38d33542df30f",
        "Цель": "1fbc535c328d1206c05b214018158f06d3970e9802578bfdecb505b703052704",
        "Границы": "f8ee6a739e28aec7c992cb15251e3e9e871b2846e943f556a1d1ce0d23c6e72a",
        "Целевое поведение": "012e1139b2a32d8457651b9e4054185c0c0a99d3bfaafd5c5261e82c1fba298b",
        "Требования": "4da394362b547b961d65490a4160b7be821989bef2c48daff8e9f350f197ab08",
        "Данные, права, ошибки": "0144b610f3cb7743154a2778126292792f7b80d735be69877c4c8d0d02e65656",
        "Карта реализации": "29305e252f5f293fdfe0b4c2d1c65b6727c70eb1605264d8f273b8755ea174e9",
        "Критерии приёмки": "f064f63c8b9254026b46c147e433893c8cc08515719ee0c34130a2ad0d819204",
        "Влияние на руководство": "e62fe8fc64237421d273c9798b9aaf72fd300fe1d3579e967ff18902807de794"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "a384d871e9aba6201b30768987f37cdecae231d8e626e99204b97616091d6d52"
        },
        "Цель": {
          "Цель": "58ac079a4b1dc7063031ec52ce18da2c7d41a933224c805ecc4debb075cdd5d4"
        },
        "Границы": {
          "Границы": "ec6eaf3d358eb8255b6d10dab46c836cc833ebae250ec14d030400748d1bfce0"
        },
        "Целевое поведение": {
          "Целевое поведение": "d03abf2ec708011dc40a338b4c93599703690763858af18b7eaa43b57c7d47b1"
        },
        "Требования": {
          "FR-1": "cf65bffac982a3c25c079a64e7fee5ad62147779911d9370661088888185a21b",
          "FR-2": "685bfc81ea4a7f390856f1fd60c616ce49812dac99b945b71dc6d490c58fa819",
          "FR-3": "94e37e5e48c90ae213564bee1313556942fea0b00748dfc993ee155e9a6ac274"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "560442a43afe2ca2cb52cdd5d62c01c27e2379f4f97f39ab5e075ca311696256"
        },
        "Карта реализации": {
          "Карта реализации": "5fcce1e4458cb1587fdb14df4c51d02b5a03eebc54e8a5bec700194c083d9090"
        },
        "Критерии приёмки": {
          "AC-1": "4ce5a0fc38d657dac51a7ca460ca42164d873cf17c6fb8b1bc0c408f16072771",
          "AC-2": "ca69ac69f436135aa329f2b1855e88c3b8bfe4060a345dc882b9f2f1f8d98efe",
          "AC-3": "b9b1c3d27309be67a4076ff4827ca1e26fbcc4d998f1cafe5b9c07bb904b4b52",
          "AC-4": "f25828b044eaf35d9ce31cbd8135485f6526732eba32c2054b22d40c06a25c30",
          "AC-5": "b8d8bb282ae745f307058a4c69fd27271ba44ea0860835cae2cb59f61c2ce4fa"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "c9476fe7f1d153b6917f84f5bf9c419ad3760206d58cc5f897d41b6adeccc30e"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "ace50b26695ee98338fff27dfd09629840da0f3ebc116c0c33df911b5c446595",
      "sections": {
        "Метаданные": "58ddaf7d7b3fe638c3bb2b294c15f7398fe65aa736d1889e26d38d33542df30f",
        "Цель": "1fbc535c328d1206c05b214018158f06d3970e9802578bfdecb505b703052704",
        "Границы": "f8ee6a739e28aec7c992cb15251e3e9e871b2846e943f556a1d1ce0d23c6e72a",
        "Целевое поведение": "012e1139b2a32d8457651b9e4054185c0c0a99d3bfaafd5c5261e82c1fba298b",
        "Требования": "4da394362b547b961d65490a4160b7be821989bef2c48daff8e9f350f197ab08",
        "Данные, права, ошибки": "0144b610f3cb7743154a2778126292792f7b80d735be69877c4c8d0d02e65656",
        "Карта реализации": "29305e252f5f293fdfe0b4c2d1c65b6727c70eb1605264d8f273b8755ea174e9",
        "Критерии приёмки": "f064f63c8b9254026b46c147e433893c8cc08515719ee0c34130a2ad0d819204",
        "Влияние на руководство": "e62fe8fc64237421d273c9798b9aaf72fd300fe1d3579e967ff18902807de794"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "a384d871e9aba6201b30768987f37cdecae231d8e626e99204b97616091d6d52"
        },
        "Цель": {
          "Цель": "58ac079a4b1dc7063031ec52ce18da2c7d41a933224c805ecc4debb075cdd5d4"
        },
        "Границы": {
          "Границы": "ec6eaf3d358eb8255b6d10dab46c836cc833ebae250ec14d030400748d1bfce0"
        },
        "Целевое поведение": {
          "Целевое поведение": "d03abf2ec708011dc40a338b4c93599703690763858af18b7eaa43b57c7d47b1"
        },
        "Требования": {
          "FR-1": "cf65bffac982a3c25c079a64e7fee5ad62147779911d9370661088888185a21b",
          "FR-2": "685bfc81ea4a7f390856f1fd60c616ce49812dac99b945b71dc6d490c58fa819",
          "FR-3": "94e37e5e48c90ae213564bee1313556942fea0b00748dfc993ee155e9a6ac274"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "560442a43afe2ca2cb52cdd5d62c01c27e2379f4f97f39ab5e075ca311696256"
        },
        "Карта реализации": {
          "Карта реализации": "5fcce1e4458cb1587fdb14df4c51d02b5a03eebc54e8a5bec700194c083d9090"
        },
        "Критерии приёмки": {
          "AC-1": "4ce5a0fc38d657dac51a7ca460ca42164d873cf17c6fb8b1bc0c408f16072771",
          "AC-2": "ca69ac69f436135aa329f2b1855e88c3b8bfe4060a345dc882b9f2f1f8d98efe",
          "AC-3": "b9b1c3d27309be67a4076ff4827ca1e26fbcc4d998f1cafe5b9c07bb904b4b52",
          "AC-4": "f25828b044eaf35d9ce31cbd8135485f6526732eba32c2054b22d40c06a25c30",
          "AC-5": "b8d8bb282ae745f307058a4c69fd27271ba44ea0860835cae2cb59f61c2ce4fa"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "c9476fe7f1d153b6917f84f5bf9c419ad3760206d58cc5f897d41b6adeccc30e"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "ace50b26695ee98338fff27dfd09629840da0f3ebc116c0c33df911b5c446595",
      "sections": {
        "Метаданные": "58ddaf7d7b3fe638c3bb2b294c15f7398fe65aa736d1889e26d38d33542df30f",
        "Цель": "1fbc535c328d1206c05b214018158f06d3970e9802578bfdecb505b703052704",
        "Границы": "f8ee6a739e28aec7c992cb15251e3e9e871b2846e943f556a1d1ce0d23c6e72a",
        "Целевое поведение": "012e1139b2a32d8457651b9e4054185c0c0a99d3bfaafd5c5261e82c1fba298b",
        "Требования": "4da394362b547b961d65490a4160b7be821989bef2c48daff8e9f350f197ab08",
        "Данные, права, ошибки": "0144b610f3cb7743154a2778126292792f7b80d735be69877c4c8d0d02e65656",
        "Карта реализации": "29305e252f5f293fdfe0b4c2d1c65b6727c70eb1605264d8f273b8755ea174e9",
        "Критерии приёмки": "f064f63c8b9254026b46c147e433893c8cc08515719ee0c34130a2ad0d819204",
        "Влияние на руководство": "e62fe8fc64237421d273c9798b9aaf72fd300fe1d3579e967ff18902807de794"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "a384d871e9aba6201b30768987f37cdecae231d8e626e99204b97616091d6d52"
        },
        "Цель": {
          "Цель": "58ac079a4b1dc7063031ec52ce18da2c7d41a933224c805ecc4debb075cdd5d4"
        },
        "Границы": {
          "Границы": "ec6eaf3d358eb8255b6d10dab46c836cc833ebae250ec14d030400748d1bfce0"
        },
        "Целевое поведение": {
          "Целевое поведение": "d03abf2ec708011dc40a338b4c93599703690763858af18b7eaa43b57c7d47b1"
        },
        "Требования": {
          "FR-1": "cf65bffac982a3c25c079a64e7fee5ad62147779911d9370661088888185a21b",
          "FR-2": "685bfc81ea4a7f390856f1fd60c616ce49812dac99b945b71dc6d490c58fa819",
          "FR-3": "94e37e5e48c90ae213564bee1313556942fea0b00748dfc993ee155e9a6ac274"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "560442a43afe2ca2cb52cdd5d62c01c27e2379f4f97f39ab5e075ca311696256"
        },
        "Карта реализации": {
          "Карта реализации": "5fcce1e4458cb1587fdb14df4c51d02b5a03eebc54e8a5bec700194c083d9090"
        },
        "Критерии приёмки": {
          "AC-1": "4ce5a0fc38d657dac51a7ca460ca42164d873cf17c6fb8b1bc0c408f16072771",
          "AC-2": "ca69ac69f436135aa329f2b1855e88c3b8bfe4060a345dc882b9f2f1f8d98efe",
          "AC-3": "b9b1c3d27309be67a4076ff4827ca1e26fbcc4d998f1cafe5b9c07bb904b4b52",
          "AC-4": "f25828b044eaf35d9ce31cbd8135485f6526732eba32c2054b22d40c06a25c30",
          "AC-5": "b8d8bb282ae745f307058a4c69fd27271ba44ea0860835cae2cb59f61c2ce4fa"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "c9476fe7f1d153b6917f84f5bf9c419ad3760206d58cc5f897d41b6adeccc30e"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "ace50b26695ee98338fff27dfd09629840da0f3ebc116c0c33df911b5c446595",
    "backend_tests": "ace50b26695ee98338fff27dfd09629840da0f3ebc116c0c33df911b5c446595"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства нет; публичный API не меняется, новый класс описан в `packages/theme/CONTRACT.md`."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "79dbe3d0bc70fa3ea6a352c8fd42b2547597137b8c0b8254dd858a2fa2d6bd01",
      "files": [
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/tests/document.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/nimmd.css",
        "crates/nimmd-core/tests/snapshots/document__math_source_mode.snap"
      ],
      "production_hash": "e86cf02bbe259e9265ceedfe2c4f5063bce681eadbf1fabe04782f960775ff62",
      "production_files": [
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/nimmd.css"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "79dbe3d0bc70fa3ea6a352c8fd42b2547597137b8c0b8254dd858a2fa2d6bd01",
      "files": [
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/tests/document.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/nimmd.css",
        "crates/nimmd-core/tests/snapshots/document__math_source_mode.snap"
      ],
      "production_hash": "e86cf02bbe259e9265ceedfe2c4f5063bce681eadbf1fabe04782f960775ff62",
      "production_files": [
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/nimmd.css"
      ]
    }
  },
  "accepted_code_hash": {
    "backend-tests": "84b48d4b1a6312dfb4b033d4691c3120d756e309f464736c7d3e1ae61220abf0",
    "testing": "84b48d4b1a6312dfb4b033d4691c3120d756e309f464736c7d3e1ae61220abf0"
  }
}
```
<!-- afd-state-snapshot:end -->
