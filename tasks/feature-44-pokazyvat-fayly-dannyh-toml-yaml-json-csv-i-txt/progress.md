# Состояние разработки: Показывать файлы данных TOML, YAML, JSON, CSV и TXT

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #44 (syper-freedm/NimMD)
- Размер: M; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T23:04:13+00:00

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

- 2026-10-25T22:43:59+00:00 `routed` — {"post_checks_parallel": false, "files_count": 17, "has_migrations": false, "touches_isolation": false, "skip": ["documentation"]}
- 2026-10-25T22:44:07+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-25T22:44:07+00:00 `note` — {"text": "диспатч post-checks"}
- 2026-10-25T23:03:53+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": [], "notes": ["deny.toml вне карты реализации, но нужен для FR-4/AC-10 (RUSTSEC-2024-0320 yaml-rust только в dev-dependencies через yaml-load)", "semgrep локально не запущен: песочница запрещает запись ~/.semgrep/semgrep.log; just lint и just syntaxes (воспроизводимость дампа) не прогонялись — за тест-исполнителем"]}
- 2026-10-25T23:03:53+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": [], "notes": ["cargo deny check: advisories ok, bans ok, licenses ok, sources ok (запуск вне песочницы: lock базы RustSec в ~/.cargo read-only)", "semgrep (.semgrep.yml + p/rust p/typescript p/javascript p/secrets) по 8 изменённым .rs: 0 находок; лог semgrep перенаправлен в tmp_dir", "gitleaks dir по файлам из changed-files.txt: no leaks found", "yaml-rust 0.4.5 (RUSTSEC-2024-0320, unmaintained) нет в cargo tree -e normal ядра: только dev-dependencies для examples/build_syntaxes.rs", "Усиление HTML в head до 2 МБ (CSV из одних разделителей, около x23) — ограничено HIGHLIGHT_LIMIT по дизайну, находкой не оформлено"]}
- 2026-10-25T23:03:54+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Coverage: SC-001 file_kind_from_extension, file_kind_serde_lowercase; SC-002 toml_yaml_json_highlighted_in_data_wrapper, markdown_has_no_data_css; SC-003 json_file_classes_match_markdown_block (+ existing insta snapshots, git status crates/nimmd-core/tests/snapshots is empty, no .snap.new); SC-004 text_escaped_without_highlight; SC-005 data_has_no_profile_frontmatter_or_scripts (Link and Inline assets, 4 kinds); SC-006 highlight_limit_splits_head_and_tail, highlight_limit_exact_boundary, long_line_without_newline_cut_at_char_boundary; SC-007 decode_lossy_replaces_invalid_bytes (tests/text.rs), not_utf8_bom_crlf_and_truncation; SC-008 detect_delimiter_cases, detect_delimiter_ties_and_mode, detect_delimiter_reads_first_50_records; SC-009 csv_colored_by_column, csv_cases_keep_text, csv_edge_inputs_keep_text; SC-010 csv_fixtures_keep_text (new fixtures tests/fixtures/data/{semicolon,quoted,tabs,ragged}.csv); SC-012 data_languages_have_spans; SC-013 rust_spans_in_utf16, unknown_and_empty_language, large_code_highlights_only_head, span_serializes_to_json", "Randomized/differential tests with fixed seeds: random_csv_matches_reference (3000 inputs, independent char-level RFC 4180 tokenizer + FR-7 delimiter + FR-8 coloring), random_tables_detect_their_delimiter (500 tables), random_code_matches_reference (400 inputs vs syntect ScopeRangeIterator over load_defaults_newlines), random_data_keeps_text (300 inputs x 5 kinds)", "Red evidence: this is a feature (new symbols), so git stash of production gives compile errors, not assertion failures. Ran mutations instead (backups in control/.tmp/backend-tests/*.orig, restored; shasum of src/*.rs and git diff --stat identical before/after): csv.rs 'records > best.1'->'>=' fails detect_delimiter_ties_and_mode (csv.rs:31), random_csv_matches_reference; highlight.rs split_head 'i + 1'->'i' fails highlight_limit_splits_head_and_tail (data.rs:288), highlight_limit_exact_boundary, large_code_highlights_only_head; spans offset bytes instead of UTF-16 fails rust_spans_in_utf16, random_code_matches_reference; data.rs highlighting TXT fails text_escaped_without_highlight; csv.rs 'column % 8'->'column' fails csv_colored_by_column, csv_cases_keep_text", "Surviving mutant: document.rs 'if markdown { push_assets }' -> 'if true' survives; it is equivalent because data always uses Profile::default() (math/mermaid off), so push_assets writes nothing. Profile equality and no <script/<link are asserted", "Checks run: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings clean; diff-lint.sh backend clean (clippy-changed, cargo-machete, semgrep); semgrep --config .semgrep.yml on the 4 test files: 0 findings; cargo test --workspace --all-targets green, 131 tests, 0 ignored/skipped", "Sandbox: semgrep cannot write ~/.semgrep/semgrep.log. Ran diff-lint/semgrep with SEMGREP_LOG_FILE and SEMGREP_SETTINGS_FILE pointing into control/.tmp/backend-tests, without disabling the sandbox", "Not run: full `just lint` (vendor needs the npm registry; deny and gitleaks git do not cover uncommitted test files). Static AC-10 checks run: just syntaxes prints 'TOML: ok' and the dump is byte-identical (cmp); LICENSE line 1 has commit 72065294da5dfb1862cd43177df4a50e29349135; cargo tree -p nimmd-core -d -e normal shows no duplicates; git grep --untracked for find_syntax_plain_text|from_binary|SyntectAdapter in src is empty", "Note for QA: src/csv.rs, data.rs and highlight.rs are untracked, so the AC-10 `git grep` without --untracked never searches them (vacuously empty). `git status --short crates/nimmd-core/assets` will list ?? until commit. I checked dump stability with cmp against a backup", "Timing: tests/data.rs takes about 13 s in debug, because of the 2 MB JSON/CSV highlight-limit inputs from AC-6"]}
- 2026-10-25T23:03:54+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный прогон не повторён: control/suite-run.backend-tests.json result=green, tree 54d452755887dd88719d2d7c1f2a2813bb092bc4 = текущему tree-hash (до и после проверок).", "Фокусно: cargo test -p nimmd-core --test data --test csv --test highlight --test text — 42 passed, 0 failed, 0 ignored; --test render --test document — 53 passed (снапшоты Markdown без .snap.new, git status tests/snapshots пуст).", "SC-003: cargo-insta не установлен — вместо cargo insta test прогнан cargo test тех же тестов insta (несовпадение снапшота роняет assert) + проверка отсутствия .snap.new.", "SC-011: just syntaxes — 'TOML: ok', shasum syntaxes.packdump до/после совпал (assets ещё не в git, поэтому сравнение по хэшу); первая строка LICENSE — commit 72065294…, тот же хэш есть в FD; cargo tree -p nimmd-core -d -e normal — 'nothing to print'; git grep --untracked (новые src-файлы не в индексе) — 0 совпадений.", "SC-012: just lint зелёный по рецептам: fmt-check, clippy, semgrep (SEMGREP_LOG_FILE/SETTINGS направлены в tmp_dir — песочница запрещает запись в ~/.semgrep), deny и vendor — вне песочницы (lock в ~/.cargo/advisory-dbs и pnpm store только для чтения), npm-licenses, unused-deps, secrets, workflows — в песочнице. cargo deny: только старые предупреждения license-exception-not-encountered.", "gitleaks dir по каждому файлу правки и тестов — 0 находок (gitleaks git не видит незакоммиченную правку)."]}
- 2026-10-25T23:03:54+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-25T23:03:54+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T23:03:54+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T23:03:54+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T23:03:54+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T23:03:54+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T23:03:54+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T23:04:13+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 44,
  "title": "Показывать файлы данных TOML, YAML, JSON, CSV и TXT",
  "slug": "pokazyvat-fayly-dannyh-toml-yaml-json-csv-i-txt",
  "branch": "feat/44-pokazyvat-fayly-dannyh-toml-yaml-json",
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
      "at": "2026-10-25T22:23:30+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T22:26:14+00:00",
      "event": "note",
      "text": "шаг 1 делегирован: развилок нет — спек 5.7 и план Task 5 задают поведение; decode_lossy оставлен (интерфейс плана для FFI #49); К21 снят пробой: TOML sublimehq@72065294 грузится syntect 5.3"
    },
    {
      "at": "2026-10-25T22:26:14+00:00",
      "event": "classified",
      "size": "M",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T22:27:22+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T22:27:22+00:00",
      "event": "note",
      "text": "диспатч fd-review"
    },
    {
      "at": "2026-10-25T22:31:28+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002"
      ],
      "notes": [
        "architecture.md не создан: новые публичные контракты зафиксированы в ФД, отдельное решение записывать не нужно"
      ]
    },
    {
      "at": "2026-10-25T22:31:28+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Прав, тенантов, секретов и сетевых точек входа нет: ядро — чистая функция строка -> HTML; security.md не нужен.",
        "XSS: текст данных и поля CSV экранируются, атрибуты (data-kind, c{N mod 8}, d) — константы ядра; AC-4 и AC-9 проверяют экранирование.",
        "Ресурсы (API4): вход ограничен MAX_BYTES, подсветка — HIGHLIGHT_LIMIT 2 МБ; fancy-regex при катастрофическом бэктрекинге даёт не-совпадение (сверено в syntect-5.3.0 src/parsing/regex.rs), CSV-автомат линеен.",
        "bincode (RUSTSEC-2025-0141) декодирует только встроенный дамп; исключение уже есть в deny.toml; воспроизводимость дампа проверяет AC-10."
      ]
    },
    {
      "at": "2026-10-25T22:31:28+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "AC-1…AC-11 проверяемы; test-cases.md: SC-001…SC-013, validate-tests OK, 2596/2700 токенов",
        "QA-001 NOTICE: мода FR-7 при ничьей не определена; QA-002 NOTICE: лимит подсветки CSV без AC, у AC-10 нет входа для highlight",
        "Контроля доступа в ядре нет — объектных сценариев доступа нет; docs/licenses/audit-2026-10-23.md (gitignored) не открывал"
      ]
    },
    {
      "at": "2026-10-25T22:31:28+00:00",
      "event": "note",
      "text": "NOTICE ARCH-001/002, QA-001/002 вскрывают пробелы (карта, неопределённая мода, непроверенный лимит CSV) → одна правка ФД"
    },
    {
      "at": "2026-10-25T22:31:28+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "requirements_rework",
      "reason": "ARCH-001, ARCH-002, QA-001, QA-002: правка ФД пачкой"
    },
    {
      "at": "2026-10-25T22:32:14+00:00",
      "event": "transition",
      "from": "requirements_rework",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T22:32:25+00:00",
      "event": "note",
      "text": "confirm-круг FD после правки NOTICE"
    },
    {
      "at": "2026-10-25T22:33:36+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Режим дельты: дельта закрывает ARCH-001 (фичи syntect в [workspace.dependencies], Cargo.toml в карте) и ARCH-002 (from_uncompressed_data + dump_to_uncompressed_file); сверено с syntect 5.3.0: parsing включает dump-load/dump-create, yaml-load включает parsing.",
        "Остальные правки (FR-7 ничья моды, FR-10 только Serialize, AC-6 CSV, AC-10 входы, формулировки Цели/Границ/Данных) новых архитектурных дефектов не вносят; reviews.md оставлен без изменений."
      ]
    },
    {
      "at": "2026-10-25T22:33:36+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Режим delta: проверены только FR-4, FR-7, FR-10, FR-11, AC-6, AC-10 и раздел «Данные, права, ошибки»; новых находок нет, секция reviews.md не менялась.",
        "FR-4: from_uncompressed_data в syntect 5.3 под фичей dump-load, которую включает parsing; возвращает Result — путь без паники сохранён; дамп встроен в бинарь, внешнего входа нет.",
        "«Данные, права, ошибки»: перечень краевых входов сокращён, но требования тотальности, экранирования и запрета unwrap/expect остались; std::fs по-прежнему только в examples/."
      ]
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Дельта: QA-001 (мода при ничьей — FR-7) и QA-002 (CSV в AC-6, входы AC-10) закрыты; SC-006 и SC-012 обновлены под новые входы",
        "FR-4: syntect 5.3 `parsing` включает `dump-load`/`dump-create`, `from_uncompressed_data` и `dump_to_uncompressed_file` есть — правка согласована",
        "AC-6 `Действие` не называет `Csv` — SC-006 рендерит CSV с `Csv`; не блокирует"
      ]
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T22:33:37+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:33:38+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:33:38+00:00",
      "event": "note",
      "text": "FD APPROVED после confirm; диспатч backend"
    },
    {
      "at": "2026-10-25T22:43:59+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "files: see control/changed-files.backend.txt (17 paths, same as changed_files)",
        "cargo fmt --check, cargo build --workspace --all-targets, cargo clippy -D warnings: green",
        "cargo test --workspace --all-targets: all existing suites ok (2+27+32+7+21+14 passed); tests/snapshots unchanged",
        "cargo deny check: advisories ok, bans ok, licenses ok, sources ok (run outside sandbox: advisory DB lock in ~/.cargo is read-only in sandbox)",
        "deny.toml: added ignore RUSTSEC-2024-0320 (yaml-rust unmaintained) - introduced by FR-4 dev feature yaml-load, dev-only, used by examples/build_syntaxes.rs; not in product build",
        "just syntaxes: 'TOML: ok'; dump deterministic (3 runs, same sha1 27323c58...); cargo tree -p nimmd-core -d -e normal: no duplicates; AC-10 git grep (find_syntax_plain_text|from_binary|SyntectAdapter) in src: empty",
        "just semgrep, cargo machete: green; diff-lint backend: exit 0, no findings (semgrep needed run outside sandbox: ~/.semgrep log)",
        "just lint vendor/npm-licenses/secrets/workflows not run: unchanged areas; vendor needs npm registry",
        "TOML.sublime-syntax and LICENSE fetched via unauthenticated curl api.github.com at commit 72065294 (gh CLI config blocked by sandbox); LICENSE first line = source + commit hash",
        "syntect normal features: html, parsing, regex-fancy (parsing already enables dump-load for from_uncompressed_data); dev adds default-syntaxes, yaml-load",
        "CSV quoted field: after closing quote the field continues to the next delimiter/newline (e.g. \"a\"b,c) so visible text always equals input",
        "data.css: contrast measured against worst of --bg and --bg-subtle (pre background); min 4.63:1 for colors, .d 5.74/5.94:1",
        "smoke-checked AC-2,4,5,6,8,9,11 outputs via scratch binary in tmp_dir (removed); no new tests written",
        "ponytail: 0/0 accepted"
      ]
    },
    {
      "at": "2026-10-25T22:43:59+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T22:43:59+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T22:43:59+00:00",
      "event": "routed",
      "post_checks_parallel": false,
      "files_count": 17,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ]
    },
    {
      "at": "2026-10-25T22:44:07+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T22:44:07+00:00",
      "event": "note",
      "text": "диспатч post-checks"
    },
    {
      "at": "2026-10-25T23:03:53+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "deny.toml вне карты реализации, но нужен для FR-4/AC-10 (RUSTSEC-2024-0320 yaml-rust только в dev-dependencies через yaml-load)",
        "semgrep локально не запущен: песочница запрещает запись ~/.semgrep/semgrep.log; just lint и just syntaxes (воспроизводимость дампа) не прогонялись — за тест-исполнителем"
      ]
    },
    {
      "at": "2026-10-25T23:03:53+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "cargo deny check: advisories ok, bans ok, licenses ok, sources ok (запуск вне песочницы: lock базы RustSec в ~/.cargo read-only)",
        "semgrep (.semgrep.yml + p/rust p/typescript p/javascript p/secrets) по 8 изменённым .rs: 0 находок; лог semgrep перенаправлен в tmp_dir",
        "gitleaks dir по файлам из changed-files.txt: no leaks found",
        "yaml-rust 0.4.5 (RUSTSEC-2024-0320, unmaintained) нет в cargo tree -e normal ядра: только dev-dependencies для examples/build_syntaxes.rs",
        "Усиление HTML в head до 2 МБ (CSV из одних разделителей, около x23) — ограничено HIGHLIGHT_LIMIT по дизайну, находкой не оформлено"
      ]
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Coverage: SC-001 file_kind_from_extension, file_kind_serde_lowercase; SC-002 toml_yaml_json_highlighted_in_data_wrapper, markdown_has_no_data_css; SC-003 json_file_classes_match_markdown_block (+ existing insta snapshots, git status crates/nimmd-core/tests/snapshots is empty, no .snap.new); SC-004 text_escaped_without_highlight; SC-005 data_has_no_profile_frontmatter_or_scripts (Link and Inline assets, 4 kinds); SC-006 highlight_limit_splits_head_and_tail, highlight_limit_exact_boundary, long_line_without_newline_cut_at_char_boundary; SC-007 decode_lossy_replaces_invalid_bytes (tests/text.rs), not_utf8_bom_crlf_and_truncation; SC-008 detect_delimiter_cases, detect_delimiter_ties_and_mode, detect_delimiter_reads_first_50_records; SC-009 csv_colored_by_column, csv_cases_keep_text, csv_edge_inputs_keep_text; SC-010 csv_fixtures_keep_text (new fixtures tests/fixtures/data/{semicolon,quoted,tabs,ragged}.csv); SC-012 data_languages_have_spans; SC-013 rust_spans_in_utf16, unknown_and_empty_language, large_code_highlights_only_head, span_serializes_to_json",
        "Randomized/differential tests with fixed seeds: random_csv_matches_reference (3000 inputs, independent char-level RFC 4180 tokenizer + FR-7 delimiter + FR-8 coloring), random_tables_detect_their_delimiter (500 tables), random_code_matches_reference (400 inputs vs syntect ScopeRangeIterator over load_defaults_newlines), random_data_keeps_text (300 inputs x 5 kinds)",
        "Red evidence: this is a feature (new symbols), so git stash of production gives compile errors, not assertion failures. Ran mutations instead (backups in control/.tmp/backend-tests/*.orig, restored; shasum of src/*.rs and git diff --stat identical before/after): csv.rs 'records > best.1'->'>=' fails detect_delimiter_ties_and_mode (csv.rs:31), random_csv_matches_reference; highlight.rs split_head 'i + 1'->'i' fails highlight_limit_splits_head_and_tail (data.rs:288), highlight_limit_exact_boundary, large_code_highlights_only_head; spans offset bytes instead of UTF-16 fails rust_spans_in_utf16, random_code_matches_reference; data.rs highlighting TXT fails text_escaped_without_highlight; csv.rs 'column % 8'->'column' fails csv_colored_by_column, csv_cases_keep_text",
        "Surviving mutant: document.rs 'if markdown { push_assets }' -> 'if true' survives; it is equivalent because data always uses Profile::default() (math/mermaid off), so push_assets writes nothing. Profile equality and no <script/<link are asserted",
        "Checks run: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings clean; diff-lint.sh backend clean (clippy-changed, cargo-machete, semgrep); semgrep --config .semgrep.yml on the 4 test files: 0 findings; cargo test --workspace --all-targets green, 131 tests, 0 ignored/skipped",
        "Sandbox: semgrep cannot write ~/.semgrep/semgrep.log. Ran diff-lint/semgrep with SEMGREP_LOG_FILE and SEMGREP_SETTINGS_FILE pointing into control/.tmp/backend-tests, without disabling the sandbox",
        "Not run: full `just lint` (vendor needs the npm registry; deny and gitleaks git do not cover uncommitted test files). Static AC-10 checks run: just syntaxes prints 'TOML: ok' and the dump is byte-identical (cmp); LICENSE line 1 has commit 72065294da5dfb1862cd43177df4a50e29349135; cargo tree -p nimmd-core -d -e normal shows no duplicates; git grep --untracked for find_syntax_plain_text|from_binary|SyntectAdapter in src is empty",
        "Note for QA: src/csv.rs, data.rs and highlight.rs are untracked, so the AC-10 `git grep` without --untracked never searches them (vacuously empty). `git status --short crates/nimmd-core/assets` will list ?? until commit. I checked dump stability with cmp against a backup",
        "Timing: tests/data.rs takes about 13 s in debug, because of the 2 MB JSON/CSV highlight-limit inputs from AC-6"
      ]
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный прогон не повторён: control/suite-run.backend-tests.json result=green, tree 54d452755887dd88719d2d7c1f2a2813bb092bc4 = текущему tree-hash (до и после проверок).",
        "Фокусно: cargo test -p nimmd-core --test data --test csv --test highlight --test text — 42 passed, 0 failed, 0 ignored; --test render --test document — 53 passed (снапшоты Markdown без .snap.new, git status tests/snapshots пуст).",
        "SC-003: cargo-insta не установлен — вместо cargo insta test прогнан cargo test тех же тестов insta (несовпадение снапшота роняет assert) + проверка отсутствия .snap.new.",
        "SC-011: just syntaxes — 'TOML: ok', shasum syntaxes.packdump до/после совпал (assets ещё не в git, поэтому сравнение по хэшу); первая строка LICENSE — commit 72065294…, тот же хэш есть в FD; cargo tree -p nimmd-core -d -e normal — 'nothing to print'; git grep --untracked (новые src-файлы не в индексе) — 0 совпадений.",
        "SC-012: just lint зелёный по рецептам: fmt-check, clippy, semgrep (SEMGREP_LOG_FILE/SETTINGS направлены в tmp_dir — песочница запрещает запись в ~/.semgrep), deny и vendor — вне песочницы (lock в ~/.cargo/advisory-dbs и pnpm store только для чтения), npm-licenses, unused-deps, secrets, workflows — в песочнице. cargo deny: только старые предупреждения license-exception-not-encountered.",
        "gitleaks dir по каждому файлу правки и тестов — 0 находок (gitleaks git не видит незакоммиченную правку)."
      ]
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T23:03:54+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T23:04:13+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "6bbc743f0e620496ebbe88dba70313a12a538bedefcf72bf2fade9a782f0b246",
      "sections": {
        "Метаданные": "32950a5418b487d6c4a265ecf91e5cc04b0f65089a18819981dbc7ebdf32ea99",
        "Цель": "376203472158deaec080b6bc7a06cafd26a676895721aeb9d57cd860f0797f51",
        "Границы": "85ad023602caa7a0d67f2f466f18966c2b33dcf9ab59ed53e25f5bd06bb08c47",
        "Целевое поведение": "4677bd5ed07edb5fe9db1904f24e7da885cf50f95e620d05c31e1d52ca1449ce",
        "Требования": "0e2d4cd5a0b146140aace06d78c2c6515bb6344c903f7fb11142ccf5f3ea8ca9",
        "Данные, права, ошибки": "80e67b596c5953410f4e6012056504d88e48a002d769823193f2d4324e363217",
        "Карта реализации": "29acbc232bb1d2c68fe7323690e87d211bf7f827e28fa85093c2dfb8aad72723",
        "Критерии приёмки": "3e38564014c83ce2121b72ef8a3a7f14a00575126d389db3d9073334fbcc3f00",
        "Влияние на руководство": "d1c06891f123b09c9c97211206ffd2b1d3d8eeccf835effaf1217a798b3a0cc1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "53d0d9a30c6fc4b5654e372096127e194f019c103f65fbf23db7660b9ff0cb4c"
        },
        "Цель": {
          "Цель": "cb96c6cab93b94382a688fff8e2022b32889b879799465c13bb27f871e201480"
        },
        "Границы": {
          "Границы": "efdc9352927e68cef4d6d9183777abeab972f846a3e9793eb20719815e6fae88"
        },
        "Целевое поведение": {
          "Целевое поведение": "5bfc0e9277ee3bd41c3f4d4f65eabc72b8e7d1c9d26da371f11c26204fe2782c"
        },
        "Требования": {
          "FR-1": "e7996446789c3640de696542a3af8c33b4f760587eb04d849adc46ffe97a4984",
          "FR-2": "dac88565db6326cd927d281a34debb80209f7c7178a14b89f43162813e037df1",
          "FR-3": "a0041e0780b40e20186339686ac880a50410008d58cd6710ce2696b37c9cf160",
          "FR-4": "034381f5569b77a6b4e72b3e3837ef04879e1dd210c8bcf3340fd15c5eed0cbf",
          "FR-5": "9eefbc2001a874e15233bff4378698e0e433a045522e2809f37701106a31e1d3",
          "FR-6": "5f15cc83ab33c572805eef30a462264cc76b5d3fa9e1ad32b78134ab97827def",
          "FR-7": "0c5356be14ed7130c9f80ab5b6bdaf62a37c4fa876d68941b7cab88d45f41f7e",
          "FR-8": "1661d907f685f689a2d342c0953d452bfc60470717dd701d0427d2e6158ff3ed",
          "FR-9": "696d5cc06cad309d8fc1d9d533a984ed6c2412d02e4e6a14237c6f3709178551",
          "FR-10": "05b245098fa111c5d372a97986b6d601f814f0cf7f2404bf2c1609b85c89a209",
          "FR-11": "5a4118a236f4dc5206c3c7714e9058a63115c909a35e96c889818a2d91204975"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "a340706c66e4e4efd6796d1b81b506216a38bbc3afefb3842eb59e20ab774965"
        },
        "Карта реализации": {
          "Карта реализации": "527b4d52970085ddc88e9c68e90f39c589e77f053cbd4f1190d23f0bdc99b9ed"
        },
        "Критерии приёмки": {
          "AC-1": "7c1a208a2276fe375f23751575a953899eef7ff330dbe9f4ff3450eada2c7067",
          "AC-2": "acf05954f5bbacaf3e367600808a12fae86bd1cc7b248ab04f2a6fade45f0155",
          "AC-3": "b7867ea1d05c931d7061ff1b77f737b80465fba12c6dd6639b7fbfd0502e91d9",
          "AC-4": "057881d1c4f009e10bdf04d9b10aaee157f55514d95b2264cd1c0a9b502c5446",
          "AC-5": "40f6e5441e32bfbd7fbf962a8b854ab6fbc6577d9d17dfe7ab3a608922a51a84",
          "AC-6": "eb29ed46090881ecf3f33222250d3dd3aaf35626ff4577214a075315bdce9186",
          "AC-7": "d5a62948d4635cf486bf9780ff77797122082e2f08aecf0c7db39a62f4d59840",
          "AC-8": "d21d9988e8fad37e6ecf927590e3be184526b3f65bc6a818688c5f49d9c0defb",
          "AC-9": "7119e5ef722b544eb00d197fb7ef91641d3e020d8a93ddb3bd8ab64a795bec79",
          "AC-10": "99bb896ce463600951005d3ff8320bb7ed61b3a949615527cc4802a793c43628",
          "AC-11": "5f11c8cbb51ad4b0494442ab3491da29e1db7d9288875df8d7072bd240f33122"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "e8364cbf6df91d05dbc36a08f6b3fc8e7f87b84bf33c8b51eca78afacd5dce9f"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "6bbc743f0e620496ebbe88dba70313a12a538bedefcf72bf2fade9a782f0b246",
      "sections": {
        "Метаданные": "32950a5418b487d6c4a265ecf91e5cc04b0f65089a18819981dbc7ebdf32ea99",
        "Цель": "376203472158deaec080b6bc7a06cafd26a676895721aeb9d57cd860f0797f51",
        "Границы": "85ad023602caa7a0d67f2f466f18966c2b33dcf9ab59ed53e25f5bd06bb08c47",
        "Целевое поведение": "4677bd5ed07edb5fe9db1904f24e7da885cf50f95e620d05c31e1d52ca1449ce",
        "Требования": "0e2d4cd5a0b146140aace06d78c2c6515bb6344c903f7fb11142ccf5f3ea8ca9",
        "Данные, права, ошибки": "80e67b596c5953410f4e6012056504d88e48a002d769823193f2d4324e363217",
        "Карта реализации": "29acbc232bb1d2c68fe7323690e87d211bf7f827e28fa85093c2dfb8aad72723",
        "Критерии приёмки": "3e38564014c83ce2121b72ef8a3a7f14a00575126d389db3d9073334fbcc3f00",
        "Влияние на руководство": "d1c06891f123b09c9c97211206ffd2b1d3d8eeccf835effaf1217a798b3a0cc1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "53d0d9a30c6fc4b5654e372096127e194f019c103f65fbf23db7660b9ff0cb4c"
        },
        "Цель": {
          "Цель": "cb96c6cab93b94382a688fff8e2022b32889b879799465c13bb27f871e201480"
        },
        "Границы": {
          "Границы": "efdc9352927e68cef4d6d9183777abeab972f846a3e9793eb20719815e6fae88"
        },
        "Целевое поведение": {
          "Целевое поведение": "5bfc0e9277ee3bd41c3f4d4f65eabc72b8e7d1c9d26da371f11c26204fe2782c"
        },
        "Требования": {
          "FR-1": "e7996446789c3640de696542a3af8c33b4f760587eb04d849adc46ffe97a4984",
          "FR-2": "dac88565db6326cd927d281a34debb80209f7c7178a14b89f43162813e037df1",
          "FR-3": "a0041e0780b40e20186339686ac880a50410008d58cd6710ce2696b37c9cf160",
          "FR-4": "034381f5569b77a6b4e72b3e3837ef04879e1dd210c8bcf3340fd15c5eed0cbf",
          "FR-5": "9eefbc2001a874e15233bff4378698e0e433a045522e2809f37701106a31e1d3",
          "FR-6": "5f15cc83ab33c572805eef30a462264cc76b5d3fa9e1ad32b78134ab97827def",
          "FR-7": "0c5356be14ed7130c9f80ab5b6bdaf62a37c4fa876d68941b7cab88d45f41f7e",
          "FR-8": "1661d907f685f689a2d342c0953d452bfc60470717dd701d0427d2e6158ff3ed",
          "FR-9": "696d5cc06cad309d8fc1d9d533a984ed6c2412d02e4e6a14237c6f3709178551",
          "FR-10": "05b245098fa111c5d372a97986b6d601f814f0cf7f2404bf2c1609b85c89a209",
          "FR-11": "5a4118a236f4dc5206c3c7714e9058a63115c909a35e96c889818a2d91204975"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "a340706c66e4e4efd6796d1b81b506216a38bbc3afefb3842eb59e20ab774965"
        },
        "Карта реализации": {
          "Карта реализации": "527b4d52970085ddc88e9c68e90f39c589e77f053cbd4f1190d23f0bdc99b9ed"
        },
        "Критерии приёмки": {
          "AC-1": "7c1a208a2276fe375f23751575a953899eef7ff330dbe9f4ff3450eada2c7067",
          "AC-2": "acf05954f5bbacaf3e367600808a12fae86bd1cc7b248ab04f2a6fade45f0155",
          "AC-3": "b7867ea1d05c931d7061ff1b77f737b80465fba12c6dd6639b7fbfd0502e91d9",
          "AC-4": "057881d1c4f009e10bdf04d9b10aaee157f55514d95b2264cd1c0a9b502c5446",
          "AC-5": "40f6e5441e32bfbd7fbf962a8b854ab6fbc6577d9d17dfe7ab3a608922a51a84",
          "AC-6": "eb29ed46090881ecf3f33222250d3dd3aaf35626ff4577214a075315bdce9186",
          "AC-7": "d5a62948d4635cf486bf9780ff77797122082e2f08aecf0c7db39a62f4d59840",
          "AC-8": "d21d9988e8fad37e6ecf927590e3be184526b3f65bc6a818688c5f49d9c0defb",
          "AC-9": "7119e5ef722b544eb00d197fb7ef91641d3e020d8a93ddb3bd8ab64a795bec79",
          "AC-10": "99bb896ce463600951005d3ff8320bb7ed61b3a949615527cc4802a793c43628",
          "AC-11": "5f11c8cbb51ad4b0494442ab3491da29e1db7d9288875df8d7072bd240f33122"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "e8364cbf6df91d05dbc36a08f6b3fc8e7f87b84bf33c8b51eca78afacd5dce9f"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "6bbc743f0e620496ebbe88dba70313a12a538bedefcf72bf2fade9a782f0b246",
      "sections": {
        "Метаданные": "32950a5418b487d6c4a265ecf91e5cc04b0f65089a18819981dbc7ebdf32ea99",
        "Цель": "376203472158deaec080b6bc7a06cafd26a676895721aeb9d57cd860f0797f51",
        "Границы": "85ad023602caa7a0d67f2f466f18966c2b33dcf9ab59ed53e25f5bd06bb08c47",
        "Целевое поведение": "4677bd5ed07edb5fe9db1904f24e7da885cf50f95e620d05c31e1d52ca1449ce",
        "Требования": "0e2d4cd5a0b146140aace06d78c2c6515bb6344c903f7fb11142ccf5f3ea8ca9",
        "Данные, права, ошибки": "80e67b596c5953410f4e6012056504d88e48a002d769823193f2d4324e363217",
        "Карта реализации": "29acbc232bb1d2c68fe7323690e87d211bf7f827e28fa85093c2dfb8aad72723",
        "Критерии приёмки": "3e38564014c83ce2121b72ef8a3a7f14a00575126d389db3d9073334fbcc3f00",
        "Влияние на руководство": "d1c06891f123b09c9c97211206ffd2b1d3d8eeccf835effaf1217a798b3a0cc1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "53d0d9a30c6fc4b5654e372096127e194f019c103f65fbf23db7660b9ff0cb4c"
        },
        "Цель": {
          "Цель": "cb96c6cab93b94382a688fff8e2022b32889b879799465c13bb27f871e201480"
        },
        "Границы": {
          "Границы": "efdc9352927e68cef4d6d9183777abeab972f846a3e9793eb20719815e6fae88"
        },
        "Целевое поведение": {
          "Целевое поведение": "5bfc0e9277ee3bd41c3f4d4f65eabc72b8e7d1c9d26da371f11c26204fe2782c"
        },
        "Требования": {
          "FR-1": "e7996446789c3640de696542a3af8c33b4f760587eb04d849adc46ffe97a4984",
          "FR-2": "dac88565db6326cd927d281a34debb80209f7c7178a14b89f43162813e037df1",
          "FR-3": "a0041e0780b40e20186339686ac880a50410008d58cd6710ce2696b37c9cf160",
          "FR-4": "034381f5569b77a6b4e72b3e3837ef04879e1dd210c8bcf3340fd15c5eed0cbf",
          "FR-5": "9eefbc2001a874e15233bff4378698e0e433a045522e2809f37701106a31e1d3",
          "FR-6": "5f15cc83ab33c572805eef30a462264cc76b5d3fa9e1ad32b78134ab97827def",
          "FR-7": "0c5356be14ed7130c9f80ab5b6bdaf62a37c4fa876d68941b7cab88d45f41f7e",
          "FR-8": "1661d907f685f689a2d342c0953d452bfc60470717dd701d0427d2e6158ff3ed",
          "FR-9": "696d5cc06cad309d8fc1d9d533a984ed6c2412d02e4e6a14237c6f3709178551",
          "FR-10": "05b245098fa111c5d372a97986b6d601f814f0cf7f2404bf2c1609b85c89a209",
          "FR-11": "5a4118a236f4dc5206c3c7714e9058a63115c909a35e96c889818a2d91204975"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "a340706c66e4e4efd6796d1b81b506216a38bbc3afefb3842eb59e20ab774965"
        },
        "Карта реализации": {
          "Карта реализации": "527b4d52970085ddc88e9c68e90f39c589e77f053cbd4f1190d23f0bdc99b9ed"
        },
        "Критерии приёмки": {
          "AC-1": "7c1a208a2276fe375f23751575a953899eef7ff330dbe9f4ff3450eada2c7067",
          "AC-2": "acf05954f5bbacaf3e367600808a12fae86bd1cc7b248ab04f2a6fade45f0155",
          "AC-3": "b7867ea1d05c931d7061ff1b77f737b80465fba12c6dd6639b7fbfd0502e91d9",
          "AC-4": "057881d1c4f009e10bdf04d9b10aaee157f55514d95b2264cd1c0a9b502c5446",
          "AC-5": "40f6e5441e32bfbd7fbf962a8b854ab6fbc6577d9d17dfe7ab3a608922a51a84",
          "AC-6": "eb29ed46090881ecf3f33222250d3dd3aaf35626ff4577214a075315bdce9186",
          "AC-7": "d5a62948d4635cf486bf9780ff77797122082e2f08aecf0c7db39a62f4d59840",
          "AC-8": "d21d9988e8fad37e6ecf927590e3be184526b3f65bc6a818688c5f49d9c0defb",
          "AC-9": "7119e5ef722b544eb00d197fb7ef91641d3e020d8a93ddb3bd8ab64a795bec79",
          "AC-10": "99bb896ce463600951005d3ff8320bb7ed61b3a949615527cc4802a793c43628",
          "AC-11": "5f11c8cbb51ad4b0494442ab3491da29e1db7d9288875df8d7072bd240f33122"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "e8364cbf6df91d05dbc36a08f6b3fc8e7f87b84bf33c8b51eca78afacd5dce9f"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "6bbc743f0e620496ebbe88dba70313a12a538bedefcf72bf2fade9a782f0b246",
    "backend_tests": "6bbc743f0e620496ebbe88dba70313a12a538bedefcf72bf2fade9a782f0b246"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства нет; API описывают doc-комментарии (документация крейта — #46)."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "cb9b72759d6a1fa0dedb14bc911c21e67b933e8416989e8da8de512144245437",
      "files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-core/assets/syntaxes.packdump",
        "crates/nimmd-core/assets/syntaxes/LICENSE",
        "crates/nimmd-core/assets/syntaxes/TOML/TOML.sublime-syntax",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/examples/build_syntaxes.rs",
        "crates/nimmd-core/src/csv.rs",
        "crates/nimmd-core/src/data.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/highlight.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "deny.toml",
        "justfile",
        "packages/theme/data.css",
        "crates/nimmd-core/tests/data.rs",
        "crates/nimmd-core/tests/csv.rs",
        "crates/nimmd-core/tests/highlight.rs",
        "crates/nimmd-core/tests/text.rs",
        "crates/nimmd-core/tests/fixtures/data/semicolon.csv",
        "crates/nimmd-core/tests/fixtures/data/quoted.csv",
        "crates/nimmd-core/tests/fixtures/data/tabs.csv",
        "crates/nimmd-core/tests/fixtures/data/ragged.csv"
      ],
      "production_hash": "244fd53626773f7c126737c10c0187df9785cf1571ad41f34fd2ee357cae2046",
      "production_files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-core/assets/syntaxes.packdump",
        "crates/nimmd-core/assets/syntaxes/LICENSE",
        "crates/nimmd-core/assets/syntaxes/TOML/TOML.sublime-syntax",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/examples/build_syntaxes.rs",
        "crates/nimmd-core/src/csv.rs",
        "crates/nimmd-core/src/data.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/highlight.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "deny.toml",
        "justfile",
        "packages/theme/data.css"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "cb9b72759d6a1fa0dedb14bc911c21e67b933e8416989e8da8de512144245437",
      "files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-core/assets/syntaxes.packdump",
        "crates/nimmd-core/assets/syntaxes/LICENSE",
        "crates/nimmd-core/assets/syntaxes/TOML/TOML.sublime-syntax",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/examples/build_syntaxes.rs",
        "crates/nimmd-core/src/csv.rs",
        "crates/nimmd-core/src/data.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/highlight.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "deny.toml",
        "justfile",
        "packages/theme/data.css",
        "crates/nimmd-core/tests/data.rs",
        "crates/nimmd-core/tests/csv.rs",
        "crates/nimmd-core/tests/highlight.rs",
        "crates/nimmd-core/tests/text.rs",
        "crates/nimmd-core/tests/fixtures/data/semicolon.csv",
        "crates/nimmd-core/tests/fixtures/data/quoted.csv",
        "crates/nimmd-core/tests/fixtures/data/tabs.csv",
        "crates/nimmd-core/tests/fixtures/data/ragged.csv"
      ],
      "production_hash": "244fd53626773f7c126737c10c0187df9785cf1571ad41f34fd2ee357cae2046",
      "production_files": [
        "Cargo.lock",
        "Cargo.toml",
        "crates/nimmd-core/assets/syntaxes.packdump",
        "crates/nimmd-core/assets/syntaxes/LICENSE",
        "crates/nimmd-core/assets/syntaxes/TOML/TOML.sublime-syntax",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/examples/build_syntaxes.rs",
        "crates/nimmd-core/src/csv.rs",
        "crates/nimmd-core/src/data.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/highlight.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "deny.toml",
        "justfile",
        "packages/theme/data.css"
      ]
    }
  },
  "accepted_code_hash": {
    "backend-tests": "410424715ab98ec252cf63ab0f0e3282a38164d3d101dea4d6883509cb5634e5",
    "testing": "410424715ab98ec252cf63ab0f0e3282a38164d3d101dea4d6883509cb5634e5"
  }
}
```
<!-- afd-state-snapshot:end -->
