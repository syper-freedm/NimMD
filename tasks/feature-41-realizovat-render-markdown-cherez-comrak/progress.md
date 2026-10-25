# Состояние разработки: Реализовать рендер Markdown через comrak

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #41 (syper-freedm/NimMD)
- Размер: M; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T12:30:51+00:00

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

- 2026-10-25T12:14:47+00:00 `routed` — {"post_checks_parallel": true, "files_count": 8, "has_migrations": false, "touches_isolation": false, "skip": ["documentation"]}
- 2026-10-25T12:14:54+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-25T12:14:54+00:00 `note` — {"text": "диспатч post-checks; в «Новые тесты» — render_fragment.rs + insta-снапшоты"}
- 2026-10-25T12:30:07+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": ["ARCH-002"], "notes": ["clippy -D warnings и cargo test -p nimmd-core зелёные; смоук AC-1..AC-11 во временном крейте подтвердил поведение", "tests/render_fragment.rs и insta-снапшоты из карты реализации в правке пока отсутствуют — их пишет тестовая фаза"]}
- 2026-10-25T12:30:08+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": ["SEC-003", "SEC-004"], "notes": ["cargo deny check: advisories ok, bans ok, licenses ok, sources ok (run outside sandbox: ~/.cargo/advisory-dbs lock is read-only in sandbox)", "gitleaks dir over changed-files.txt: no leaks found", "semgrep .semgrep.yml over changed src: 0 findings; registry packs p/rust p/typescript p/javascript p/secrets failed to download (semgrep.dev connection error) -> SEC-004 NOTICE", "manual probes: javascript:/vbscript:/file:/data:text/html, wikilink, autolink, entity-encoded schemes -> empty href; raw HTML escaped; mermaid/math/alert/footnote/table content escaped; info-string attribute injection not reflected", "SEC-003: syntect Err -> comrak writes code raw; 26025 fuzz runs over all default grammars: 0 leaks, 0 panics; not reachable as found", "Snyk not connected, skipped; no workflows touched; no stack.isolation"]}
- 2026-10-25T12:30:08+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Coverage: SC-001 table_alignment_and_tasks; SC-002 code_highlighted_with_classes; SC-003 code_without_or_with_unknown_language; SC-004 five_alert_types_ignore_case; SC-005 obsidian_callout_is_plain_blockquote; SC-006 math_and_prices; SC-007 mermaid_closed_unclosed_and_any_case; SC-008 raw_html_is_escaped; SC-009 dangerous_urls_dropped_safe_images_kept; SC-010 wikilink_and_footnotes; SC-011 leading_rule_block_kept; SC-012 detect_parses_body_after_frontmatter (+ cargo test -p nimmd-core --test detect green, 27 passed); SC-013 deep_nesting_is_fast; SC-014 ordered_marker_over_limit_escapes_punctuation; SC-015 leading_indent_is_capped (+ --test text green, 14 passed); SC-016 disabled_and_enabled_extensions; extra: render_is_total_on_random_input (xorshift seed 0x00C0FFEED00DF00D, 2000 inputs, no panic, no <script / <img src=x).", "Snapshots made with INSTA_UPDATE=always (cargo-insta is not installed) and checked by eye; the final run used the default INSTA_UPDATE and left no .snap.new files.", "Red run 1: git stash push -- crates/nimmd-core/src/text.rs crates/nimmd-core/src/detect.rs; cargo test -p nimmd-core --test render -> FAILED leading_indent_is_capped (left 300 spaces + '- x', right 256), ordered_marker_over_limit_escapes_punctuation (left '...1. \\\\1. ...', right '...1\\\\. ...'), deep_nesting_is_fast; stash pop, git diff --stat unchanged.", "Red run 2: only detect.rs stashed -> detect_parses_body_after_frontmatter FAILED: Profile { ... frontmatter: true, math: true ... } for '---\\ntitle: $a+b$\\n---\\nтекст'. The literal AC-9 input does not catch the regression on its own, so I added checks for math/mermaid inside the frontmatter.", "Red run 3: render.rs is untracked, so stash could not take it. I backed it up in tmp_dir, disabled the normalize_mermaid call and ran the test: mermaid_closed_unclosed_and_any_case FAILED (```Mermaid went to syntect). Restored the file and confirmed with cmp that it matches the backup.", "Red run 4, after the timing change: text.rs stashed -> deep_nesting_is_fast FAILED on the structural check (indent width 260 > 256). With the 5 s limit alone the timing check passed without the fix (ladder about 4.5 s), so the structural check is what catches the regression.", "Deviation from AC-10(в), sanctioned by NOTICE QA-004 and ARCH-002: the 800-line ladder takes 0.85-0.95 s in debug on Apple Silicon. 1 s would leave no margin (the render suite in parallel already takes 0.98 s), so its limit was raised to 5 s (5x margin). Inputs (а) and (б) keep the 1 s limit. If the AC must hold at exactly 1 s, the FD has to decide that.", "Checks: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings 0 issues; diff-lint.sh backend exit 0 (clippy-changed clean, cargo-machete clean, semgrep clean). Semgrep needed SEMGREP_LOG_FILE/SEMGREP_SETTINGS_FILE pointed at tmp_dir: the sandbox does not allow writes to ~/.semgrep. Full cargo test --workspace --all-targets green: 0+27+7+17+14 passed, 0 ignored.", "Not run: cargo deny check and just lint (network). These do not affect the test files.", "SEC-002 (hostile code block in the Task 7 benchmark) is outside the tests of this task: there are no benchmarks yet."]}
- 2026-10-25T12:30:08+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный прогон не повторялся: control/suite-run.backend-tests.json result=green, tree f8bb12e98919ec4f6bacdff847829efe18c06feb совпадает с tree-hash.sh", "Точечно: cargo test -p nimmd-core --test render --test text --test detect — render 17/17, text 14/14, detect 27/27 ok, 0 ignored", "SC-013/AC-10: замер render_fragment в debug (временный крейт в control/.tmp/testing, --offline): (а) 5–43 мс, (б) <1 мс, (в) лестница 0,885–0,898 с — меньше 1 с; тест deep_nesting_is_fast держит для (в) порог 5 с (по QA-004/ARCH-002), запас к 1 с у кода ~10%", "SC-015: 65 табуляций проверены как 64 табуляции (ширина 256 колонок) в leading_indent_is_capped; инвариант FR-7 в limit_nesting_is_lossless_and_idempotent зелёный", "control/changed-files.docs.txt пуст — сверять руководство нечего"]}
- 2026-10-25T12:30:08+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-25T12:30:08+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T12:30:08+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T12:30:08+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T12:30:08+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T12:30:08+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T12:30:08+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T12:30:51+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 41,
  "title": "Реализовать рендер Markdown через comrak",
  "slug": "realizovat-render-markdown-cherez-comrak",
  "branch": "feat/41-realizovat-render-markdown-cherez-comrak",
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
      "at": "2026-10-25T12:01:29+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T12:02:59+00:00",
      "event": "classified",
      "size": "M",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T12:03:00+00:00",
      "event": "note",
      "text": "ФД написан; продуктовых развилок нет; ARCH-005 из #40 включён как FR-7(б)"
    },
    {
      "at": "2026-10-25T12:03:00+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T12:03:00+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:03:00+00:00",
      "event": "note",
      "text": "диспатч FD-ревью"
    },
    {
      "at": "2026-10-25T12:06:13+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001"
      ],
      "notes": [
        "Сверено с comrak 0.56.0: имена опций FR-2, SyntectAdapterBuilder::css, dangerous_url (FR-3), alert_start только пяти типов (AC-4), Send+Sync у адаптеров и Options (OnceLock реализуем). Существующие тесты limit_nesting правилу FR-7(а) не противоречат."
      ]
    },
    {
      "at": "2026-10-25T12:06:13+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001",
        "SEC-002"
      ],
      "notes": [
        "comrak 0.56: raw HTML escaping requires render.escape = true, absent from FR-2",
        "security.md not created: no new permissions, sensitive data or protected flow"
      ]
    },
    {
      "at": "2026-10-25T12:06:13+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "BLOCKER",
      "findings": [
        "QA-001",
        "QA-002",
        "QA-003",
        "QA-004"
      ],
      "notes": [
        "QA-001 BLOCKER: comrak 0.56 при unsafe_=false и render.escape=false пишет <!-- raw HTML omitted --> (html.rs render_html_block/render_html_inline), а не &lt; — AC-7 (а),(б) противоречит FR-2/FR-3; исправление — render.escape = true в FR-2",
        "QA-002..QA-004 NOTICE: FR-2/FR-3 без AC (description_lists, header_ids, multiline_block_quotes, vbscript:/file:/автоссылки); FR-7(б) ломает инвариант теста limit_nesting_is_lossless_and_idempotent (tests/text.rs:323); тайминг 1 с в debug на CI",
        "test-cases.md: SC-001..SC-015, все AC-1..AC-10 покрыты, afd.py validate-tests — OK; объектный доступ неприменим (библиотека без контроля доступа)"
      ]
    },
    {
      "at": "2026-10-25T12:06:29+00:00",
      "event": "note",
      "text": "QA-001: принято автором ФД без скептика — архитектор/ИБ/QA независимо сверили html.rs comrak 0.56 (escape)"
    },
    {
      "at": "2026-10-25T12:06:29+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "requirements_rework",
      "reason": "QA-001"
    },
    {
      "at": "2026-10-25T12:06:29+00:00",
      "event": "transition",
      "from": "requirements_rework",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T12:06:37+00:00",
      "event": "note",
      "text": "confirm-круг FD после QA-001"
    },
    {
      "at": "2026-10-25T12:09:07+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001"
      ],
      "notes": [
        "Режим дельты: FR-2, FR-3, FR-7, AC-7, AC-11 — правки закрывают QA-001/QA-002/QA-003/SEC-001, новых архитектурных дефектов не вносят; render.escape есть в comrak 0.56 (options.rs:1145). Секция reviews.md оставлена без изменений."
      ]
    },
    {
      "at": "2026-10-25T12:09:07+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001",
        "SEC-002"
      ],
      "notes": [
        "Дельта FR-2/FR-3/AC-7/AC-11/FR-7 проверена: render.escape = true (есть в comrak 0.56, html.rs:684/733) закрывает SEC-001, AC-7 расширен на vbscript:/file:/автоссылки, AC-11 и инвариант FR-7 новых дыр не открывают. reviews.md не менялся."
      ]
    },
    {
      "at": "2026-10-25T12:09:07+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-004",
        "QA-005"
      ],
      "notes": [
        "Дельта FR-2/FR-3/FR-7/AC-7/AC-11 закрывает QA-001 (render.escape = true, поле есть в comrak 0.56 options.rs:1145), QA-002 (AC-7 (в) и AC-11), QA-003 (новый инвариант limit_nesting)",
        "dangerous_url comrak 0.56 (scanners.re:372-373, re2c '...' — без учёта регистра) подтверждает FR-3",
        "QA-005 NOTICE: AC-11 (б) не ловит header_id_prefix (id=\"h-id\"); SC-016 проверяет отсутствие id= вообще",
        "test-cases.md: добавлен SC-016 (AC-11), SC-008/SC-009/SC-015 обновлены, таблица покрытия убрана ради лимита (2698/2700); validate-tests и validate-reviews --stage document — OK"
      ]
    },
    {
      "at": "2026-10-25T12:09:07+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:09:07+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:09:08+00:00",
      "event": "note",
      "text": "ФД одобрен; NOTICE ARCH-001 (регистр Mermaid), SEC-002, QA-004, QA-005 — разработчику; диспатч backend"
    },
    {
      "at": "2026-10-25T12:14:46+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: Cargo.lock, crates/nimmd-core/{Cargo.toml, src/detect.rs, src/lib.rs, src/parse.rs, src/render.rs, src/text.rs, tests/text.rs}",
        "checks: cargo fmt --check ok; cargo build ok; cargo clippy -D warnings ok; cargo test --workspace --all-targets: detect 27/27, profile 7/7, text 14/14 passed; cargo deny check: advisories/bans/licenses/sources ok (pre-existing warning: unmatched license exception cbindgen in deny.toml); cargo machete clean; semgrep .semgrep.yml clean; diff-lint backend exit 0",
        "the existing test limit_nesting_is_lossless_and_idempotent was adapted to the new FR-7 invariant (only leading indentation may be shortened); new tests were not written (they belong to the backend-tests phase)",
        "manual one-off run of the AC inputs (file deleted): AC-1..AC-11 match; AC-10 in debug: >x60000 5 ms, 1.x100 0.4 ms, 800-line ladder 32 ms; 300 spaces -> 256",
        "ARCH-001 is handled by normalizing the info string to lowercase 'mermaid' in the AST before rendering (comrak looks up codefence_renderers by exact key)",
        "QA-005: the code has no header_ids/header_attributes, so the output has no id= at all",
        "SEC-002 and QA-004 concern the Task 7 benchmark and the test threshold; nothing in the backend code addresses them",
        "residual risk: when syntect returns an error, comrak SyntectAdapter writes the code block unescaped; search errors in syntect 5.3 count as no match, so this is reachable only through a broken built-in grammar; no triggering input found",
        "insta was added to dev-dependencies per the FD; it is not yet used in tests (machete does not complain); downloading insta needed cargo outside the sandbox (~/.cargo cannot be written inside it)",
        "ponytail: unused ThemeSet loaded by SyntectAdapter::new — rejected: FR-4 requires the comrak adapter, and the cost is one-time",
        "ponytail: raw code on a syntect error — rejected as a code change: no triggering input; recorded as residual risk for the security review",
        "ponytail: 0/2 accepted"
      ]
    },
    {
      "at": "2026-10-25T12:14:46+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T12:14:47+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T12:14:47+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 8,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ]
    },
    {
      "at": "2026-10-25T12:14:54+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:14:54+00:00",
      "event": "note",
      "text": "диспатч post-checks; в «Новые тесты» — render_fragment.rs + insta-снапшоты"
    },
    {
      "at": "2026-10-25T12:30:07+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-002"
      ],
      "notes": [
        "clippy -D warnings и cargo test -p nimmd-core зелёные; смоук AC-1..AC-11 во временном крейте подтвердил поведение",
        "tests/render_fragment.rs и insta-снапшоты из карты реализации в правке пока отсутствуют — их пишет тестовая фаза"
      ]
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-003",
        "SEC-004"
      ],
      "notes": [
        "cargo deny check: advisories ok, bans ok, licenses ok, sources ok (run outside sandbox: ~/.cargo/advisory-dbs lock is read-only in sandbox)",
        "gitleaks dir over changed-files.txt: no leaks found",
        "semgrep .semgrep.yml over changed src: 0 findings; registry packs p/rust p/typescript p/javascript p/secrets failed to download (semgrep.dev connection error) -> SEC-004 NOTICE",
        "manual probes: javascript:/vbscript:/file:/data:text/html, wikilink, autolink, entity-encoded schemes -> empty href; raw HTML escaped; mermaid/math/alert/footnote/table content escaped; info-string attribute injection not reflected",
        "SEC-003: syntect Err -> comrak writes code raw; 26025 fuzz runs over all default grammars: 0 leaks, 0 panics; not reachable as found",
        "Snyk not connected, skipped; no workflows touched; no stack.isolation"
      ]
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Coverage: SC-001 table_alignment_and_tasks; SC-002 code_highlighted_with_classes; SC-003 code_without_or_with_unknown_language; SC-004 five_alert_types_ignore_case; SC-005 obsidian_callout_is_plain_blockquote; SC-006 math_and_prices; SC-007 mermaid_closed_unclosed_and_any_case; SC-008 raw_html_is_escaped; SC-009 dangerous_urls_dropped_safe_images_kept; SC-010 wikilink_and_footnotes; SC-011 leading_rule_block_kept; SC-012 detect_parses_body_after_frontmatter (+ cargo test -p nimmd-core --test detect green, 27 passed); SC-013 deep_nesting_is_fast; SC-014 ordered_marker_over_limit_escapes_punctuation; SC-015 leading_indent_is_capped (+ --test text green, 14 passed); SC-016 disabled_and_enabled_extensions; extra: render_is_total_on_random_input (xorshift seed 0x00C0FFEED00DF00D, 2000 inputs, no panic, no <script / <img src=x).",
        "Snapshots made with INSTA_UPDATE=always (cargo-insta is not installed) and checked by eye; the final run used the default INSTA_UPDATE and left no .snap.new files.",
        "Red run 1: git stash push -- crates/nimmd-core/src/text.rs crates/nimmd-core/src/detect.rs; cargo test -p nimmd-core --test render -> FAILED leading_indent_is_capped (left 300 spaces + '- x', right 256), ordered_marker_over_limit_escapes_punctuation (left '...1. \\\\1. ...', right '...1\\\\. ...'), deep_nesting_is_fast; stash pop, git diff --stat unchanged.",
        "Red run 2: only detect.rs stashed -> detect_parses_body_after_frontmatter FAILED: Profile { ... frontmatter: true, math: true ... } for '---\\ntitle: $a+b$\\n---\\nтекст'. The literal AC-9 input does not catch the regression on its own, so I added checks for math/mermaid inside the frontmatter.",
        "Red run 3: render.rs is untracked, so stash could not take it. I backed it up in tmp_dir, disabled the normalize_mermaid call and ran the test: mermaid_closed_unclosed_and_any_case FAILED (```Mermaid went to syntect). Restored the file and confirmed with cmp that it matches the backup.",
        "Red run 4, after the timing change: text.rs stashed -> deep_nesting_is_fast FAILED on the structural check (indent width 260 > 256). With the 5 s limit alone the timing check passed without the fix (ladder about 4.5 s), so the structural check is what catches the regression.",
        "Deviation from AC-10(в), sanctioned by NOTICE QA-004 and ARCH-002: the 800-line ladder takes 0.85-0.95 s in debug on Apple Silicon. 1 s would leave no margin (the render suite in parallel already takes 0.98 s), so its limit was raised to 5 s (5x margin). Inputs (а) and (б) keep the 1 s limit. If the AC must hold at exactly 1 s, the FD has to decide that.",
        "Checks: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings 0 issues; diff-lint.sh backend exit 0 (clippy-changed clean, cargo-machete clean, semgrep clean). Semgrep needed SEMGREP_LOG_FILE/SEMGREP_SETTINGS_FILE pointed at tmp_dir: the sandbox does not allow writes to ~/.semgrep. Full cargo test --workspace --all-targets green: 0+27+7+17+14 passed, 0 ignored.",
        "Not run: cargo deny check and just lint (network). These do not affect the test files.",
        "SEC-002 (hostile code block in the Task 7 benchmark) is outside the tests of this task: there are no benchmarks yet."
      ]
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный прогон не повторялся: control/suite-run.backend-tests.json result=green, tree f8bb12e98919ec4f6bacdff847829efe18c06feb совпадает с tree-hash.sh",
        "Точечно: cargo test -p nimmd-core --test render --test text --test detect — render 17/17, text 14/14, detect 27/27 ok, 0 ignored",
        "SC-013/AC-10: замер render_fragment в debug (временный крейт в control/.tmp/testing, --offline): (а) 5–43 мс, (б) <1 мс, (в) лестница 0,885–0,898 с — меньше 1 с; тест deep_nesting_is_fast держит для (в) порог 5 с (по QA-004/ARCH-002), запас к 1 с у кода ~10%",
        "SC-015: 65 табуляций проверены как 64 табуляции (ширина 256 колонок) в leading_indent_is_capped; инвариант FR-7 в limit_nesting_is_lossless_and_idempotent зелёный",
        "control/changed-files.docs.txt пуст — сверять руководство нечего"
      ]
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T12:30:08+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T12:30:51+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "e4d95c874ac8737740d55039474501baebb5d37eed89d92151c789534087e789",
      "sections": {
        "Метаданные": "0a4a09161c544bac20966e1f4818fdffefcd75908acb836e184b9c1537cebdd0",
        "Цель": "b898d843e61962a9a38cd58be545408b80dad78044cda8155de090c1ed02d364",
        "Границы": "8a7bd13cbf2689bd26f8b243d087ee3b36d017e919dde3dda84497d68c30d923",
        "Целевое поведение": "a6825f962bbbfd80c5decb676ee4d065e2abe07cb942e9bbc27fbe8adbfe9668",
        "Требования": "dad6025fbe4be91154ec35ff66569aa51c6901618d3e9557fb00473b466aa5de",
        "Данные, права, ошибки": "effe2dab72355018dec8fadf3163e2803af4ef0cd2b52cdcbf0e0bfca4657f1d",
        "Карта реализации": "58fa5db3e683a96d41338959b95c4ae6672b753ad4ee1fbd46e9ab015a2c7b8b",
        "Критерии приёмки": "b99851dbe3d9844590c0787dd890e90ef850a5bdea66458af4bebfccef7a58eb",
        "Влияние на руководство": "c0063e787dbfc3ab4d15172808687aac4002e98af9b993dfd0aaf8e3eebce3cd"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "55f7aa0a3b4665d243ad80f975963f20d43a9eeb16d7756d6edbda538d1e1821"
        },
        "Цель": {
          "Цель": "cf539782b21256be48eff682523b1c6d604c431eed8078880bb3421d8c6bc704"
        },
        "Границы": {
          "Границы": "d87ba3f0e869f590ca6bec0ebde93044f3a4143ad6bb1180b49bdeb8b5d09ce0"
        },
        "Целевое поведение": {
          "Целевое поведение": "6fcdf66ce26b3cfef76ebc13e75132495af7772752ddcb02795e84e2d3b3c653"
        },
        "Требования": {
          "FR-1": "f3e6b2f8caebbc578aeea2de54a207f2f7aafe254b1e93ceaebac25b829da224",
          "FR-2": "1a19ad951f2816d5363c860d1193cfbc922d23e937b98a849957526d541d99a7",
          "FR-3": "ed025fed00e8dcd369bb096953fa34ec43310afd9290b3887433d41b8dce432f",
          "FR-4": "8ca8aaf1b9e3ae5872e952b36b5d89f1384676325354fc811bc4b567857f0843",
          "FR-5": "90e5c80d6b2077cb0daeebb6a3de6aede3044396978b74951f3271cdca8a06e4",
          "FR-6": "ac64e25a528a30d9f0fdcc3098e99ca7e0ce40388901089c52fb9d9e6100091a",
          "FR-7": "9ed16469995981b495791e24ab81729a5b11a24cb73871f5859d559a52ae64c8"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "0e4686ec76398bc96dfbfb03b782365f18ab8e92ac63612d66e62bd9fff79b68"
        },
        "Карта реализации": {
          "Карта реализации": "ddaa8621745c4d3002dfcfe7f4eca5ef29d8b08bcef40ae7ab3b7aff4ce93d0e"
        },
        "Критерии приёмки": {
          "AC-1": "257a5b1a93ab659a7a22bdca2ed14e83467909e7832f37a284e2a88a802db407",
          "AC-2": "8c2e4f7cfd1bcdeda8cc236f6f2921616c22f206a3c1c553a4d44d2ee40199ba",
          "AC-3": "b44c38a88a681a599e17d92ccdf769643d33cc442ea3f5c5c7bf672a885ef309",
          "AC-4": "04dfb206708d98a3990a9dfb5eb17358df9b6acc28dafee9e93e76928c5da4db",
          "AC-5": "1dc527e661b2ea8174cfd27e11306939989cc731ce415a3a838d593108729345",
          "AC-6": "b9e0f45d6a73a6333e1d7de64b49ad3f9f643a9b068e19be50ef0bd6864a7591",
          "AC-7": "d48e114bb27d1604a8a8f2926bf6805e7a73991f2df91108ad810a3539b8dc5d",
          "AC-8": "aa35eba0db1039c46bfc4c7dfeec5a52c770c7ef66c98fa1aea32d35877e7878",
          "AC-9": "0621b119917485cd7f81ca0ccd1b3800f68374029939c5b9a161900c824765e1",
          "AC-10": "4084ea66d5393723a362bbe89fbcdcc6bed61deb78039fc4a81fe8406a76f2c3",
          "AC-11": "fca3dc39fcbd073caeeb638b3cd69300998b41e2df46b66d5b71efb8779b94da"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "bbdeed9a2754c7b769f4bea3aa6903f66f32a5e5f7cbf276c6b061fa41f4ff52"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "e4d95c874ac8737740d55039474501baebb5d37eed89d92151c789534087e789",
      "sections": {
        "Метаданные": "0a4a09161c544bac20966e1f4818fdffefcd75908acb836e184b9c1537cebdd0",
        "Цель": "b898d843e61962a9a38cd58be545408b80dad78044cda8155de090c1ed02d364",
        "Границы": "8a7bd13cbf2689bd26f8b243d087ee3b36d017e919dde3dda84497d68c30d923",
        "Целевое поведение": "a6825f962bbbfd80c5decb676ee4d065e2abe07cb942e9bbc27fbe8adbfe9668",
        "Требования": "dad6025fbe4be91154ec35ff66569aa51c6901618d3e9557fb00473b466aa5de",
        "Данные, права, ошибки": "effe2dab72355018dec8fadf3163e2803af4ef0cd2b52cdcbf0e0bfca4657f1d",
        "Карта реализации": "58fa5db3e683a96d41338959b95c4ae6672b753ad4ee1fbd46e9ab015a2c7b8b",
        "Критерии приёмки": "b99851dbe3d9844590c0787dd890e90ef850a5bdea66458af4bebfccef7a58eb",
        "Влияние на руководство": "c0063e787dbfc3ab4d15172808687aac4002e98af9b993dfd0aaf8e3eebce3cd"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "55f7aa0a3b4665d243ad80f975963f20d43a9eeb16d7756d6edbda538d1e1821"
        },
        "Цель": {
          "Цель": "cf539782b21256be48eff682523b1c6d604c431eed8078880bb3421d8c6bc704"
        },
        "Границы": {
          "Границы": "d87ba3f0e869f590ca6bec0ebde93044f3a4143ad6bb1180b49bdeb8b5d09ce0"
        },
        "Целевое поведение": {
          "Целевое поведение": "6fcdf66ce26b3cfef76ebc13e75132495af7772752ddcb02795e84e2d3b3c653"
        },
        "Требования": {
          "FR-1": "f3e6b2f8caebbc578aeea2de54a207f2f7aafe254b1e93ceaebac25b829da224",
          "FR-2": "1a19ad951f2816d5363c860d1193cfbc922d23e937b98a849957526d541d99a7",
          "FR-3": "ed025fed00e8dcd369bb096953fa34ec43310afd9290b3887433d41b8dce432f",
          "FR-4": "8ca8aaf1b9e3ae5872e952b36b5d89f1384676325354fc811bc4b567857f0843",
          "FR-5": "90e5c80d6b2077cb0daeebb6a3de6aede3044396978b74951f3271cdca8a06e4",
          "FR-6": "ac64e25a528a30d9f0fdcc3098e99ca7e0ce40388901089c52fb9d9e6100091a",
          "FR-7": "9ed16469995981b495791e24ab81729a5b11a24cb73871f5859d559a52ae64c8"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "0e4686ec76398bc96dfbfb03b782365f18ab8e92ac63612d66e62bd9fff79b68"
        },
        "Карта реализации": {
          "Карта реализации": "ddaa8621745c4d3002dfcfe7f4eca5ef29d8b08bcef40ae7ab3b7aff4ce93d0e"
        },
        "Критерии приёмки": {
          "AC-1": "257a5b1a93ab659a7a22bdca2ed14e83467909e7832f37a284e2a88a802db407",
          "AC-2": "8c2e4f7cfd1bcdeda8cc236f6f2921616c22f206a3c1c553a4d44d2ee40199ba",
          "AC-3": "b44c38a88a681a599e17d92ccdf769643d33cc442ea3f5c5c7bf672a885ef309",
          "AC-4": "04dfb206708d98a3990a9dfb5eb17358df9b6acc28dafee9e93e76928c5da4db",
          "AC-5": "1dc527e661b2ea8174cfd27e11306939989cc731ce415a3a838d593108729345",
          "AC-6": "b9e0f45d6a73a6333e1d7de64b49ad3f9f643a9b068e19be50ef0bd6864a7591",
          "AC-7": "d48e114bb27d1604a8a8f2926bf6805e7a73991f2df91108ad810a3539b8dc5d",
          "AC-8": "aa35eba0db1039c46bfc4c7dfeec5a52c770c7ef66c98fa1aea32d35877e7878",
          "AC-9": "0621b119917485cd7f81ca0ccd1b3800f68374029939c5b9a161900c824765e1",
          "AC-10": "4084ea66d5393723a362bbe89fbcdcc6bed61deb78039fc4a81fe8406a76f2c3",
          "AC-11": "fca3dc39fcbd073caeeb638b3cd69300998b41e2df46b66d5b71efb8779b94da"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "bbdeed9a2754c7b769f4bea3aa6903f66f32a5e5f7cbf276c6b061fa41f4ff52"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "e4d95c874ac8737740d55039474501baebb5d37eed89d92151c789534087e789",
      "sections": {
        "Метаданные": "0a4a09161c544bac20966e1f4818fdffefcd75908acb836e184b9c1537cebdd0",
        "Цель": "b898d843e61962a9a38cd58be545408b80dad78044cda8155de090c1ed02d364",
        "Границы": "8a7bd13cbf2689bd26f8b243d087ee3b36d017e919dde3dda84497d68c30d923",
        "Целевое поведение": "a6825f962bbbfd80c5decb676ee4d065e2abe07cb942e9bbc27fbe8adbfe9668",
        "Требования": "dad6025fbe4be91154ec35ff66569aa51c6901618d3e9557fb00473b466aa5de",
        "Данные, права, ошибки": "effe2dab72355018dec8fadf3163e2803af4ef0cd2b52cdcbf0e0bfca4657f1d",
        "Карта реализации": "58fa5db3e683a96d41338959b95c4ae6672b753ad4ee1fbd46e9ab015a2c7b8b",
        "Критерии приёмки": "b99851dbe3d9844590c0787dd890e90ef850a5bdea66458af4bebfccef7a58eb",
        "Влияние на руководство": "c0063e787dbfc3ab4d15172808687aac4002e98af9b993dfd0aaf8e3eebce3cd"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "55f7aa0a3b4665d243ad80f975963f20d43a9eeb16d7756d6edbda538d1e1821"
        },
        "Цель": {
          "Цель": "cf539782b21256be48eff682523b1c6d604c431eed8078880bb3421d8c6bc704"
        },
        "Границы": {
          "Границы": "d87ba3f0e869f590ca6bec0ebde93044f3a4143ad6bb1180b49bdeb8b5d09ce0"
        },
        "Целевое поведение": {
          "Целевое поведение": "6fcdf66ce26b3cfef76ebc13e75132495af7772752ddcb02795e84e2d3b3c653"
        },
        "Требования": {
          "FR-1": "f3e6b2f8caebbc578aeea2de54a207f2f7aafe254b1e93ceaebac25b829da224",
          "FR-2": "1a19ad951f2816d5363c860d1193cfbc922d23e937b98a849957526d541d99a7",
          "FR-3": "ed025fed00e8dcd369bb096953fa34ec43310afd9290b3887433d41b8dce432f",
          "FR-4": "8ca8aaf1b9e3ae5872e952b36b5d89f1384676325354fc811bc4b567857f0843",
          "FR-5": "90e5c80d6b2077cb0daeebb6a3de6aede3044396978b74951f3271cdca8a06e4",
          "FR-6": "ac64e25a528a30d9f0fdcc3098e99ca7e0ce40388901089c52fb9d9e6100091a",
          "FR-7": "9ed16469995981b495791e24ab81729a5b11a24cb73871f5859d559a52ae64c8"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "0e4686ec76398bc96dfbfb03b782365f18ab8e92ac63612d66e62bd9fff79b68"
        },
        "Карта реализации": {
          "Карта реализации": "ddaa8621745c4d3002dfcfe7f4eca5ef29d8b08bcef40ae7ab3b7aff4ce93d0e"
        },
        "Критерии приёмки": {
          "AC-1": "257a5b1a93ab659a7a22bdca2ed14e83467909e7832f37a284e2a88a802db407",
          "AC-2": "8c2e4f7cfd1bcdeda8cc236f6f2921616c22f206a3c1c553a4d44d2ee40199ba",
          "AC-3": "b44c38a88a681a599e17d92ccdf769643d33cc442ea3f5c5c7bf672a885ef309",
          "AC-4": "04dfb206708d98a3990a9dfb5eb17358df9b6acc28dafee9e93e76928c5da4db",
          "AC-5": "1dc527e661b2ea8174cfd27e11306939989cc731ce415a3a838d593108729345",
          "AC-6": "b9e0f45d6a73a6333e1d7de64b49ad3f9f643a9b068e19be50ef0bd6864a7591",
          "AC-7": "d48e114bb27d1604a8a8f2926bf6805e7a73991f2df91108ad810a3539b8dc5d",
          "AC-8": "aa35eba0db1039c46bfc4c7dfeec5a52c770c7ef66c98fa1aea32d35877e7878",
          "AC-9": "0621b119917485cd7f81ca0ccd1b3800f68374029939c5b9a161900c824765e1",
          "AC-10": "4084ea66d5393723a362bbe89fbcdcc6bed61deb78039fc4a81fe8406a76f2c3",
          "AC-11": "fca3dc39fcbd073caeeb638b3cd69300998b41e2df46b66d5b71efb8779b94da"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "bbdeed9a2754c7b769f4bea3aa6903f66f32a5e5f7cbf276c6b061fa41f4ff52"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "e4d95c874ac8737740d55039474501baebb5d37eed89d92151c789534087e789",
    "backend_tests": "e4d95c874ac8737740d55039474501baebb5d37eed89d92151c789534087e789"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства в репозитории ещё нет, фрагмент виден только через хосты (#43)."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "05a1b113807af1b2f12e7180c00111a6a9e4c85e46814a4947e6173ef4014bc9",
      "files": [
        "Cargo.lock",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/tests/text.rs",
        "crates/nimmd-core/tests/render.rs",
        "crates/nimmd-core/tests/snapshots/render__code_highlighted_with_classes.snap",
        "crates/nimmd-core/tests/snapshots/render__table_alignment_and_tasks.snap",
        "crates/nimmd-core/tests/snapshots/render__wikilink_and_footnotes.snap"
      ],
      "production_hash": "dd9197ffec558b3a6faffa054e64e0f1faa71e5afa3cf02e9c4ff4cfb09d7111",
      "production_files": [
        "Cargo.lock",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "05a1b113807af1b2f12e7180c00111a6a9e4c85e46814a4947e6173ef4014bc9",
      "files": [
        "Cargo.lock",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/tests/text.rs",
        "crates/nimmd-core/tests/render.rs",
        "crates/nimmd-core/tests/snapshots/render__code_highlighted_with_classes.snap",
        "crates/nimmd-core/tests/snapshots/render__table_alignment_and_tasks.snap",
        "crates/nimmd-core/tests/snapshots/render__wikilink_and_footnotes.snap"
      ],
      "production_hash": "dd9197ffec558b3a6faffa054e64e0f1faa71e5afa3cf02e9c4ff4cfb09d7111",
      "production_files": [
        "Cargo.lock",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs"
      ]
    }
  },
  "accepted_code_hash": {
    "backend-tests": "35dd9a491744673ff2bc131c3b4768caef12377729502da995fac8bbe54c43fa",
    "testing": "35dd9a491744673ff2bc131c3b4768caef12377729502da995fac8bbe54c43fa"
  }
}
```
<!-- afd-state-snapshot:end -->
