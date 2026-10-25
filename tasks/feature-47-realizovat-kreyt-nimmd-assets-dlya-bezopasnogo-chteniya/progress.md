# Состояние разработки: Реализовать крейт nimmd-assets для безопасного чтения картинок

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #47 (syper-freedm/NimMD)
- Размер: M; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T23:25:54+00:00

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
| implementation.backend_tests | done |
| implementation.frontend | not_required |
| implementation.frontend_tests | not_required |
| verification.code_review | approved |
| verification.security_review | approved |
| verification.testing | approved |

## Подзадачи

нет

## Журнал (последние события)

- 2026-10-25T23:14:57+00:00 `routed` — {"post_checks_parallel": true, "files_count": 4, "has_migrations": false, "touches_isolation": false, "skip": ["documentation"]}
- 2026-10-25T23:15:03+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-25T23:15:04+00:00 `note` — {"text": "диспатч post-checks"}
- 2026-10-25T23:25:18+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": [], "notes": ["Все 4 файла из changed-files.txt связаны с FR-1..FR-4; git diff по Cargo.toml/Cargo.lock — только members и запись пакета без зависимостей.", "cargo fmt --check и cargo clippy -p nimmd-assets --all-targets -D warnings — чисто; grep unsafe/unwrap()/expect( в src — только #![forbid(unsafe_code)].", "crates/nimmd-assets/tests/assets.rs из Карты реализации ещё не создан — тесты за QA/тест-этапом, на этом ревью не проверялись.", "semgrep локально не запустился: sandbox запретил запись ~/.semgrep/semgrep.log; правило nimmd-core-no-io на nimmd-assets не распространяется (include /crates/nimmd-core/src/**)."]}
- 2026-10-25T23:25:18+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": ["SEC-003"], "notes": ["cargo deny advisories не прогнан: песочница запрещает lock ~/.cargo/advisory-dbs/db.lock; у nimmd-assets нет зависимостей", "semgrep запущен с HOME=tmp_dir (лог ~/.semgrep запрещён песочницей), exit 0, находок нет"]}
- 2026-10-25T23:25:18+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Coverage: SC-001 reads_images_next_to_document_and_counts_each_file_once; SC-002 rejects_before_touching_filesystem; SC-003 symlink_inside_directory_leading_out_escapes; SC-004 document_symlink_resolves_base_to_target_directory; SC-005 only_regular_image_files_are_read + symlink_with_image_name_to_non_image_is_not_image (unix); SC-006 file_size_limit; SC-007 budget_counts_unique_files; SC-009 errors_display_distinct_russian_text; SC-008 static: git grep gives only lib.rs:5 #![forbid(unsafe_code)]; SC-010 cargo test --workspace --all-targets green", "Edge cases: rejection_edge_cases (broken %, encoded ':'/'/'/'~', uppercase scheme, '..' deep, not-scheme '1a:'/':x'), decodes_once_not_twice (%252e is a literal file name), budget_boundary_is_inclusive (4x MAX_ASSET_BYTES == MAX_BUDGET_BYTES is ok, the next byte is BudgetExceeded, empty file ok), symlink_to_image_inside_directory_is_counted_once, missing_document_or_directory_is_io; differential percent_encoding_every_byte_does_not_change_verdict (LCG seed 0x6e_696d_6d64, 2000 cases, reference = same href without encoding)", "Red evidence: a new crate (feature), so stashing the production code only gives a compile failure; instead I ran mutations on lib.rs (backup in tmp_dir, restored, shasum -c OK) with cargo test -p nimmd-assets --test assets. All 8 caught: drop starts_with(base) -> symlink_inside_directory_leading_out_escapes 'left: Ok([115,...]) right: Err(Escape)'; known=false -> 4 tests 'left: 101 right: 81'; budget '>' -> '>=' -> budget_boundary_is_inclusive; TooLarge '>' -> '>=' -> file_size_limit; split on '/' only -> rejects_before_touching_filesystem 'left: Err(Io) right: Err(Escape)'; non-canonical doc -> document_symlink_resolves_base_to_target_directory; drop is_file -> only_regular_image_files_are_read 'left: Err(Io) right: Err(NotImage)'; double decode -> decodes_once_not_twice", "Checks: cargo fmt --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -D warnings OK; cargo test --workspace --all-targets: all green, 0 ignored (nimmd-assets 15 passed); diff-lint backend exit 0 (clippy-changed clean, machete clean, semgrep clean); cargo machete clean", "Not run: cargo deny check — the sandbox blocks the lock on ~/.cargo/advisory-dbs (read-only path); my test files add no dependencies. The full just lint was not run (vendor/deny need network and write access outside the sandbox). semgrep only runs with SEMGREP_LOG_FILE/SEMGREP_SETTINGS_FILE pointed at tmp_dir (the sandbox does not allow writing ~/.semgrep)", "Unix tests (SC-003, SC-004, symlinks) run on macOS; on Windows they are compiled out by #[cfg(unix)] as SC-003/004/005 prescribe. The tests write about 70 MB of fixtures to target/tmp/nimmd-assets (CARGO_TARGET_TMPDIR)", "SEC-001/SEC-002 (NOTICE): a host concern; the implementation's doc comments cover them, and the tests check that repeat reads do not consume budget (budget_counts_unique_files)"]}
- 2026-10-25T23:25:18+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный набор не перезапускал: control/suite-run.backend-tests.json result=green, tree 4f38b89445b9ebd371dfa91070d20e2fe0d22c7e = текущий tree-hash.sh.", "SC-001..SC-007, SC-009: cargo test -p nimmd-assets — 15 passed, 0 failed, 0 ignored (unix-тесты симлинков выполнены на macOS, не пропущены).", "SC-008: git grep без --untracked даёт 0 совпадений — crates/nimmd-assets/ ещё не в индексе; git grep --untracked -n \"unsafe\\|unwrap()\\|expect(\" crates/nimmd-assets/src → только lib.rs:5 #![forbid(unsafe_code)].", "SC-010: just lint по шагам — fmt-check, clippy, semgrep (лог semgrep перенаправлен в tmp_dir через SEMGREP_LOG_FILE), unused-deps (machete: нет неиспользуемых), secrets (no leaks), workflows — rc=0 в песочнице; deny, vendor, npm-licenses — rc=0 вне песочницы (песочница запрещает lock ~/.cargo/advisory-dbs и ~/.semgrep). deny: advisories ok, bans ok, licenses ok, sources ok — закрывает SEC-003.", "вне объёма: cargo deny предупреждает «unmatched license exception» для cbindgen, deny.toml:28.", "Наблюдение без влияния на вердикт: между canonicalize и File::open остаётся окно TOCTOU (подмена симлинка); AC его не покрывают."]}
- 2026-10-25T23:25:19+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-25T23:25:19+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T23:25:19+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T23:25:19+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T23:25:19+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T23:25:19+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T23:25:19+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T23:25:54+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 47,
  "title": "Реализовать крейт nimmd-assets для безопасного чтения картинок",
  "slug": "realizovat-kreyt-nimmd-assets-dlya-bezopasnogo-chteniya",
  "branch": "feat/47-realizovat-kreyt-nimmd-assets-dlya",
  "size": "M",
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
    "backend_tests": "done",
    "frontend": "not_required",
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
      "at": "2026-10-25T23:06:06+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T23:07:36+00:00",
      "event": "note",
      "text": "шаг 1 делегирован: развилок нет — спек 5.3/8 задают правило; решено сам: \\ — разделитель на любой ОС, лексический .. до ФС, MIME вне объёма, симлинк-тесты под cfg(unix)"
    },
    {
      "at": "2026-10-25T23:07:36+00:00",
      "event": "classified",
      "size": "M",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T23:07:36+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T23:07:37+00:00",
      "event": "note",
      "text": "диспатч fd-review"
    },
    {
      "at": "2026-10-25T23:09:46+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002"
      ],
      "notes": [
        "Дублей нет (git grep read_asset/canonicalize/nimmd-assets пуст); правило semgrep nimmd-core-no-io ограничено crates/nimmd-core/src, новый крейт с std::fs его не нарушает; architecture.md не создавался — контракт полностью зафиксирован в FR-2..FR-4."
      ]
    },
    {
      "at": "2026-10-25T23:09:46+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001",
        "SEC-002"
      ],
      "notes": [
        "security.md не создан: новых прав, классов чувствительных данных и защищённых потоков (сессия/auth/лицензия/платёж) нет; модель угроз описана в разделе ФД «Данные, права, ошибки».",
        "Секретов при сверке не встречено."
      ]
    },
    {
      "at": "2026-10-25T23:09:47+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "QA-001, QA-002 — NOTICE, вердикт не блокируют",
        "Ролей в крейте нет; изоляция данных (граница каталога документа) покрыта SC-002…SC-004",
        "Правило semgrep nimmd-core-no-io ограничено /crates/nimmd-core/src/** — std::fs в nimmd-assets его не нарушает"
      ]
    },
    {
      "at": "2026-10-25T23:09:47+00:00",
      "event": "note",
      "text": "ARCH-001/QA-001 (\\ и сборка кандидата), ARCH-002 (нет родителя), QA-002 (критерий Display) → правка ФД; SEC-001 → разработчику (doc-комментарий); SEC-002 → комментарий в #50"
    },
    {
      "at": "2026-10-25T23:09:47+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "requirements_rework",
      "reason": "ARCH-001, ARCH-002, QA-001, QA-002: правка ФД пачкой"
    },
    {
      "at": "2026-10-25T23:09:47+00:00",
      "event": "transition",
      "from": "requirements_rework",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T23:09:54+00:00",
      "event": "note",
      "text": "confirm-круг FD (AC-1, AC-6, FR-2)"
    },
    {
      "at": "2026-10-25T23:11:00+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002"
      ],
      "notes": [
        "Режим дельты: правки FR-2 шаг 4, AC-1, AC-6 закрывают ARCH-001 (кандидат из компонентов шага 3) и ARCH-002 (нет родителя → Io); новых дефектов не вносят. reviews.md не менялся."
      ]
    },
    {
      "at": "2026-10-25T23:11:00+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001",
        "SEC-002"
      ],
      "notes": [
        "Режим дельты (FR-2, AC-1, AC-6): сборка кандидата из компонентов шага 3 не ослабляет защиту — финальная проверка canonicalize + Path::starts_with(база) сохранена (компонент вида `C:`, заменяющий путь при push на Windows, даёт Escape). Секция reviews.md не менялась."
      ]
    },
    {
      "at": "2026-10-25T23:11:00+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Дельта FR-2 шаг 4, AC-1, AC-6: закрывает NOTICE QA-001 (кандидат собирается из компонентов шага 3) и QA-002 (критерий Display в AC-6); новых находок нет.",
        "SC-001 дополнен href `img\\a.png` (ожидаются байты img/a.png, бюджет не меняется); SC-009 уже проверяет новый критерий AC-6."
      ]
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T23:11:01+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T23:11:02+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T23:11:02+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T23:11:02+00:00",
      "event": "note",
      "text": "FD APPROVED; диспатч backend"
    },
    {
      "at": "2026-10-25T23:14:57+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: Cargo.lock, Cargo.toml, crates/nimmd-assets/Cargo.toml, crates/nimmd-assets/src/lib.rs",
        "cargo fmt --check: ok; cargo build/clippy --workspace --all-targets -D warnings: ok; cargo test --workspace --all-targets: all green (новых тестов нет — этап backend-tests)",
        "временный smoke-тест AC-1/AC-2/AC-3(симлинк)/AC-4(каталог, missing) прошёл и удалён; tests/assets.rs пишет afd-test-developer",
        "cargo deny check: advisories/bans/licenses/sources ok (запуск вне песочницы: lock в ~/.cargo read-only); cargo machete: без находок; semgrep .semgrep.yml: 0 находок (вне песочницы: ~/.semgrep/semgrep.log)",
        "diff-lint backend: clippy-changed clean, cargo-machete clean, semgrep-project-rules clean",
        "git grep unsafe|unwrap()|expect( в crates/nimmd-assets/src: только #![forbid(unsafe_code)]",
        "SEC-001: в doc-комментарии AssetBudget указано, что повторные чтения не ограничены и хост кэширует/считает ссылки",
        "SEC-002: в doc-комментарии read_asset указано, что SVG отдаётся как есть и хост показывает его только как <img>/data URI; фиксация в задаче хоста — вне этого этапа",
        "just lint целиком не запускался (vendor/npm-licenses/gitleaks/workflows не затронуты правкой)",
        "migrations: нет; isolation: нет; arch_map: нет",
        "ponytail: 0/0 accepted"
      ]
    },
    {
      "at": "2026-10-25T23:14:57+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T23:14:57+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T23:14:57+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 4,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ]
    },
    {
      "at": "2026-10-25T23:15:03+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T23:15:04+00:00",
      "event": "note",
      "text": "диспатч post-checks"
    },
    {
      "at": "2026-10-25T23:25:18+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Все 4 файла из changed-files.txt связаны с FR-1..FR-4; git diff по Cargo.toml/Cargo.lock — только members и запись пакета без зависимостей.",
        "cargo fmt --check и cargo clippy -p nimmd-assets --all-targets -D warnings — чисто; grep unsafe/unwrap()/expect( в src — только #![forbid(unsafe_code)].",
        "crates/nimmd-assets/tests/assets.rs из Карты реализации ещё не создан — тесты за QA/тест-этапом, на этом ревью не проверялись.",
        "semgrep локально не запустился: sandbox запретил запись ~/.semgrep/semgrep.log; правило nimmd-core-no-io на nimmd-assets не распространяется (include /crates/nimmd-core/src/**)."
      ]
    },
    {
      "at": "2026-10-25T23:25:18+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-003"
      ],
      "notes": [
        "cargo deny advisories не прогнан: песочница запрещает lock ~/.cargo/advisory-dbs/db.lock; у nimmd-assets нет зависимостей",
        "semgrep запущен с HOME=tmp_dir (лог ~/.semgrep запрещён песочницей), exit 0, находок нет"
      ]
    },
    {
      "at": "2026-10-25T23:25:18+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Coverage: SC-001 reads_images_next_to_document_and_counts_each_file_once; SC-002 rejects_before_touching_filesystem; SC-003 symlink_inside_directory_leading_out_escapes; SC-004 document_symlink_resolves_base_to_target_directory; SC-005 only_regular_image_files_are_read + symlink_with_image_name_to_non_image_is_not_image (unix); SC-006 file_size_limit; SC-007 budget_counts_unique_files; SC-009 errors_display_distinct_russian_text; SC-008 static: git grep gives only lib.rs:5 #![forbid(unsafe_code)]; SC-010 cargo test --workspace --all-targets green",
        "Edge cases: rejection_edge_cases (broken %, encoded ':'/'/'/'~', uppercase scheme, '..' deep, not-scheme '1a:'/':x'), decodes_once_not_twice (%252e is a literal file name), budget_boundary_is_inclusive (4x MAX_ASSET_BYTES == MAX_BUDGET_BYTES is ok, the next byte is BudgetExceeded, empty file ok), symlink_to_image_inside_directory_is_counted_once, missing_document_or_directory_is_io; differential percent_encoding_every_byte_does_not_change_verdict (LCG seed 0x6e_696d_6d64, 2000 cases, reference = same href without encoding)",
        "Red evidence: a new crate (feature), so stashing the production code only gives a compile failure; instead I ran mutations on lib.rs (backup in tmp_dir, restored, shasum -c OK) with cargo test -p nimmd-assets --test assets. All 8 caught: drop starts_with(base) -> symlink_inside_directory_leading_out_escapes 'left: Ok([115,...]) right: Err(Escape)'; known=false -> 4 tests 'left: 101 right: 81'; budget '>' -> '>=' -> budget_boundary_is_inclusive; TooLarge '>' -> '>=' -> file_size_limit; split on '/' only -> rejects_before_touching_filesystem 'left: Err(Io) right: Err(Escape)'; non-canonical doc -> document_symlink_resolves_base_to_target_directory; drop is_file -> only_regular_image_files_are_read 'left: Err(Io) right: Err(NotImage)'; double decode -> decodes_once_not_twice",
        "Checks: cargo fmt --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -D warnings OK; cargo test --workspace --all-targets: all green, 0 ignored (nimmd-assets 15 passed); diff-lint backend exit 0 (clippy-changed clean, machete clean, semgrep clean); cargo machete clean",
        "Not run: cargo deny check — the sandbox blocks the lock on ~/.cargo/advisory-dbs (read-only path); my test files add no dependencies. The full just lint was not run (vendor/deny need network and write access outside the sandbox). semgrep only runs with SEMGREP_LOG_FILE/SEMGREP_SETTINGS_FILE pointed at tmp_dir (the sandbox does not allow writing ~/.semgrep)",
        "Unix tests (SC-003, SC-004, symlinks) run on macOS; on Windows they are compiled out by #[cfg(unix)] as SC-003/004/005 prescribe. The tests write about 70 MB of fixtures to target/tmp/nimmd-assets (CARGO_TARGET_TMPDIR)",
        "SEC-001/SEC-002 (NOTICE): a host concern; the implementation's doc comments cover them, and the tests check that repeat reads do not consume budget (budget_counts_unique_files)"
      ]
    },
    {
      "at": "2026-10-25T23:25:18+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный набор не перезапускал: control/suite-run.backend-tests.json result=green, tree 4f38b89445b9ebd371dfa91070d20e2fe0d22c7e = текущий tree-hash.sh.",
        "SC-001..SC-007, SC-009: cargo test -p nimmd-assets — 15 passed, 0 failed, 0 ignored (unix-тесты симлинков выполнены на macOS, не пропущены).",
        "SC-008: git grep без --untracked даёт 0 совпадений — crates/nimmd-assets/ ещё не в индексе; git grep --untracked -n \"unsafe\\|unwrap()\\|expect(\" crates/nimmd-assets/src → только lib.rs:5 #![forbid(unsafe_code)].",
        "SC-010: just lint по шагам — fmt-check, clippy, semgrep (лог semgrep перенаправлен в tmp_dir через SEMGREP_LOG_FILE), unused-deps (machete: нет неиспользуемых), secrets (no leaks), workflows — rc=0 в песочнице; deny, vendor, npm-licenses — rc=0 вне песочницы (песочница запрещает lock ~/.cargo/advisory-dbs и ~/.semgrep). deny: advisories ok, bans ok, licenses ok, sources ok — закрывает SEC-003.",
        "вне объёма: cargo deny предупреждает «unmatched license exception» для cbindgen, deny.toml:28.",
        "Наблюдение без влияния на вердикт: между canonicalize и File::open остаётся окно TOCTOU (подмена симлинка); AC его не покрывают."
      ]
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T23:25:19+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T23:25:54+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "5e03783ff42a90978108f945611b7ec256c809e75aa96864bd7d501346e06b19",
      "sections": {
        "Метаданные": "97f9bb3e4ce2103bc236d3acd752984502db2d949fcfeba20765e81b05175ecc",
        "Цель": "1542a9ea8ec014f27357ff2407a0be57c392eba8c3f462c5f31b6a85112f5f0e",
        "Границы": "252d4421df805e6c8806fe4e891c3a5c0e8d997ca69d751190d6f83362b34342",
        "Целевое поведение": "f04b22609ff83286871ec2912454f07fea68270dc1fd21fd50fe02c556fa10d5",
        "Требования": "a7fc45e60a6917a72a894f3d04a52f38048ff979bb3ca3f0f77e11b23333c2b1",
        "Данные, права, ошибки": "1714defb550ce15eef44b0b6be23a97ddfaf8f13bb35148e7d1f74dbfbbf01c9",
        "Карта реализации": "91d93dfa42e78358033d368f6dffb09610bf8ab7c3b75994852bf800f4b7525b",
        "Критерии приёмки": "281bd8b090b8490e84a22b682c1cde56ad79bc2e3f3866d4483591339902f242",
        "Влияние на руководство": "a614aff15bd3e021662dbd40b449337bd753b2ed926b6860f2aaaae3a0729a72"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "9cacfdc93d847cedbdf281e7b024c8892525ba9c52218f747b2edb0e101ba068"
        },
        "Цель": {
          "Цель": "9c803cce7cafba815a233ab3e3d0a26ed09518931596e865484121878384ad06"
        },
        "Границы": {
          "Границы": "b2b814ff44d823aff1e15161e3ebbbbe15e44737c29c5397b894dde89f9fe8b1"
        },
        "Целевое поведение": {
          "Целевое поведение": "ec5b3af2b04c9a3767e1b277a86b7c36e5fdea0aaac8305510e1eb03acb6486c"
        },
        "Требования": {
          "FR-1": "8fe003c53c0afeac9c4254db8ec9e11048f42fcdcec927aa29b8595324c64bcd",
          "FR-2": "d4d1eabb94fa6e4445008b110b4066529cac535cdb64632c751ffef08b9ac6e4",
          "FR-3": "2fa86e66ea5769e73962f1b3c84a708bc7abe6ad594218e0245f969cb5eb5288",
          "FR-4": "a318992baf9551e8ae9d4f4a1e6d7c79af894fe3dad153da8739b51a3ea347a5"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "cb4d6b0d63aac6341afef652530cc6263a116c755ded9b1069ff0d9d2fcfd32a"
        },
        "Карта реализации": {
          "Карта реализации": "9f5466d2a20e89abc029dfc047029f0d73b3c64e0423fc76c4f746349a7b8e6a"
        },
        "Критерии приёмки": {
          "AC-1": "0d6b42bddfc1fcdc622ec475e0ede733bbe689bf7f14c82f4c5d01301e793f91",
          "AC-2": "61f62b3f27027ac0a13b8e8c132f79f47a24e3051b5c277d6281eea8c4bca1a0",
          "AC-3": "1788214234de406273858fcb32847256458ec334ae3f6250260a7a1f0663b241",
          "AC-4": "7f24ead6fbacfd20de856eb57d4b1dc44e9f3391fef6eab6afa0d7a25fabd688",
          "AC-5": "4302b43991d1672966f58b76b3a549d57645082f3a9ab952433beda50939415f",
          "AC-6": "d947b0019003b66f28ce261a4e0932630f3207241b2e7ae1b7c8df6938351e00"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "ed6740e4999367a4f639821176cccc463634993bbcf6f0b9c31f904c59dcbf9e"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "5e03783ff42a90978108f945611b7ec256c809e75aa96864bd7d501346e06b19",
      "sections": {
        "Метаданные": "97f9bb3e4ce2103bc236d3acd752984502db2d949fcfeba20765e81b05175ecc",
        "Цель": "1542a9ea8ec014f27357ff2407a0be57c392eba8c3f462c5f31b6a85112f5f0e",
        "Границы": "252d4421df805e6c8806fe4e891c3a5c0e8d997ca69d751190d6f83362b34342",
        "Целевое поведение": "f04b22609ff83286871ec2912454f07fea68270dc1fd21fd50fe02c556fa10d5",
        "Требования": "a7fc45e60a6917a72a894f3d04a52f38048ff979bb3ca3f0f77e11b23333c2b1",
        "Данные, права, ошибки": "1714defb550ce15eef44b0b6be23a97ddfaf8f13bb35148e7d1f74dbfbbf01c9",
        "Карта реализации": "91d93dfa42e78358033d368f6dffb09610bf8ab7c3b75994852bf800f4b7525b",
        "Критерии приёмки": "281bd8b090b8490e84a22b682c1cde56ad79bc2e3f3866d4483591339902f242",
        "Влияние на руководство": "a614aff15bd3e021662dbd40b449337bd753b2ed926b6860f2aaaae3a0729a72"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "9cacfdc93d847cedbdf281e7b024c8892525ba9c52218f747b2edb0e101ba068"
        },
        "Цель": {
          "Цель": "9c803cce7cafba815a233ab3e3d0a26ed09518931596e865484121878384ad06"
        },
        "Границы": {
          "Границы": "b2b814ff44d823aff1e15161e3ebbbbe15e44737c29c5397b894dde89f9fe8b1"
        },
        "Целевое поведение": {
          "Целевое поведение": "ec5b3af2b04c9a3767e1b277a86b7c36e5fdea0aaac8305510e1eb03acb6486c"
        },
        "Требования": {
          "FR-1": "8fe003c53c0afeac9c4254db8ec9e11048f42fcdcec927aa29b8595324c64bcd",
          "FR-2": "d4d1eabb94fa6e4445008b110b4066529cac535cdb64632c751ffef08b9ac6e4",
          "FR-3": "2fa86e66ea5769e73962f1b3c84a708bc7abe6ad594218e0245f969cb5eb5288",
          "FR-4": "a318992baf9551e8ae9d4f4a1e6d7c79af894fe3dad153da8739b51a3ea347a5"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "cb4d6b0d63aac6341afef652530cc6263a116c755ded9b1069ff0d9d2fcfd32a"
        },
        "Карта реализации": {
          "Карта реализации": "9f5466d2a20e89abc029dfc047029f0d73b3c64e0423fc76c4f746349a7b8e6a"
        },
        "Критерии приёмки": {
          "AC-1": "0d6b42bddfc1fcdc622ec475e0ede733bbe689bf7f14c82f4c5d01301e793f91",
          "AC-2": "61f62b3f27027ac0a13b8e8c132f79f47a24e3051b5c277d6281eea8c4bca1a0",
          "AC-3": "1788214234de406273858fcb32847256458ec334ae3f6250260a7a1f0663b241",
          "AC-4": "7f24ead6fbacfd20de856eb57d4b1dc44e9f3391fef6eab6afa0d7a25fabd688",
          "AC-5": "4302b43991d1672966f58b76b3a549d57645082f3a9ab952433beda50939415f",
          "AC-6": "d947b0019003b66f28ce261a4e0932630f3207241b2e7ae1b7c8df6938351e00"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "ed6740e4999367a4f639821176cccc463634993bbcf6f0b9c31f904c59dcbf9e"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "5e03783ff42a90978108f945611b7ec256c809e75aa96864bd7d501346e06b19",
      "sections": {
        "Метаданные": "97f9bb3e4ce2103bc236d3acd752984502db2d949fcfeba20765e81b05175ecc",
        "Цель": "1542a9ea8ec014f27357ff2407a0be57c392eba8c3f462c5f31b6a85112f5f0e",
        "Границы": "252d4421df805e6c8806fe4e891c3a5c0e8d997ca69d751190d6f83362b34342",
        "Целевое поведение": "f04b22609ff83286871ec2912454f07fea68270dc1fd21fd50fe02c556fa10d5",
        "Требования": "a7fc45e60a6917a72a894f3d04a52f38048ff979bb3ca3f0f77e11b23333c2b1",
        "Данные, права, ошибки": "1714defb550ce15eef44b0b6be23a97ddfaf8f13bb35148e7d1f74dbfbbf01c9",
        "Карта реализации": "91d93dfa42e78358033d368f6dffb09610bf8ab7c3b75994852bf800f4b7525b",
        "Критерии приёмки": "281bd8b090b8490e84a22b682c1cde56ad79bc2e3f3866d4483591339902f242",
        "Влияние на руководство": "a614aff15bd3e021662dbd40b449337bd753b2ed926b6860f2aaaae3a0729a72"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "9cacfdc93d847cedbdf281e7b024c8892525ba9c52218f747b2edb0e101ba068"
        },
        "Цель": {
          "Цель": "9c803cce7cafba815a233ab3e3d0a26ed09518931596e865484121878384ad06"
        },
        "Границы": {
          "Границы": "b2b814ff44d823aff1e15161e3ebbbbe15e44737c29c5397b894dde89f9fe8b1"
        },
        "Целевое поведение": {
          "Целевое поведение": "ec5b3af2b04c9a3767e1b277a86b7c36e5fdea0aaac8305510e1eb03acb6486c"
        },
        "Требования": {
          "FR-1": "8fe003c53c0afeac9c4254db8ec9e11048f42fcdcec927aa29b8595324c64bcd",
          "FR-2": "d4d1eabb94fa6e4445008b110b4066529cac535cdb64632c751ffef08b9ac6e4",
          "FR-3": "2fa86e66ea5769e73962f1b3c84a708bc7abe6ad594218e0245f969cb5eb5288",
          "FR-4": "a318992baf9551e8ae9d4f4a1e6d7c79af894fe3dad153da8739b51a3ea347a5"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "cb4d6b0d63aac6341afef652530cc6263a116c755ded9b1069ff0d9d2fcfd32a"
        },
        "Карта реализации": {
          "Карта реализации": "9f5466d2a20e89abc029dfc047029f0d73b3c64e0423fc76c4f746349a7b8e6a"
        },
        "Критерии приёмки": {
          "AC-1": "0d6b42bddfc1fcdc622ec475e0ede733bbe689bf7f14c82f4c5d01301e793f91",
          "AC-2": "61f62b3f27027ac0a13b8e8c132f79f47a24e3051b5c277d6281eea8c4bca1a0",
          "AC-3": "1788214234de406273858fcb32847256458ec334ae3f6250260a7a1f0663b241",
          "AC-4": "7f24ead6fbacfd20de856eb57d4b1dc44e9f3391fef6eab6afa0d7a25fabd688",
          "AC-5": "4302b43991d1672966f58b76b3a549d57645082f3a9ab952433beda50939415f",
          "AC-6": "d947b0019003b66f28ce261a4e0932630f3207241b2e7ae1b7c8df6938351e00"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "ed6740e4999367a4f639821176cccc463634993bbcf6f0b9c31f904c59dcbf9e"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "5e03783ff42a90978108f945611b7ec256c809e75aa96864bd7d501346e06b19",
    "backend_tests": "5e03783ff42a90978108f945611b7ec256c809e75aa96864bd7d501346e06b19"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства нет; правила описывают doc-комментарии крейта."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "da98233f8bacb99f614d7d1c68b8ebc95245f9060f2ed736b96b371d49d6c5c3",
      "files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-assets/Cargo.toml",
        "crates/nimmd-assets/src/lib.rs",
        "crates/nimmd-assets/tests/assets.rs"
      ],
      "production_hash": "96e135f596dba88bfdef118bf1ec73e48bf0845d8092cb3776108e68ccbdba5b",
      "production_files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-assets/Cargo.toml",
        "crates/nimmd-assets/src/lib.rs"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "da98233f8bacb99f614d7d1c68b8ebc95245f9060f2ed736b96b371d49d6c5c3",
      "files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-assets/Cargo.toml",
        "crates/nimmd-assets/src/lib.rs",
        "crates/nimmd-assets/tests/assets.rs"
      ],
      "production_hash": "96e135f596dba88bfdef118bf1ec73e48bf0845d8092cb3776108e68ccbdba5b",
      "production_files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-assets/Cargo.toml",
        "crates/nimmd-assets/src/lib.rs"
      ]
    }
  },
  "accepted_code_hash": {
    "backend-tests": "da98233f8bacb99f614d7d1c68b8ebc95245f9060f2ed736b96b371d49d6c5c3",
    "testing": "da98233f8bacb99f614d7d1c68b8ebc95245f9060f2ed736b96b371d49d6c5c3"
  }
}
```
<!-- afd-state-snapshot:end -->
