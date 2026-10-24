# Состояние разработки: Реализовать типы профиля и предобработку текста

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #39 (syper-freedm/NimMD)
- Размер: M; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-24T21:38:45+00:00

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

- 2026-10-24T21:32:23+00:00 `routed` — {"post_checks_parallel": true, "files_count": 5, "has_migrations": false, "touches_isolation": false, "skip": ["documentation"]}
- 2026-10-24T21:32:29+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-24T21:32:29+00:00 `note` — {"text": "диспатч post-checks: code-review, security-code-review, backend-tests, testing (documentation в skip)"}
- 2026-10-24T21:38:13+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": ["ARCH-003"], "notes": ["cargo fmt --check и cargo clippy --workspace --all-targets -D warnings зелёные; .gitignore изменён вне changed-files.txt (NOTICE ARCH-003)"]}
- 2026-10-24T21:38:13+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": [], "notes": ["cargo deny check и semgrep запущены с обходом песочницы/переопределением путей логов: песочница запрещала запись в ~/.cargo/advisory-dbs и ~/.semgrep"]}
- 2026-10-24T21:38:13+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Coverage: SC-001 default_profile_is_commonmark_without_features; SC-002 dialect_json_names_round_trip, profile_json_round_trip; SC-003 dialect_from_str_is_ascii_case_insensitive, dialect_from_str_rejects_unknown, unknown_dialect_is_an_error; SC-004 strip_bom_removes_one_leading_bom; SC-005 detect_line_ending_picks_majority; SC-006 normalize_newlines_converts_cr_and_crlf; SC-007 split_frontmatter_cases, split_frontmatter_concatenates_back, split_frontmatter_stops_at_first_closing_line; SC-008 frontmatter_dialect_cases, frontmatter_dialect_edge_cases; SC-009 truncate_at_char_boundary_cases; SC-010 functions_are_total_and_match_reference (fixed strings + 2000 xorshift strings, seed 0x9E3779B97F4A7C15, differential vs naive references for detect_line_ending/normalize_newlines/truncate); SEC-002 profile_rejects_unknown_fields; SC-011 static: lib.rs grep = 3 matches, just lint green", "Red runs (feature, not bug: stash would only break compilation, so mutation testing instead; src backed up to tmp_dir, restored, shasum identical): tie->CrLf fails detect_line_ending_picks_majority (left: CrLf right: Lf); closing '--- ' accepted fails split_frontmatter_cases; indented markdown: key fails frontmatter_dialect_cases (left: Some(Gfm) right: None); truncate (0..max) fails functions_are_total_and_match_reference; lone CR kept fails normalize_newlines_converts_cr_and_crlf (left: \"a\\nb\\rc\"); no rename_all fails dialect_json_names_round_trip (left: \"\\\"CommonMark\\\"\"); no deny_unknown_fields fails profile_rejects_unknown_fields; trim in from_str fails dialect_from_str_rejects_unknown", "Checks: cargo test --workspace --all-targets: 17 passed, 0 failed, 0 ignored; cargo fmt --check, cargo clippy --all-targets -D warnings, cargo build: clean; diff-lint.sh backend: clippy-changed clean, cargo-machete clean, semgrep clean; just lint recipes all green", "Env: semgrep needs SEMGREP_LOG_FILE in a writable dir under the sandbox (~/.semgrep is read-only); cargo deny check needs to run outside the sandbox (advisory-db lock in ~/.cargo) - run there: advisories/bans/licenses/sources ok"]}
- 2026-10-24T21:38:13+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный cargo test --workspace --all-targets не повторялся: control/suite-run.backend-tests.json green, tree e416b03c438fdc06ce6f0ed3fc8d9d18e7ac4894 равен текущему tree-hash.", "cargo test -p nimmd-core --test profile --test text: 7 + 10 passed, 0 ignored; SC-001…SC-010 pass.", "SC-011: git grep — 3 совпадения в crates/nimmd-core/src/lib.rs; just lint зелёный по всем рецептам.", "Песочница: semgrep не мог писать ~/.semgrep/semgrep.log — прогнан с SEMGREP_LOG_FILE в tmp_dir (0 findings); cargo deny check упал на блокировке ~/.cargo/advisory-dbs (read-only) — повторён вне песочницы: advisories ok, bans ok, licenses ok, sources ok."]}
- 2026-10-24T21:38:13+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-24T21:38:13+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-24T21:38:13+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-24T21:38:14+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-24T21:38:14+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-24T21:38:14+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-24T21:38:14+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-24T21:38:45+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 39,
  "title": "Реализовать типы профиля и предобработку текста",
  "slug": "realizovat-tipy-profilya-i-predobrabotku-teksta",
  "branch": "feat/39-realizovat-tipy-profilya-i-predobrabotku",
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
      "at": "2026-10-24T21:24:51+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-24T21:25:54+00:00",
      "event": "classified",
      "size": "M",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-24T21:25:54+00:00",
      "event": "note",
      "text": "init: ветка claude/project-thread-3a2x75 от claude/project-thread-69hel4 (#36 не влита в main); метка core → область backend; вилок продукта на шаге 1 нет — всё отвечено спекой 5.1/6 и планом Task 1"
    },
    {
      "at": "2026-10-24T21:26:46+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-24T21:26:46+00:00",
      "event": "note",
      "text": "диспатч FD-ревью: architecture, security, qa"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002"
      ],
      "notes": [
        "Карта ложится на пустой крейт nimmd-core (lib.rs только с forbid(unsafe_code)); дублей strip_bom/split_frontmatter/Dialect в отслеживаемом коде нет (git grep).",
        "serde с derive уже в [workspace.dependencies]; инвариант semgrep nimmd-core-no-io не затрагивается.",
        "Спека и план в docs/ gitignored — не открывались; AC-2..AC-6 сверены вручную с FR-3..FR-8, расхождений нет."
      ]
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001",
        "SEC-002"
      ],
      "notes": [
        "Чистый библиотечный API без ввода-вывода, прав и чувствительных данных: security.md не нужен.",
        "Тотальность функций и линейная сложность заявлены в ФД; MAX_BYTES ограничивает вход от недоверенного файла Quick Look.",
        "SEC-001, SEC-002 — NOTICE для реализатора, ФД не править."
      ]
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "QA-001, QA-002 — NOTICE, правка ФД не нужна: FR-9, Display/Error у UnknownDialect и тотальность без AC закрыты сценариями SC-003, SC-010, SC-011; составные AC-1/AC-2 разнесены по сценариям.",
        "Ожидания AC-2…AC-6 пересчитаны по FR-4…FR-8 вручную — расхождений нет.",
        "SC-002 требует serde_json в [dev-dependencies] nimmd-core; в [workspace.dependencies] он уже есть."
      ]
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "note",
      "text": "FD-ревью: три APPROVED, NOTICE ARCH-001/002, SEC-001/002, QA-001/002 — разработчику, ФД не правится; агенты afd-* поставлены посреди сессии и не видны — workflow-копии диспатчат general-purpose с файлом роли"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:29:32+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-24T21:29:33+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-24T21:29:33+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-24T21:29:33+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-24T21:29:33+00:00",
      "event": "note",
      "text": "диспатч implementation: backend"
    },
    {
      "at": "2026-10-24T21:32:23+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: profile.rs, text.rs, lib.rs, nimmd-core/Cargo.toml, Cargo.lock",
        "self-checks green: cargo fmt --check, cargo build --workspace --all-targets, cargo clippy -D warnings, cargo test --workspace --all-targets (0 tests in crate yet), cargo deny check (advisories/bans/licenses/sources ok; pre-existing warning: unmatched license exceptions option-ext/cbindgen in deny.toml), just lint green, diff-lint: clippy-changed clean, machete clean, semgrep clean",
        "AC-1..AC-6 and SC-010 totality verified by a throwaway integration test (removed, not committed); new tests remain for afd-test-developer (crates/nimmd-core/tests/text.rs)",
        "ARCH-001: serde_json = { workspace = true } added to [dev-dependencies] of nimmd-core",
        "ARCH-002: #[derive(Default)] + #[default] CommonMark on Dialect",
        "SEC-001: text.rs slices via strip_prefix/split_at_checked/get only; module-level #![deny(clippy::indexing_slicing, clippy::string_slice)]",
        "SEC-002: #[serde(deny_unknown_fields)] on Features and Profile",
        "Features carries #[allow(clippy::struct_excessive_bools)]: nine bool fields are fixed by FR-1",
        "cargo deny / just lint ran outside sandbox (advisory DB lock under ~/.cargo is read-only in sandbox)"
      ]
    },
    {
      "at": "2026-10-24T21:32:23+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-24T21:32:23+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-24T21:32:23+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 5,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ]
    },
    {
      "at": "2026-10-24T21:32:29+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-24T21:32:29+00:00",
      "event": "note",
      "text": "диспатч post-checks: code-review, security-code-review, backend-tests, testing (documentation в skip)"
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-003"
      ],
      "notes": [
        "cargo fmt --check и cargo clippy --workspace --all-targets -D warnings зелёные; .gitignore изменён вне changed-files.txt (NOTICE ARCH-003)"
      ]
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "cargo deny check и semgrep запущены с обходом песочницы/переопределением путей логов: песочница запрещала запись в ~/.cargo/advisory-dbs и ~/.semgrep"
      ]
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Coverage: SC-001 default_profile_is_commonmark_without_features; SC-002 dialect_json_names_round_trip, profile_json_round_trip; SC-003 dialect_from_str_is_ascii_case_insensitive, dialect_from_str_rejects_unknown, unknown_dialect_is_an_error; SC-004 strip_bom_removes_one_leading_bom; SC-005 detect_line_ending_picks_majority; SC-006 normalize_newlines_converts_cr_and_crlf; SC-007 split_frontmatter_cases, split_frontmatter_concatenates_back, split_frontmatter_stops_at_first_closing_line; SC-008 frontmatter_dialect_cases, frontmatter_dialect_edge_cases; SC-009 truncate_at_char_boundary_cases; SC-010 functions_are_total_and_match_reference (fixed strings + 2000 xorshift strings, seed 0x9E3779B97F4A7C15, differential vs naive references for detect_line_ending/normalize_newlines/truncate); SEC-002 profile_rejects_unknown_fields; SC-011 static: lib.rs grep = 3 matches, just lint green",
        "Red runs (feature, not bug: stash would only break compilation, so mutation testing instead; src backed up to tmp_dir, restored, shasum identical): tie->CrLf fails detect_line_ending_picks_majority (left: CrLf right: Lf); closing '--- ' accepted fails split_frontmatter_cases; indented markdown: key fails frontmatter_dialect_cases (left: Some(Gfm) right: None); truncate (0..max) fails functions_are_total_and_match_reference; lone CR kept fails normalize_newlines_converts_cr_and_crlf (left: \"a\\nb\\rc\"); no rename_all fails dialect_json_names_round_trip (left: \"\\\"CommonMark\\\"\"); no deny_unknown_fields fails profile_rejects_unknown_fields; trim in from_str fails dialect_from_str_rejects_unknown",
        "Checks: cargo test --workspace --all-targets: 17 passed, 0 failed, 0 ignored; cargo fmt --check, cargo clippy --all-targets -D warnings, cargo build: clean; diff-lint.sh backend: clippy-changed clean, cargo-machete clean, semgrep clean; just lint recipes all green",
        "Env: semgrep needs SEMGREP_LOG_FILE in a writable dir under the sandbox (~/.semgrep is read-only); cargo deny check needs to run outside the sandbox (advisory-db lock in ~/.cargo) - run there: advisories/bans/licenses/sources ok"
      ]
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный cargo test --workspace --all-targets не повторялся: control/suite-run.backend-tests.json green, tree e416b03c438fdc06ce6f0ed3fc8d9d18e7ac4894 равен текущему tree-hash.",
        "cargo test -p nimmd-core --test profile --test text: 7 + 10 passed, 0 ignored; SC-001…SC-010 pass.",
        "SC-011: git grep — 3 совпадения в crates/nimmd-core/src/lib.rs; just lint зелёный по всем рецептам.",
        "Песочница: semgrep не мог писать ~/.semgrep/semgrep.log — прогнан с SEMGREP_LOG_FILE в tmp_dir (0 findings); cargo deny check упал на блокировке ~/.cargo/advisory-dbs (read-only) — повторён вне песочницы: advisories ok, bans ok, licenses ok, sources ok."
      ]
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:38:13+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:38:14+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-24T21:38:14+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-24T21:38:14+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-24T21:38:14+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-24T21:38:45+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "5b93bfa2bbef4ab9db27a38b4ed4abe5a5fee901342314c47f485944fd11ad97",
      "sections": {
        "Метаданные": "36e4852bc6cb48ef83f9c1292d405aa4da68b62fafbe6c2b73ca62afc203623d",
        "Цель": "8b2af5da268084960058dbf4a7986898f5b76c0bcf203856ff88505c0d4db29a",
        "Границы": "3cdfc62a813e2f474bd4aa952979fe807c6662d34f288d8d085404f4b44f1865",
        "Целевое поведение": "e9457f3ce08e9568557873568b0f376254a9c5b5c338fda71d50c151e672856c",
        "Требования": "9fbaf5a352b5a0101dc008103821046f7392b698f2611b2f804b66629ecb0fa4",
        "Данные, права, ошибки": "3889567237b7e634d5371fa76acd162dafe62f704d7d36cc66beb5406d720ed6",
        "Карта реализации": "b137be4621638ef492429cdef1f00e1ae19759d2d64aa434e83f7ef7f99a8fc3",
        "Критерии приёмки": "f5b57c7a0adce682265a0107b7d33ef544474ed404c531c596ba4af3615f08b3",
        "Влияние на руководство": "4d8ef4179ab881d9f9ea12961512d0752400ad29d5a3bb7d738db57f20dc2e47"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "c2f3f24538ba518506a14ecb5345b8634fbbf86af24586766aba54cbf1a7fa15"
        },
        "Цель": {
          "Цель": "95de8944149c0bd52867989e13bbb16aee87bf0a2351493e6e3bbf29be82e03a"
        },
        "Границы": {
          "Границы": "8e63a1cbbdd74f58ccf04d47b3cab5f8da9a5add3ecb6892db98934431b0d6e6"
        },
        "Целевое поведение": {
          "Целевое поведение": "730c50e0ec4e0d052d2beda29559d1f0b3bd6a4609308a7d671f66c1190d4f2e"
        },
        "Требования": {
          "FR-1": "65e911a9c5fe8ba36d86025e0482e6375de64aad884f42fcb856074f43b501f2",
          "FR-2": "2badc85d3154d81ed0540ea786b7b68e20c2fc4e30cbbfdbf7bd82b469f965b0",
          "FR-3": "8f3451a8d91998df22bae02bbadcb7e55601169e3888a681bc67153c4f14dac8",
          "FR-4": "be5c82c9e3e4c2826c12132b05cacbd97b37b6a01ad29fd474d14360675df739",
          "FR-5": "05b38af4f211d58aee39f7875ef7832f9695565399324e0783c4ecd48406b452",
          "FR-6": "d80fe3608ef53ff039b471cfdbfd26250d753d3aa5ee9cb0e2a85d1788d67a20",
          "FR-7": "35fe405957b41e9f6b3f9ca6cc83ccf55530add4ef75e0538fa070e0a1c6e02e",
          "FR-8": "b7171b0c7c0c9e66211e5ddc7118302bc30f865c7fc7153f1a305c486ef12cbd",
          "FR-9": "3263fff321b1bf32c81f5b6d83bae5f3da79931ea70bdd8601dd21efd3f93341"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "0cf5c064d779683d62efe18936828863ae5181e8e6b4b05999bbc492270c1bd1"
        },
        "Карта реализации": {
          "Карта реализации": "b4dfe43495428599edd2cd84003a780bbea7940702dff27d69ea3ef2043ff394"
        },
        "Критерии приёмки": {
          "AC-1": "3e963be5df4564c712ab079f1af45062f1988086e0e8332b7d99e0a43dd09e15",
          "AC-2": "335b810332870f37b535c77cc835d5ce2cc1efc1deed49f359b56a5a3110ecfe",
          "AC-3": "2bfee9527047d11ccbced3d3bc1315fe1311b4ca7c06117f8238618c67858981",
          "AC-4": "0541add3d72815125ce0873df6efc32ac608a0a2e05b3ab6d0a26293186cb1d0",
          "AC-5": "413488da2db8404278a0846b369b95eee0eb38965e3ac31b9576c988e599643b",
          "AC-6": "1a883011333b13b46abe703dd063af1b9104e402158b6d9ad968be58a2c4fd48"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "2330a8f636c73ca29e7fa647fe79cad8eb002b49e4b6eb622f426e8f43c24447"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "5b93bfa2bbef4ab9db27a38b4ed4abe5a5fee901342314c47f485944fd11ad97",
      "sections": {
        "Метаданные": "36e4852bc6cb48ef83f9c1292d405aa4da68b62fafbe6c2b73ca62afc203623d",
        "Цель": "8b2af5da268084960058dbf4a7986898f5b76c0bcf203856ff88505c0d4db29a",
        "Границы": "3cdfc62a813e2f474bd4aa952979fe807c6662d34f288d8d085404f4b44f1865",
        "Целевое поведение": "e9457f3ce08e9568557873568b0f376254a9c5b5c338fda71d50c151e672856c",
        "Требования": "9fbaf5a352b5a0101dc008103821046f7392b698f2611b2f804b66629ecb0fa4",
        "Данные, права, ошибки": "3889567237b7e634d5371fa76acd162dafe62f704d7d36cc66beb5406d720ed6",
        "Карта реализации": "b137be4621638ef492429cdef1f00e1ae19759d2d64aa434e83f7ef7f99a8fc3",
        "Критерии приёмки": "f5b57c7a0adce682265a0107b7d33ef544474ed404c531c596ba4af3615f08b3",
        "Влияние на руководство": "4d8ef4179ab881d9f9ea12961512d0752400ad29d5a3bb7d738db57f20dc2e47"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "c2f3f24538ba518506a14ecb5345b8634fbbf86af24586766aba54cbf1a7fa15"
        },
        "Цель": {
          "Цель": "95de8944149c0bd52867989e13bbb16aee87bf0a2351493e6e3bbf29be82e03a"
        },
        "Границы": {
          "Границы": "8e63a1cbbdd74f58ccf04d47b3cab5f8da9a5add3ecb6892db98934431b0d6e6"
        },
        "Целевое поведение": {
          "Целевое поведение": "730c50e0ec4e0d052d2beda29559d1f0b3bd6a4609308a7d671f66c1190d4f2e"
        },
        "Требования": {
          "FR-1": "65e911a9c5fe8ba36d86025e0482e6375de64aad884f42fcb856074f43b501f2",
          "FR-2": "2badc85d3154d81ed0540ea786b7b68e20c2fc4e30cbbfdbf7bd82b469f965b0",
          "FR-3": "8f3451a8d91998df22bae02bbadcb7e55601169e3888a681bc67153c4f14dac8",
          "FR-4": "be5c82c9e3e4c2826c12132b05cacbd97b37b6a01ad29fd474d14360675df739",
          "FR-5": "05b38af4f211d58aee39f7875ef7832f9695565399324e0783c4ecd48406b452",
          "FR-6": "d80fe3608ef53ff039b471cfdbfd26250d753d3aa5ee9cb0e2a85d1788d67a20",
          "FR-7": "35fe405957b41e9f6b3f9ca6cc83ccf55530add4ef75e0538fa070e0a1c6e02e",
          "FR-8": "b7171b0c7c0c9e66211e5ddc7118302bc30f865c7fc7153f1a305c486ef12cbd",
          "FR-9": "3263fff321b1bf32c81f5b6d83bae5f3da79931ea70bdd8601dd21efd3f93341"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "0cf5c064d779683d62efe18936828863ae5181e8e6b4b05999bbc492270c1bd1"
        },
        "Карта реализации": {
          "Карта реализации": "b4dfe43495428599edd2cd84003a780bbea7940702dff27d69ea3ef2043ff394"
        },
        "Критерии приёмки": {
          "AC-1": "3e963be5df4564c712ab079f1af45062f1988086e0e8332b7d99e0a43dd09e15",
          "AC-2": "335b810332870f37b535c77cc835d5ce2cc1efc1deed49f359b56a5a3110ecfe",
          "AC-3": "2bfee9527047d11ccbced3d3bc1315fe1311b4ca7c06117f8238618c67858981",
          "AC-4": "0541add3d72815125ce0873df6efc32ac608a0a2e05b3ab6d0a26293186cb1d0",
          "AC-5": "413488da2db8404278a0846b369b95eee0eb38965e3ac31b9576c988e599643b",
          "AC-6": "1a883011333b13b46abe703dd063af1b9104e402158b6d9ad968be58a2c4fd48"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "2330a8f636c73ca29e7fa647fe79cad8eb002b49e4b6eb622f426e8f43c24447"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "5b93bfa2bbef4ab9db27a38b4ed4abe5a5fee901342314c47f485944fd11ad97",
      "sections": {
        "Метаданные": "36e4852bc6cb48ef83f9c1292d405aa4da68b62fafbe6c2b73ca62afc203623d",
        "Цель": "8b2af5da268084960058dbf4a7986898f5b76c0bcf203856ff88505c0d4db29a",
        "Границы": "3cdfc62a813e2f474bd4aa952979fe807c6662d34f288d8d085404f4b44f1865",
        "Целевое поведение": "e9457f3ce08e9568557873568b0f376254a9c5b5c338fda71d50c151e672856c",
        "Требования": "9fbaf5a352b5a0101dc008103821046f7392b698f2611b2f804b66629ecb0fa4",
        "Данные, права, ошибки": "3889567237b7e634d5371fa76acd162dafe62f704d7d36cc66beb5406d720ed6",
        "Карта реализации": "b137be4621638ef492429cdef1f00e1ae19759d2d64aa434e83f7ef7f99a8fc3",
        "Критерии приёмки": "f5b57c7a0adce682265a0107b7d33ef544474ed404c531c596ba4af3615f08b3",
        "Влияние на руководство": "4d8ef4179ab881d9f9ea12961512d0752400ad29d5a3bb7d738db57f20dc2e47"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "c2f3f24538ba518506a14ecb5345b8634fbbf86af24586766aba54cbf1a7fa15"
        },
        "Цель": {
          "Цель": "95de8944149c0bd52867989e13bbb16aee87bf0a2351493e6e3bbf29be82e03a"
        },
        "Границы": {
          "Границы": "8e63a1cbbdd74f58ccf04d47b3cab5f8da9a5add3ecb6892db98934431b0d6e6"
        },
        "Целевое поведение": {
          "Целевое поведение": "730c50e0ec4e0d052d2beda29559d1f0b3bd6a4609308a7d671f66c1190d4f2e"
        },
        "Требования": {
          "FR-1": "65e911a9c5fe8ba36d86025e0482e6375de64aad884f42fcb856074f43b501f2",
          "FR-2": "2badc85d3154d81ed0540ea786b7b68e20c2fc4e30cbbfdbf7bd82b469f965b0",
          "FR-3": "8f3451a8d91998df22bae02bbadcb7e55601169e3888a681bc67153c4f14dac8",
          "FR-4": "be5c82c9e3e4c2826c12132b05cacbd97b37b6a01ad29fd474d14360675df739",
          "FR-5": "05b38af4f211d58aee39f7875ef7832f9695565399324e0783c4ecd48406b452",
          "FR-6": "d80fe3608ef53ff039b471cfdbfd26250d753d3aa5ee9cb0e2a85d1788d67a20",
          "FR-7": "35fe405957b41e9f6b3f9ca6cc83ccf55530add4ef75e0538fa070e0a1c6e02e",
          "FR-8": "b7171b0c7c0c9e66211e5ddc7118302bc30f865c7fc7153f1a305c486ef12cbd",
          "FR-9": "3263fff321b1bf32c81f5b6d83bae5f3da79931ea70bdd8601dd21efd3f93341"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "0cf5c064d779683d62efe18936828863ae5181e8e6b4b05999bbc492270c1bd1"
        },
        "Карта реализации": {
          "Карта реализации": "b4dfe43495428599edd2cd84003a780bbea7940702dff27d69ea3ef2043ff394"
        },
        "Критерии приёмки": {
          "AC-1": "3e963be5df4564c712ab079f1af45062f1988086e0e8332b7d99e0a43dd09e15",
          "AC-2": "335b810332870f37b535c77cc835d5ce2cc1efc1deed49f359b56a5a3110ecfe",
          "AC-3": "2bfee9527047d11ccbced3d3bc1315fe1311b4ca7c06117f8238618c67858981",
          "AC-4": "0541add3d72815125ce0873df6efc32ac608a0a2e05b3ab6d0a26293186cb1d0",
          "AC-5": "413488da2db8404278a0846b369b95eee0eb38965e3ac31b9576c988e599643b",
          "AC-6": "1a883011333b13b46abe703dd063af1b9104e402158b6d9ad968be58a2c4fd48"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "2330a8f636c73ca29e7fa647fe79cad8eb002b49e4b6eb622f426e8f43c24447"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "5b93bfa2bbef4ab9db27a38b4ed4abe5a5fee901342314c47f485944fd11ad97",
    "backend_tests": "5b93bfa2bbef4ab9db27a38b4ed4abe5a5fee901342314c47f485944fd11ad97"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: внутренний API ядра без пользовательского интерфейса; документация крейта — #46."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "2bf3f193b1a7702ac110b371c8c9560871be051a697367084a289ff67d68afed",
      "files": [
        "crates/nimmd-core/src/profile.rs",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock",
        "crates/nimmd-core/tests/profile.rs",
        "crates/nimmd-core/tests/text.rs"
      ],
      "production_hash": "da2660e2c0494b1462e01de1500eb087bb5627128e75f1ef5056dec7b3ee5b5a",
      "production_files": [
        "crates/nimmd-core/src/profile.rs",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "2bf3f193b1a7702ac110b371c8c9560871be051a697367084a289ff67d68afed",
      "files": [
        "crates/nimmd-core/src/profile.rs",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock",
        "crates/nimmd-core/tests/profile.rs",
        "crates/nimmd-core/tests/text.rs"
      ],
      "production_hash": "da2660e2c0494b1462e01de1500eb087bb5627128e75f1ef5056dec7b3ee5b5a",
      "production_files": [
        "crates/nimmd-core/src/profile.rs",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock"
      ]
    }
  },
  "accepted_code_hash": {
    "backend-tests": "b615055d5f15a1c2ada59a34a6e58cf6c659210bcda5c8d5227ff87694f833b6",
    "testing": "b615055d5f15a1c2ada59a34a6e58cf6c659210bcda5c8d5227ff87694f833b6"
  }
}
```
<!-- afd-state-snapshot:end -->
