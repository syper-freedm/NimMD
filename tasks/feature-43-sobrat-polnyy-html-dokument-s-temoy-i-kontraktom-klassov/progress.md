# Состояние разработки: Собрать полный HTML-документ с темой и контрактом классов

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #43 (syper-freedm/NimMD)
- Размер: M; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T13:06:09+00:00

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

- 2026-10-25T12:50:28+00:00 `routed` — {"post_checks_parallel": false, "files_count": 13, "has_migrations": false, "touches_isolation": false, "skip": ["documentation"]}
- 2026-10-25T12:50:33+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-25T12:50:33+00:00 `note` — {"text": "диспатч post-checks"}
- 2026-10-25T13:05:31+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": ["ARCH-004", "ARCH-005", "ARCH-006"]}
- 2026-10-25T13:05:31+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": ["SEC-004"], "notes": ["cargo deny check: advisories/bans/licenses/sources ok (run outside sandbox: advisory DB lock in ~/.cargo is read-only in sandbox)", "gitleaks dir over control/changed-files.txt: no leaks; semgrep .semgrep.yml: clean; registry rulesets unavailable (semgrep.dev connection error) -> SEC-004 NOTICE", "Manual: latex2mathml echoes only Letter tokens into text, '<'/'&' sanitized by escape_stray; \\text/\\operatorname with markup fall back to escaped Source; title, frontmatter, base_url escaped; Inline scripts close_safe; no document text in scripts"]}
- 2026-10-25T13:05:31+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Coverage: SC-001 empty_and_blank_input; SC-002 bom_crlf_frontmatter_and_title (+frontmatter_text_is_escaped, title_from_first_level_one_heading); SC-003 theme_attribute_and_single_style; SC-004 body_is_substring_and_profile_independent; SC-005 profile_matches_detect_on_fixtures (+profile_matches_detect_on_random_input, explicit_profile_wins_over_detector); SC-006 remote_images_replaced_unless_allowed (+remote_image_edge_urls); SC-007 math_katex_mode_keeps_spans, math_mathml_mode (insta snapshot), math_source_mode, dangerous_formulas_are_escaped_in_every_mode, deep_formulas_fall_back_to_source_in_mathml, unsupported_formula_in_mathml_shows_source; SC-008 asset_mode_none_has_no_scripts, asset_mode_link_follows_profile, katex_assets_only_in_katex_mode; SC-009 inline_assets_escape_closing_tags, link_base_url_is_escaped; SC-010 truncated_banner_after_fragment, not_utf8_banner_first_in_main; SC-011 class_contract, contract_core_classes_appear_in_output, css_selector_parser_sees_classes; plus rendered_serializes_lowercase, defaults_match_design, render_is_total_on_random_input (xorshift seed 0x00DE_C0DE_F00D_BEEF).", "QA-002 accounted for: deep_formulas_fall_back_to_source_in_mathml checks both D (nesting 10 000, length limit) and N (nesting MAX_NESTING+1 = 65, 131 bytes, depth limit), plus MAX_NESTING/2 still yields <math>.", "Issue is a feature (not bug): stash-red-run of the whole change would only break compilation, so red evidence was gathered by single-line mutations of production code (backups in tmp_dir, restored; shasum of src/*.rs and git diff --stat identical before/after).", "Mutation M1 (drop !within_depth in mathml): deep_formulas_fall_back_to_source_in_mathml fails at document.rs:407 assert_eq!(mathml, source); unsupported_formula_in_mathml_shows_source panics in latex2mathml parse.rs:297 (\\middle).", "Mutation M2 (escape_stray -> identity): dangerous_formulas_are_escaped_in_every_mode fails at document.rs:358 (char after '<' not a letter or '/').", "Mutation M3 (is_remote without '//' branch): remote_images_replaced_unless_allowed fails at document.rs:254 (<img count != 1); render_is_total_on_random_input fails.", "Mutation M4 (close_safe -> identity): inline_assets_escape_closing_tags fails at document.rs:493. M5 (base_url unescaped): link_base_url_is_escaped fails at document.rs:511. M6 (katex flag from parsed AST instead of profile+mode): explicit_profile_wins_over_detector and katex_assets_only_in_katex_mode fail.", "Checks run: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings OK; diff-lint.sh backend exit 0 (clippy-changed clean, cargo-machete clean, semgrep-project-rules clean); cargo test --workspace --all-targets green: 95 passed, 0 failed, 0 ignored (document.rs 30).", "semgrep under the sandbox can't write ~/.semgrep/semgrep.log; ran with SEMGREP_LOG_FILE/SEMGREP_SETTINGS_FILE pointing into tmp_dir.", "Observation (not a defect per FR-6): comrak percent-encodes a backslash in an image URL (\\\\h/p.png -> %5Ch/p.png), so the backslash branch of is_remote never fires on the url field; the test does not assert it.", "Observation: title skips inline raw HTML (`# a <b>` gives title 'a '), consistent with FR-4 (text and code nodes only); the escaping test uses \\< escapes."]}
- 2026-10-25T13:05:31+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный набор не перезапускался: control/suite-run.backend-tests.json result=green, tree=1f228f760219ad6f18a4c22114c4e0892344d1c9 = текущий tree-hash.sh", "Фокусный прогон: cargo test -p nimmd-core --test document — 30 passed, 0 failed, 0 ignored; .snap.new нет", "SC-001 empty_and_blank_input; SC-002 bom_crlf_frontmatter_and_title; SC-003 theme_attribute_and_single_style; SC-004 body_is_substring_and_profile_independent; SC-005 profile_matches_detect_on_fixtures", "SC-006 remote_images_replaced_unless_allowed; SC-007 math_katex_mode_keeps_spans, math_mathml_mode (снапшот document__math_mathml_mode.snap), math_source_mode, dangerous_formulas_are_escaped_in_every_mode, deep_formulas_fall_back_to_source_in_mathml (10 000 и MAX_NESTING+1=65)", "SC-008 asset_mode_none_has_no_scripts, asset_mode_link_follows_profile; SC-009 inline_assets_escape_closing_tags, link_base_url_is_escaped; SC-010 truncated_banner_after_fragment, not_utf8_banner_first_in_main; SC-011 class_contract", "stack.stress_runs нет; changed-files.docs.txt пуст — сверка руководства не требуется; frontend-тестов нет"]}
- 2026-10-25T13:05:54+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-25T13:05:54+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T13:05:54+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T13:05:54+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T13:05:54+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T13:05:54+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T13:05:55+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T13:06:09+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 43,
  "title": "Собрать полный HTML-документ с темой и контрактом классов",
  "slug": "sobrat-polnyy-html-dokument-s-temoy-i-kontraktom-klassov",
  "branch": "feat/43-sobrat-polnyy-html-dokument-s-temoy-i",
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
      "at": "2026-10-25T12:32:05+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T12:33:51+00:00",
      "event": "classified",
      "size": "M",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T12:33:51+00:00",
      "event": "note",
      "text": "ФД написан; размер L из GitHub переоценён в M (один модуль + CSS); продуктовых развилок нет"
    },
    {
      "at": "2026-10-25T12:33:51+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T12:33:52+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:33:52+00:00",
      "event": "note",
      "text": "диспатч FD-ревью"
    },
    {
      "at": "2026-10-25T12:36:03+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002",
        "ARCH-003"
      ],
      "notes": [
        "Сверено с кодом: ast_features уже pub(crate) в detect.rs, parse::options() общий, Profile имеет Serialize, latex2mathml есть в [workspace.dependencies]; comrak 0.56 даёт create_formatter! с format_document_with_plugins для FR-6/FR-7.",
        "Не проверено: экранирует ли latex2mathml \\text{<script>} и паникует ли на битом TeX (исходника нет в реестре) — это зона ИБ/QA, FR-7 требование сформулировано явно.",
        "architecture.md не создан: контракт полностью зафиксирован в ФД."
      ]
    },
    {
      "at": "2026-10-25T12:36:03+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "BLOCKER",
      "findings": [
        "SEC-001",
        "SEC-002"
      ],
      "notes": [
        "SEC-001: FR-6 prefix check is bypassed by scheme-without-slashes URLs (https:host/p.png); WHATWG URL parsing turns them into remote requests.",
        "SEC-002: latex2mathml source was not available locally, so its escaping and recursion depth are not verified.",
        "security.md not created: no new permissions, sensitive data classes or protected flows.",
        "The CSP meta in the document head was not checked against the spec: docs/ is gitignored and off limits."
      ]
    },
    {
      "at": "2026-10-25T12:36:03+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001"
      ],
      "notes": [
        "Все AC-1…AC-10 наблюдаемы и детерминированы; утверждения ФД сверены с кодом (text.rs, detect.rs, render.rs, parse.rs: tasklist_classes, alerts, math_dollars; latex2mathml в workspace Cargo.toml; фикстуры tests/fixtures/**).",
        "QA-001 NOTICE: AC-10 — для проверки всех классов общего списка эталонный документ должен содержать алерты всех пяти видов; учтено в SC-011.",
        "Ролей и изоляции данных нет (библиотека ядра) — сценарии доступа к объектам неприменимы.",
        "test-cases.md: SC-001…SC-011, уровень integration, файл crates/nimmd-core/tests/document.rs."
      ]
    },
    {
      "at": "2026-10-25T12:36:20+00:00",
      "event": "note",
      "text": "SEC-001: принято автором ФД без скептика — обход префикса по схеме без слэшей очевиден по WHATWG URL"
    },
    {
      "at": "2026-10-25T12:36:21+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "requirements_rework",
      "reason": "SEC-001"
    },
    {
      "at": "2026-10-25T12:36:21+00:00",
      "event": "transition",
      "from": "requirements_rework",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T12:36:27+00:00",
      "event": "note",
      "text": "confirm-круг FD после SEC-001"
    },
    {
      "at": "2026-10-25T12:38:50+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002",
        "ARCH-003"
      ],
      "notes": [
        "Режим дельты: правки FR-3, FR-8, FR-10 и карты реализации закрывают NOTICE ARCH-001, ARCH-002 и ARCH-003; правки FR-6, FR-7, AC-6, AC-7 и AC-10 новых архитектурных дефектов не вносят.",
        "Сверено: text::MAX_NESTING (64) уже есть в crates/nimmd-core/src/text.rs; JS-правила .semgrep.yml исключают только packages/vendor и node_modules, поэтому packages/theme/*.js они проверяют."
      ]
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-003"
      ],
      "notes": [
        "Дельта FR-3/6/7/8/10, AC-6/7/10 закрывает SEC-001 (схема http/https по декодированному url) и SEC-002 (лимит длины/вложенности до latex2mathml, расширенный AC-7).",
        "SEC-003 NOTICE: рекурсия latex2mathml по аргументам без скобок не сверена — крейта ещё нет в Cargo.lock."
      ]
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-002"
      ],
      "notes": [
        "Дельта: FR-3, FR-6, FR-7, FR-8, FR-10, AC-6, AC-7, AC-10 перечитаны; все AC остаются наблюдаемыми и детерминированными.",
        "QA-001 снят: AC-10 теперь требует алерты всех пяти видов.",
        "QA-002 NOTICE: AC-7 — вход вложенности 10 000 упирается в предел длины 10 000 байт, ветка MAX_NESTING (=64, text.rs:108) не проверяется; в SC-007 добавлен вход N (вложенность 65).",
        "test-cases.md: SC-006 — восемь элементов (+f, g), remote-image x6, img x7 при true; SC-007 — опасные формулы (проверка <, & и имён тегов), D и N в MathML = body Source; SC-003 (один <style> при assets None), SC-008 (скрипты из packages/theme/*.js), SC-011 (contains-task-list из CONTRACT.md) остаются верны."
      ]
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:38:51+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:38:52+00:00",
      "event": "note",
      "text": "ФД одобрен; SEC-003 (рекурсия latex2mathml без скобок) — разработчик обязан замерить \\frac/\\sqrt × 2000 в потоке 512 КБ и при падении сузить лимит; диспатч backend"
    },
    {
      "at": "2026-10-25T12:50:27+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "self-checks: cargo fmt --check OK; cargo build OK; cargo clippy -D warnings OK; cargo test --workspace --all-targets: 65 passed, 0 failed (27+7+17+14); cargo deny check: advisories/bans/licenses/sources ok; just semgrep: scan clean, rule tests passed; cargo machete: clean; diff-lint backend: clippy-changed clean, semgrep-project-rules clean",
        "semgrep, cargo deny and diff-lint ran outside the sandbox: semgrep writes ~/.semgrep/semgrep.log and cargo fetch/deny need the network; gitleaks not run (git mode scans history only, the change is not committed)",
        "AC smoke check (scratch crate in control/.tmp/backend/smoke, not in the repo): AC-1..AC-10 verified by assertions, including SC-006 (1 img / 6 remote-image; 7 img), SC-009, AC-5 on 7 fixtures, nesting 65 and 10000 and \\sqrt x2000 in MathML -> Source without panic, nesting 64 -> <math",
        "SEC-003 applied: the latex2mathml parser recurses without braces (\\sqrt, \\frac, styles, ^, _, BigOp...). within_depth counts every recursion-triggering token ({, ^, _, any command other than leaf tokens per latex2mathml::token::Token::from_command), sound upper bound <= MAX_NESTING. Measured: debug ~24 KB stack per level (2 MB holds ~83), release ~1 KB",
        "Deviation from the letter of FR-7 (stricter): the limit applies to the TOTAL count of such tokens, not to nesting depth; a flat formula with >64 {/^/_/argument commands goes to Source in MathML mode. An exact depth estimate requires mirroring the latex2mathml parser (parse_text swallows tokens, see render.rs); a simpler one undercounts on {\\text}{\\text}...",
        "Beyond the FD, for safety: latex2mathml 0.2.3 panics (unimplemented!) on \\middle with a non-bracket argument, so a formula with \\middle goes to Source; output containing [PARSE ERROR (unknown commands) goes to Source; latex2mathml does not escape text, so escape_stray escapes < not followed by a letter or / and & not starting an entity (a<b -> <mo>&lt;</mo>)",
        "Beyond the FD (FR-6): a remote address also includes a leading backslash pair (\\\\host, /\\host) and leading control/space characters are stripped, because browsers treat \\ as / and strip those characters",
        "QA-002 is for afd-test-developer (input N with x65 <= 10000 bytes); the implementation handles it (65 > MAX_NESTING -> Source)",
        "CONTRACT.md lists nimmd-data and data as core-only per FR-10, but the core does not produce them yet; the AC-10 test checks only the shared list and CSS selectors, nimmd.css does not use them",
        "LineEnding is serialized as lf/crlf (rename_all lowercase), consistent with the other enums; MathMode::MathML -> mathml",
        "clippy.toml: KaTeX and MathML added to doc-valid-idents (clippy doc_markdown)",
        "ponytail: duplicated dark palette in nimmd.css/highlight.css — rejected: plain CSS cannot share one block between a media query and a selector without a build step",
        "ponytail: 0/1 accepted"
      ]
    },
    {
      "at": "2026-10-25T12:50:27+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T12:50:28+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T12:50:28+00:00",
      "event": "routed",
      "post_checks_parallel": false,
      "files_count": 13,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ]
    },
    {
      "at": "2026-10-25T12:50:33+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T12:50:33+00:00",
      "event": "note",
      "text": "диспатч post-checks"
    },
    {
      "at": "2026-10-25T13:05:31+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-004",
        "ARCH-005",
        "ARCH-006"
      ]
    },
    {
      "at": "2026-10-25T13:05:31+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-004"
      ],
      "notes": [
        "cargo deny check: advisories/bans/licenses/sources ok (run outside sandbox: advisory DB lock in ~/.cargo is read-only in sandbox)",
        "gitleaks dir over control/changed-files.txt: no leaks; semgrep .semgrep.yml: clean; registry rulesets unavailable (semgrep.dev connection error) -> SEC-004 NOTICE",
        "Manual: latex2mathml echoes only Letter tokens into text, '<'/'&' sanitized by escape_stray; \\text/\\operatorname with markup fall back to escaped Source; title, frontmatter, base_url escaped; Inline scripts close_safe; no document text in scripts"
      ]
    },
    {
      "at": "2026-10-25T13:05:31+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Coverage: SC-001 empty_and_blank_input; SC-002 bom_crlf_frontmatter_and_title (+frontmatter_text_is_escaped, title_from_first_level_one_heading); SC-003 theme_attribute_and_single_style; SC-004 body_is_substring_and_profile_independent; SC-005 profile_matches_detect_on_fixtures (+profile_matches_detect_on_random_input, explicit_profile_wins_over_detector); SC-006 remote_images_replaced_unless_allowed (+remote_image_edge_urls); SC-007 math_katex_mode_keeps_spans, math_mathml_mode (insta snapshot), math_source_mode, dangerous_formulas_are_escaped_in_every_mode, deep_formulas_fall_back_to_source_in_mathml, unsupported_formula_in_mathml_shows_source; SC-008 asset_mode_none_has_no_scripts, asset_mode_link_follows_profile, katex_assets_only_in_katex_mode; SC-009 inline_assets_escape_closing_tags, link_base_url_is_escaped; SC-010 truncated_banner_after_fragment, not_utf8_banner_first_in_main; SC-011 class_contract, contract_core_classes_appear_in_output, css_selector_parser_sees_classes; plus rendered_serializes_lowercase, defaults_match_design, render_is_total_on_random_input (xorshift seed 0x00DE_C0DE_F00D_BEEF).",
        "QA-002 accounted for: deep_formulas_fall_back_to_source_in_mathml checks both D (nesting 10 000, length limit) and N (nesting MAX_NESTING+1 = 65, 131 bytes, depth limit), plus MAX_NESTING/2 still yields <math>.",
        "Issue is a feature (not bug): stash-red-run of the whole change would only break compilation, so red evidence was gathered by single-line mutations of production code (backups in tmp_dir, restored; shasum of src/*.rs and git diff --stat identical before/after).",
        "Mutation M1 (drop !within_depth in mathml): deep_formulas_fall_back_to_source_in_mathml fails at document.rs:407 assert_eq!(mathml, source); unsupported_formula_in_mathml_shows_source panics in latex2mathml parse.rs:297 (\\middle).",
        "Mutation M2 (escape_stray -> identity): dangerous_formulas_are_escaped_in_every_mode fails at document.rs:358 (char after '<' not a letter or '/').",
        "Mutation M3 (is_remote without '//' branch): remote_images_replaced_unless_allowed fails at document.rs:254 (<img count != 1); render_is_total_on_random_input fails.",
        "Mutation M4 (close_safe -> identity): inline_assets_escape_closing_tags fails at document.rs:493. M5 (base_url unescaped): link_base_url_is_escaped fails at document.rs:511. M6 (katex flag from parsed AST instead of profile+mode): explicit_profile_wins_over_detector and katex_assets_only_in_katex_mode fail.",
        "Checks run: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings OK; diff-lint.sh backend exit 0 (clippy-changed clean, cargo-machete clean, semgrep-project-rules clean); cargo test --workspace --all-targets green: 95 passed, 0 failed, 0 ignored (document.rs 30).",
        "semgrep under the sandbox can't write ~/.semgrep/semgrep.log; ran with SEMGREP_LOG_FILE/SEMGREP_SETTINGS_FILE pointing into tmp_dir.",
        "Observation (not a defect per FR-6): comrak percent-encodes a backslash in an image URL (\\\\h/p.png -> %5Ch/p.png), so the backslash branch of is_remote never fires on the url field; the test does not assert it.",
        "Observation: title skips inline raw HTML (`# a <b>` gives title 'a '), consistent with FR-4 (text and code nodes only); the escaping test uses \\< escapes."
      ]
    },
    {
      "at": "2026-10-25T13:05:31+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный набор не перезапускался: control/suite-run.backend-tests.json result=green, tree=1f228f760219ad6f18a4c22114c4e0892344d1c9 = текущий tree-hash.sh",
        "Фокусный прогон: cargo test -p nimmd-core --test document — 30 passed, 0 failed, 0 ignored; .snap.new нет",
        "SC-001 empty_and_blank_input; SC-002 bom_crlf_frontmatter_and_title; SC-003 theme_attribute_and_single_style; SC-004 body_is_substring_and_profile_independent; SC-005 profile_matches_detect_on_fixtures",
        "SC-006 remote_images_replaced_unless_allowed; SC-007 math_katex_mode_keeps_spans, math_mathml_mode (снапшот document__math_mathml_mode.snap), math_source_mode, dangerous_formulas_are_escaped_in_every_mode, deep_formulas_fall_back_to_source_in_mathml (10 000 и MAX_NESTING+1=65)",
        "SC-008 asset_mode_none_has_no_scripts, asset_mode_link_follows_profile; SC-009 inline_assets_escape_closing_tags, link_base_url_is_escaped; SC-010 truncated_banner_after_fragment, not_utf8_banner_first_in_main; SC-011 class_contract",
        "stack.stress_runs нет; changed-files.docs.txt пуст — сверка руководства не требуется; frontend-тестов нет"
      ]
    },
    {
      "at": "2026-10-25T13:05:54+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-25T13:05:54+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T13:05:54+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T13:05:54+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T13:05:54+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T13:05:54+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T13:05:55+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T13:06:09+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "d36b430fd294b600dbc6cbbd481c39535027778c96524de8ebe82de0c4399729",
      "sections": {
        "Метаданные": "d74255c6096b481705f2954d6995c1a877141411e425226ac2a916db25a482f6",
        "Цель": "0c364c3e9e09a7feabe640f5c93a39f9bee6b7d7b66f45ed4aeea35f7e9282c0",
        "Границы": "9064682c7ff284d2089f50ec26ffe7f27b771c77e1f6c2fa741eb4a91e6b2d55",
        "Целевое поведение": "a4da526a921141c53b4704c713899eb7f55fd535e30b31862f32da538c7e885a",
        "Требования": "cc7e41d179fe49fab8dda6b7f4b7e9a0ac5f1ce878d7f690d7d24efc0c5b9513",
        "Данные, права, ошибки": "f3c6f665a5658a0fb4a994f1886fe10e74df2fbcce5419442431846b02e3bf93",
        "Карта реализации": "921a0e22aa5226c8a6da269c5744632eb6656310e83a8adc95de4309897386a1",
        "Критерии приёмки": "aa8c527675cb8764e71c301c080809d8a95b8daa7d33adbfae49364605a0d398",
        "Влияние на руководство": "ca530aeffa226f8e56490ec60437005ee6c3f0f06e7b92e76ab85892bc1ed185"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "bec88e40988e36ea0358ef5c88281100422b1066b17535e2b59b263dc576338b"
        },
        "Цель": {
          "Цель": "e73157d7fb7dc9bba077a389c5d3c876f50dd9a9167cd6ce1f55cb156f4d15de"
        },
        "Границы": {
          "Границы": "f73e0ed13620512308521f5757d8fc3cb6b671562806c66937e7548a48d3b433"
        },
        "Целевое поведение": {
          "Целевое поведение": "6d39cb5e69af25459e99780d1ff148deb8002042cd24c8fd6046c92607059e75"
        },
        "Требования": {
          "FR-1": "d71c83edb2f0ffedb835b02bac530f2545c57aee33e4a3ec530722099ff467de",
          "FR-2": "c26c8f884297f33addfdbd6745aac42886ad590bdeab824afe087a051f38624a",
          "FR-3": "118f8f3723affffe42d49c2c02c53f422933b9493d65e84b9ecfe1e5df212d71",
          "FR-4": "7f22dc5ff357d4b74f52fe4678d130fdadc5347e9a7bcebf6de051cd4f14df04",
          "FR-5": "fa1028c9092af90d567556ce5b3a16f011640e4007c5105ae9e081c820600c6a",
          "FR-6": "4d6f397826611820f9aa24ac8e2ec8d857a22f372987f28c4e46252da78261b2",
          "FR-7": "d59ccd177324c29cc209066d59b25eb2a805b6636419680b4ccb5f2034c0ebdb",
          "FR-8": "3b47036fe6089724c88d2c705672a2390be12a4523f741d1baa1b4e684de4b03",
          "FR-9": "1f051e99829c161195392322fb325503a707e66c04edbc64ce177a386abc18d1",
          "FR-10": "4e210e6fcf22205cc1b93cd1b24fcefd6b250a47940fb647a40382ca0a86cd85"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "96cecd61c032d10818c3d0e5567e03821177b05db12a22357fa39d3e3b2f7d16"
        },
        "Карта реализации": {
          "Карта реализации": "5b4a93ddfc7323578fcf124778bc6c6924de5b4ffb8b351f244b846c18cc8b62"
        },
        "Критерии приёмки": {
          "AC-1": "33eef3f3fb0ded6d509e17258080923ec6cd155199c1cb7eee3ad10f3e03307a",
          "AC-2": "f49f52aa508dc79d0b4e13fdaf628b8be9b6249863d6f5f9e0ddd4411323f548",
          "AC-3": "b049a0d673af0ac71487509a52b50d69c89520b84f51b7ac9124deb5daced20f",
          "AC-4": "564e1f60808cce70c57aa98a2cd48a121c27876c4f9f290b44e0b711faf83de2",
          "AC-5": "9416abead1f18b5b39742a15f57c0fbcf89536725c33658c9eca40f83d173bed",
          "AC-6": "c26e79a548d038b13e949f983affbba8565766f02d2303e882bcce19dd911b81",
          "AC-7": "7ab888fb76a2e496ee55bd5e8dcec3efd3dad0356cc4e72291a56720c7038505",
          "AC-8": "85ee699f6ebb427d8e7f594544f2b92e39fabcf34630e797e251f5dd01ba8447",
          "AC-9": "e53a47581ee41ac01525fe468e14f2a547b0118759d2a3a86b106605aaa0b383",
          "AC-10": "877fa44e5eeba77517a18b9a9df3ed30920014cf97b33b9fe9d4d998d9ac16d9"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "75ad1b4efffabc7535761dfb2f02d091f1c51b81c5781efba1afab27be9eb0d4"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "d36b430fd294b600dbc6cbbd481c39535027778c96524de8ebe82de0c4399729",
      "sections": {
        "Метаданные": "d74255c6096b481705f2954d6995c1a877141411e425226ac2a916db25a482f6",
        "Цель": "0c364c3e9e09a7feabe640f5c93a39f9bee6b7d7b66f45ed4aeea35f7e9282c0",
        "Границы": "9064682c7ff284d2089f50ec26ffe7f27b771c77e1f6c2fa741eb4a91e6b2d55",
        "Целевое поведение": "a4da526a921141c53b4704c713899eb7f55fd535e30b31862f32da538c7e885a",
        "Требования": "cc7e41d179fe49fab8dda6b7f4b7e9a0ac5f1ce878d7f690d7d24efc0c5b9513",
        "Данные, права, ошибки": "f3c6f665a5658a0fb4a994f1886fe10e74df2fbcce5419442431846b02e3bf93",
        "Карта реализации": "921a0e22aa5226c8a6da269c5744632eb6656310e83a8adc95de4309897386a1",
        "Критерии приёмки": "aa8c527675cb8764e71c301c080809d8a95b8daa7d33adbfae49364605a0d398",
        "Влияние на руководство": "ca530aeffa226f8e56490ec60437005ee6c3f0f06e7b92e76ab85892bc1ed185"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "bec88e40988e36ea0358ef5c88281100422b1066b17535e2b59b263dc576338b"
        },
        "Цель": {
          "Цель": "e73157d7fb7dc9bba077a389c5d3c876f50dd9a9167cd6ce1f55cb156f4d15de"
        },
        "Границы": {
          "Границы": "f73e0ed13620512308521f5757d8fc3cb6b671562806c66937e7548a48d3b433"
        },
        "Целевое поведение": {
          "Целевое поведение": "6d39cb5e69af25459e99780d1ff148deb8002042cd24c8fd6046c92607059e75"
        },
        "Требования": {
          "FR-1": "d71c83edb2f0ffedb835b02bac530f2545c57aee33e4a3ec530722099ff467de",
          "FR-2": "c26c8f884297f33addfdbd6745aac42886ad590bdeab824afe087a051f38624a",
          "FR-3": "118f8f3723affffe42d49c2c02c53f422933b9493d65e84b9ecfe1e5df212d71",
          "FR-4": "7f22dc5ff357d4b74f52fe4678d130fdadc5347e9a7bcebf6de051cd4f14df04",
          "FR-5": "fa1028c9092af90d567556ce5b3a16f011640e4007c5105ae9e081c820600c6a",
          "FR-6": "4d6f397826611820f9aa24ac8e2ec8d857a22f372987f28c4e46252da78261b2",
          "FR-7": "d59ccd177324c29cc209066d59b25eb2a805b6636419680b4ccb5f2034c0ebdb",
          "FR-8": "3b47036fe6089724c88d2c705672a2390be12a4523f741d1baa1b4e684de4b03",
          "FR-9": "1f051e99829c161195392322fb325503a707e66c04edbc64ce177a386abc18d1",
          "FR-10": "4e210e6fcf22205cc1b93cd1b24fcefd6b250a47940fb647a40382ca0a86cd85"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "96cecd61c032d10818c3d0e5567e03821177b05db12a22357fa39d3e3b2f7d16"
        },
        "Карта реализации": {
          "Карта реализации": "5b4a93ddfc7323578fcf124778bc6c6924de5b4ffb8b351f244b846c18cc8b62"
        },
        "Критерии приёмки": {
          "AC-1": "33eef3f3fb0ded6d509e17258080923ec6cd155199c1cb7eee3ad10f3e03307a",
          "AC-2": "f49f52aa508dc79d0b4e13fdaf628b8be9b6249863d6f5f9e0ddd4411323f548",
          "AC-3": "b049a0d673af0ac71487509a52b50d69c89520b84f51b7ac9124deb5daced20f",
          "AC-4": "564e1f60808cce70c57aa98a2cd48a121c27876c4f9f290b44e0b711faf83de2",
          "AC-5": "9416abead1f18b5b39742a15f57c0fbcf89536725c33658c9eca40f83d173bed",
          "AC-6": "c26e79a548d038b13e949f983affbba8565766f02d2303e882bcce19dd911b81",
          "AC-7": "7ab888fb76a2e496ee55bd5e8dcec3efd3dad0356cc4e72291a56720c7038505",
          "AC-8": "85ee699f6ebb427d8e7f594544f2b92e39fabcf34630e797e251f5dd01ba8447",
          "AC-9": "e53a47581ee41ac01525fe468e14f2a547b0118759d2a3a86b106605aaa0b383",
          "AC-10": "877fa44e5eeba77517a18b9a9df3ed30920014cf97b33b9fe9d4d998d9ac16d9"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "75ad1b4efffabc7535761dfb2f02d091f1c51b81c5781efba1afab27be9eb0d4"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "d36b430fd294b600dbc6cbbd481c39535027778c96524de8ebe82de0c4399729",
      "sections": {
        "Метаданные": "d74255c6096b481705f2954d6995c1a877141411e425226ac2a916db25a482f6",
        "Цель": "0c364c3e9e09a7feabe640f5c93a39f9bee6b7d7b66f45ed4aeea35f7e9282c0",
        "Границы": "9064682c7ff284d2089f50ec26ffe7f27b771c77e1f6c2fa741eb4a91e6b2d55",
        "Целевое поведение": "a4da526a921141c53b4704c713899eb7f55fd535e30b31862f32da538c7e885a",
        "Требования": "cc7e41d179fe49fab8dda6b7f4b7e9a0ac5f1ce878d7f690d7d24efc0c5b9513",
        "Данные, права, ошибки": "f3c6f665a5658a0fb4a994f1886fe10e74df2fbcce5419442431846b02e3bf93",
        "Карта реализации": "921a0e22aa5226c8a6da269c5744632eb6656310e83a8adc95de4309897386a1",
        "Критерии приёмки": "aa8c527675cb8764e71c301c080809d8a95b8daa7d33adbfae49364605a0d398",
        "Влияние на руководство": "ca530aeffa226f8e56490ec60437005ee6c3f0f06e7b92e76ab85892bc1ed185"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "bec88e40988e36ea0358ef5c88281100422b1066b17535e2b59b263dc576338b"
        },
        "Цель": {
          "Цель": "e73157d7fb7dc9bba077a389c5d3c876f50dd9a9167cd6ce1f55cb156f4d15de"
        },
        "Границы": {
          "Границы": "f73e0ed13620512308521f5757d8fc3cb6b671562806c66937e7548a48d3b433"
        },
        "Целевое поведение": {
          "Целевое поведение": "6d39cb5e69af25459e99780d1ff148deb8002042cd24c8fd6046c92607059e75"
        },
        "Требования": {
          "FR-1": "d71c83edb2f0ffedb835b02bac530f2545c57aee33e4a3ec530722099ff467de",
          "FR-2": "c26c8f884297f33addfdbd6745aac42886ad590bdeab824afe087a051f38624a",
          "FR-3": "118f8f3723affffe42d49c2c02c53f422933b9493d65e84b9ecfe1e5df212d71",
          "FR-4": "7f22dc5ff357d4b74f52fe4678d130fdadc5347e9a7bcebf6de051cd4f14df04",
          "FR-5": "fa1028c9092af90d567556ce5b3a16f011640e4007c5105ae9e081c820600c6a",
          "FR-6": "4d6f397826611820f9aa24ac8e2ec8d857a22f372987f28c4e46252da78261b2",
          "FR-7": "d59ccd177324c29cc209066d59b25eb2a805b6636419680b4ccb5f2034c0ebdb",
          "FR-8": "3b47036fe6089724c88d2c705672a2390be12a4523f741d1baa1b4e684de4b03",
          "FR-9": "1f051e99829c161195392322fb325503a707e66c04edbc64ce177a386abc18d1",
          "FR-10": "4e210e6fcf22205cc1b93cd1b24fcefd6b250a47940fb647a40382ca0a86cd85"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "96cecd61c032d10818c3d0e5567e03821177b05db12a22357fa39d3e3b2f7d16"
        },
        "Карта реализации": {
          "Карта реализации": "5b4a93ddfc7323578fcf124778bc6c6924de5b4ffb8b351f244b846c18cc8b62"
        },
        "Критерии приёмки": {
          "AC-1": "33eef3f3fb0ded6d509e17258080923ec6cd155199c1cb7eee3ad10f3e03307a",
          "AC-2": "f49f52aa508dc79d0b4e13fdaf628b8be9b6249863d6f5f9e0ddd4411323f548",
          "AC-3": "b049a0d673af0ac71487509a52b50d69c89520b84f51b7ac9124deb5daced20f",
          "AC-4": "564e1f60808cce70c57aa98a2cd48a121c27876c4f9f290b44e0b711faf83de2",
          "AC-5": "9416abead1f18b5b39742a15f57c0fbcf89536725c33658c9eca40f83d173bed",
          "AC-6": "c26e79a548d038b13e949f983affbba8565766f02d2303e882bcce19dd911b81",
          "AC-7": "7ab888fb76a2e496ee55bd5e8dcec3efd3dad0356cc4e72291a56720c7038505",
          "AC-8": "85ee699f6ebb427d8e7f594544f2b92e39fabcf34630e797e251f5dd01ba8447",
          "AC-9": "e53a47581ee41ac01525fe468e14f2a547b0118759d2a3a86b106605aaa0b383",
          "AC-10": "877fa44e5eeba77517a18b9a9df3ed30920014cf97b33b9fe9d4d998d9ac16d9"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "75ad1b4efffabc7535761dfb2f02d091f1c51b81c5781efba1afab27be9eb0d4"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "d36b430fd294b600dbc6cbbd481c39535027778c96524de8ebe82de0c4399729",
    "backend_tests": "d36b430fd294b600dbc6cbbd481c39535027778c96524de8ebe82de0c4399729"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства в репозитории ещё нет; API ядра описывает doc-комментарий и `CONTRACT.md`."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "5f73b1e9b30b74756a9ed77dc99c01682b5757f25d5cdd7f00fe1b6fd9731d29",
      "files": [
        "Cargo.lock",
        "clippy.toml",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/highlight.css",
        "packages/theme/katex-init.js",
        "packages/theme/mermaid-init.js",
        "packages/theme/nimmd.css",
        "crates/nimmd-core/tests/document.rs",
        "crates/nimmd-core/tests/snapshots/document__math_mathml_mode.snap"
      ],
      "production_hash": "b3b293fe496e342fcc62e796cfdeb422a3c5c49d1f45f10d5cdd51a9972b7c3c",
      "production_files": [
        "Cargo.lock",
        "clippy.toml",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/highlight.css",
        "packages/theme/katex-init.js",
        "packages/theme/mermaid-init.js",
        "packages/theme/nimmd.css"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "5f73b1e9b30b74756a9ed77dc99c01682b5757f25d5cdd7f00fe1b6fd9731d29",
      "files": [
        "Cargo.lock",
        "clippy.toml",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/highlight.css",
        "packages/theme/katex-init.js",
        "packages/theme/mermaid-init.js",
        "packages/theme/nimmd.css",
        "crates/nimmd-core/tests/document.rs",
        "crates/nimmd-core/tests/snapshots/document__math_mathml_mode.snap"
      ],
      "production_hash": "b3b293fe496e342fcc62e796cfdeb422a3c5c49d1f45f10d5cdd51a9972b7c3c",
      "production_files": [
        "Cargo.lock",
        "clippy.toml",
        "crates/nimmd-core/Cargo.toml",
        "crates/nimmd-core/src/detect.rs",
        "crates/nimmd-core/src/document.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/render.rs",
        "crates/nimmd-core/src/text.rs",
        "packages/theme/CONTRACT.md",
        "packages/theme/highlight.css",
        "packages/theme/katex-init.js",
        "packages/theme/mermaid-init.js",
        "packages/theme/nimmd.css"
      ]
    }
  },
  "accepted_code_hash": {
    "backend-tests": "d34639d6dcbf44a5c9fe6f565370f844989ac1aea0608133e990cedcd1af15c8",
    "testing": "d34639d6dcbf44a5c9fe6f565370f844989ac1aea0608133e990cedcd1af15c8"
  }
}
```
<!-- afd-state-snapshot:end -->
