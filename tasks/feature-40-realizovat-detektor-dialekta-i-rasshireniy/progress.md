# Состояние разработки: Реализовать детектор диалекта и расширений

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #40 (syper-freedm/NimMD)
- Размер: M; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T12:00:13+00:00

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

- 2026-10-25T11:55:06+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": [], "notes": ["SEC-002 закрыт: detect вызывает text::limit_nesting перед parse_document; замер debug: 60k '>' 7 мс, 60k '- ' 7 мс, 1 МБ '>' 43 мс, 10 МБ строк по 200 '>' 0,83 с", "cargo deny не перезапущен: песочница запрещает lock ~/.cargo/advisory-dbs; результат круга 1 — ok; правка SEC-002 зависимости не трогает (text.rs, detect.rs)", "Лестница списков через отступ продолжения (ARCH-005) не перепроверялась — вынесена в #41"]}
- 2026-10-25T11:55:06+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": [], "notes": ["Покрытие: SC-001 plain_commonmark; SC-002 gfm_table_task_strike; SC-003 footnote_is_gfm_not_multimarkdown; SC-004 obsidian_unambiguous_signs; SC-005 obsidian_weak_signs; SC-006 multimarkdown_signs (+ multimarkdown_metadata_edges, heading_id_edges); SC-007 bare_autolink; SC-008 signs_inside_code_fences_are_ignored (+ fence_closing_rules, table_header_inside_fence_does_not_count); SC-009 issue_reference_and_heading_are_not_tags; SC-010 math_from_ast; SC-011 mermaid_from_ast; SC-012 frontmatter_overrides_dialect_only (+ frontmatter_content_is_not_scanned); SC-013 edge_inputs_are_total; SC-015 deep_nesting_is_fast (detect.rs) + limit_nesting_deep_prefix, limit_nesting_boundary (text.rs). SC-014 — static, тестом не автоматизируется.", "Дополнительно: callout_kind_is_validated, task_markers, table_needs_header_row_with_pipe, footnote_edges, dialect_priority, limit_nesting_mixed_markers_and_lines; рандомизированные с фиксированным seed: detect_is_total_on_random_input (тотальность, frontmatter == split_frontmatter, всё внутри незакрытого ~~~-блока даёт Profile::default()), limit_nesting_is_lossless_and_idempotent (без вставленных \\ — исходник, Borrowed <=> без изменений, не больше одного \\ на строку, повторное применение — Borrowed).", "Red-before-fix: задача feature (новый код), stash производственного кода дал бы только ошибку компиляции, поэтому проверено мутациями (бэкап в control/.tmp/backend-tests/*.rs.orig, восстановление сверено cmp и git diff --stat). M1 (строки внутри fenced-блока не пропускаются): cargo test -p nimmd-core --test detect -> FAILED signs_inside_code_fences_are_ignored, fence_closing_rules, detect_is_total_on_random_input; left: Profile { dialect: Gfm, ... tasklists: true, strikethrough: true } right: CommonMark/all false. M2 (callout || tag): FAILED obsidian_weak_signs, issue_reference_and_heading_are_not_tags, callout_kind_is_validated; left: Obsidian, right: CommonMark. M3 (предел 63 вместо 64): cargo test --test text -> FAILED limit_nesting_deep_prefix, limit_nesting_boundary, limit_nesting_mixed_markers_and_lines; left: 63 '>' right: 64 '>'. M4 (detect без limit_nesting): FAILED deep_nesting_is_fast, прогон 7.72s.", "Проверки: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings — 0 замечаний; cargo test --workspace --all-targets — 47 passed (detect 26, profile 7, text 14), 0 failed, 0 ignored; diff-lint.sh backend — exit 0 (clippy-changed clean, cargo-machete clean, semgrep-project-rules без находок; semgrep запускался с SEMGREP_LOG_FILE в tmp_dir — песочница запрещает запись в ~/.semgrep). cargo deny check не запускался (нужна сеть; тесты зависимостей не меняют).", "ARCH-005 (NOTICE) подтверждён измерением, но тест не добавлен: он расходится с ФД. FR-10 явно задаёт подсчёт маркеров в префиксе одной строки, а AC-14 не содержит пункта (г). Лестница из 200 строк (строка k = 32*k табов + 64 '- ' + 'x', около 660 КБ, 12 800 уровней) выполняется в detect за 8.3 с в debug и 1.11 с в release; на 10 МБ (около 800 строк) это примерно в 16 раз дольше. Риск DoS для Quick Look — для решения на уровне ФД (считать глубину продолжения)."]}
- 2026-10-25T11:55:06+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["Полный набор не перезапускался: control/suite-run.backend-tests.json result=green, tree 1e72fd9c6fbbf9cb6813c0e8ad65574297b5875f = текущему tree-hash.sh", "Фокусный прогон: cargo test -p nimmd-core --test detect --test text — detect 26 passed, text 14 passed, 0 ignored, 0 skipped", "SC-014: just clippy rc=0; semgrep --config .semgrep.yml по файлам правки — 0 findings (HOME перенаправлен в tmp_dir: песочница запрещает запись в ~/.semgrep; обращение к semgrep.dev заблокировано, на результат не влияет); git grep unwrap/expect/panic!/срезов по detect.rs и parse.rs — пусто", "SC-001…SC-015: все pass; changed-files.docs.txt пуст — проверка руководства не требуется"]}
- 2026-10-25T11:55:06+00:00 `note` — {"text": "QA-005: дефект тестов (нет фикстур) — rework backend-tests без перехода"}
- 2026-10-25T11:58:25+00:00 `handback_accepted` — {"phase": "backend-tests", "verdict": "DONE", "findings": ["QA-005"], "notes": ["QA-005: добавлен раннер fixtures_match_profiles в crates/nimmd-core/tests/detect.rs: обходит tests/fixtures/<группа>/*.md (std::fs, без новых крейтов; serde_json уже в dev-dependencies), читает парный <имя>.profile.json как Profile (deny_unknown_fields, все 9 полей Features обязательны), сверяет с nimmd_core::detect; в сообщении assert — путь фикстуры относительно fixtures/ (например features/callouts.md); отсутствующий или битый .profile.json — panic с именем фикстуры; assert checked > 0 против пустого каталога. Inline-тесты detect.rs не тронуты.", "Перед detect текст проходит text::normalize_newlines: checkout с autocrlf на Windows CI даёт \\r\\n, а предусловие detect — \\n.", "Фикстуры (12 пар) и ожидания по AC: commonmark/plain (AC-1, CommonMark, все false); commonmark/issue_ref (AC-6 без callout: #123/#42 не теги, CommonMark, все false); gfm/table_tasks (AC-2, Gfm, tables+tasklists+strikethrough); gfm/footnote_only (AC-3, Gfm, footnotes); obsidian/wikilinks ([[Note]] и ![[image.png]], Obsidian, wikilinks); obsidian/callout_in_code (callout, #tag, [[Note]], %%c%% внутри ```markdown-блока — AC-5, CommonMark, все false); multimarkdown/header_ids (AC-8б, MultiMarkdown, все false); features/math (AC-9а, CommonMark, math); features/not_math (AC-9б, CommonMark, все false); features/mermaid_unclosed (AC-10а, CommonMark, mermaid); features/frontmatter_override (AC-11 второй, Gfm, frontmatter+wikilinks); features/callouts (AC-7б, два callout без тега — CommonMark, callouts).", "Red-проверка раннера: (1) в features/callouts.profile.json временно callouts=false -> cargo test -p nimmd-core --test detect fixtures: FAILED, 'assertion `left == right` failed: features/callouts.md'; (2) каталог fixtures временно пуст -> FAILED 'в .../tests/fixtures нет фикстур'. Обе правки откачены (фикстура восстановлена из копии в tmp_dir, каталог возвращён на место), полный прогон после этого зелёный.", "Проверки: cargo fmt --all -- --check OK; cargo clippy --workspace --all-targets -- -D warnings — 0 замечаний; cargo test --workspace --all-targets — 48 passed (detect 27, profile 7, text 14), 0 failed, 0 ignored; suite-run записан после последнего прогона.", "Линтеры по .md-фикстурам: semgrep scan --config .semgrep.yml полный (как в just lint) и явно по tests/fixtures — rc=0 (запуск с SEMGREP_LOG_FILE в tmp_dir: песочница запрещает ~/.semgrep); в фикстурах нет trust/unsafe_/std:: (git grep пуст); gitleaks dir по fixtures и detect.rs — no leaks; diff-lint.sh backend — exit 0; фикстуры не попадают под .gitignore (git ls-files --others --exclude-standard видит все 24). markdownlint и подобных в just lint нет. cargo deny / vendor / npm-licenses / actionlint не запускались (сеть; тесты их не затрагивают).", "crates/nimmd-core/tests/text.rs не менялся в этом цикле (в списке с прошлого цикла), поэтому в changed_files не включён."]}
- 2026-10-25T11:58:25+00:00 `marked` — {"key": "implementation.backend_tests", "value": "done"}
- 2026-10-25T11:58:26+00:00 `note` — {"text": "backend-tests QA-005 принят; повторный testing"}
- 2026-10-25T11:59:50+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["QA-005: `cargo test -p nimmd-core --test detect fixtures_match_profiles` ok; весь detect 27 passed, 0 ignored", "12 пар фикстур сверены с AC-1…AC-11: ожидания .profile.json совпадают с Ожиданием AC", "Полный набор: suite-run.backend-tests.json green, tree 10de5d6 совпадает с текущим; не перезапускался", "`cargo clippy -p nimmd-core --all-targets -- -D warnings` ok, `cargo fmt --all --check` ok"]}
- 2026-10-25T11:59:50+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T11:59:50+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T11:59:50+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T11:59:50+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T11:59:50+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T11:59:50+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T12:00:13+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 40,
  "title": "Реализовать детектор диалекта и расширений",
  "slug": "realizovat-detektor-dialekta-i-rasshireniy",
  "branch": "feat/40-realizovat-detektor-dialekta-i",
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
      "at": "2026-10-25T11:07:00+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T11:08:43+00:00",
      "event": "classified",
      "size": "M",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T11:08:53+00:00",
      "event": "note",
      "text": "ФД написан; продуктовых развилок нет (расхождения спек/план решены правилом плана «побеждает спека»), afd-product-owner не вызывался"
    },
    {
      "at": "2026-10-25T11:08:53+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T11:08:53+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T11:08:53+00:00",
      "event": "note",
      "text": "диспатч FD-ревью (arch, sec, qa)"
    },
    {
      "at": "2026-10-25T11:11:25+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002"
      ],
      "notes": [
        "comrak 0.56.0 есть в индексе crates.io; API и фича syntect-fancy не сверялись — крейт не скачан в песочнице"
      ]
    },
    {
      "at": "2026-10-25T11:11:25+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001"
      ],
      "notes": [
        "Новых прав, данных классов, сетевого/файлового доступа и защищённых потоков нет — security.md не создавался.",
        "comrak 0.56 есть в [workspace.dependencies], BSD-2-Clause в белом списке deny.toml; split_frontmatter/frontmatter_dialect/MAX_BYTES существуют в text.rs."
      ]
    },
    {
      "at": "2026-10-25T11:11:25+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "Все 13 AC наблюдаемы, детерминированы и в границах; validate-tests: OK; test-cases.md 14 сценариев (13 integration в crates/nimmd-core/tests/detect.rs + 1 static), около 2676 из 2700 токенов.",
        "QA-001/QA-002 — NOTICE: однозначные признаки `%%…%%` и блок метаданных MMD нигде не проверены изолированно; AC-12 не называет ожидаемый диалект (SC-013 ждёт CommonMark).",
        "Сверено с кодом: Profile/Features/Dialect в profile.rs, split_frontmatter/frontmatter_dialect в text.rs (незакрытый frontmatter -> (None, src)), comrak в [workspace.dependencies], BSD-2-Clause в deny.toml.",
        "Не проверено: comrak не в Cargo.lock и не в кэше registry, поэтому правило math_dollars для `цена $5 и $10` (AC-9б) сверено только по документированному pandoc-правилу (закрывающий `$` не после пробела)."
      ]
    },
    {
      "at": "2026-10-25T11:11:25+00:00",
      "event": "note",
      "text": "FD-ревью приняты, все APPROVED; NOTICE ARCH-001/002, SEC-001, QA-001/002"
    },
    {
      "at": "2026-10-25T11:11:44+00:00",
      "event": "note",
      "text": "confirm-круг FD-ревью после правки ФД по NOTICE"
    },
    {
      "at": "2026-10-25T11:13:01+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-002",
        "ARCH-003"
      ],
      "notes": [
        "Режим дельты: проверены только FR-1, FR-9, AC-4, AC-8, AC-12; правки согласованы с FR-4/FR-5 и закрывают ARCH-001/ARCH-002. ARCH-003 — новый NOTICE: стойкость comrak к глубокой вложенности не проверена."
      ]
    },
    {
      "at": "2026-10-25T11:13:01+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001"
      ],
      "notes": [
        "Режим дельты: FR-1, FR-9, AC-4, AC-8, AC-12 проверены; правка FR-9/AC-12 закрывает SEC-001 (итеративный обход AST, тест на 10 000 уровней цитаты); новых находок нет, reviews.md не менялся."
      ]
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "Дельта (FR-1, FR-9, AC-4, AC-8, AC-12) закрывает QA-001 и QA-002; новые случаи AC-4 (только `%%c%%`), AC-8(д) и AC-12 (вложенность `>` x 10 000) согласуются с FR-4/FR-5, наблюдаемы и детерминированы.",
        "test-cases.md: SC-004, SC-006 и SC-013 дополнены новыми случаями; для экономии объёма SC-001 и SC-014 сокращены; 2693 из 2700 токенов; validate-tests: OK.",
        "FR-1 (предусловие `\\n` без BOM) и FR-9 (итеративный обход через `descendants()`) — правки для ARCH/SEC; новых AC не требуют, на FR-9 указывает случай с глубокой вложенностью в SC-013."
      ]
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T11:13:02+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T11:13:03+00:00",
      "event": "note",
      "text": "ФД одобрен (ARCH-003 noted — проверяется AC-12); диспатч backend"
    },
    {
      "at": "2026-10-25T11:23:00+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: crates/nimmd-core/Cargo.toml, Cargo.lock, crates/nimmd-core/src/{parse,lib,detect}.rs, deny.toml",
        "self-checks green: cargo fmt --check, cargo build/clippy --workspace --all-targets -D warnings, cargo test --workspace (17 existing tests ok), cargo deny check (advisories/bans/licenses/sources ok), just lint (semgrep, machete, gitleaks, actionlint, zizmor ok), diff-lint backend exit 0, SC-014 grep empty",
        "все AC-1..AC-13 прогнаны во временном тесте (удалён, тесты пишет afd-test-developer): результаты совпали с ожиданиями",
        "ARCH-003: comrak 0.56 на 10 000 уровней '>' и 10 000 вложенных списков без переполнения стека даже на потоке 512 КБ; ограничение глубины не потребовалось",
        "out-of-scope: разбор comrak глубоко вложенных цитат квадратичен по глубине (release: 10k '>' ~36 мс, 30k ~305 мс; 1 МБ '>' — минуты) — угроза зависания Quick Look, касается и рендера #41; предлагаю ограничить глубину '>' до разбора отдельной задачей",
        "deny.toml: comrak с syntect-fancy (из workspace) принёс advisories; quick-xml/time/yaml-rust (через plist) тянутся только в ветке iOS/wasm comrak — добавлен [graph] targets на 3 ОС продукта; bincode 1.3.3 (RUSTSEC-2025-0141, unmaintained, не уязвимость, безопасной версии нет) — ignore с причиной; нужно подтверждение ревьюером ИБ",
        "cargo fetch и cargo deny запускались вне песочницы: песочница не даёт писать в ~/.cargo (registry, advisory-db lock)",
        "ponytail: Meta enum -> Option<bool> — rejected: именованное состояние читается яснее",
        "ponytail: 0/1 accepted"
      ]
    },
    {
      "at": "2026-10-25T11:23:00+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T11:23:00+00:00",
      "event": "note",
      "text": "backend принят; квадратичная глубина '>' в comrak — уже в объёме #41 (ограничение глубины вложенности, спек 5.2), follow-up не нужен"
    },
    {
      "at": "2026-10-25T11:23:00+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T11:23:00+00:00",
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
      "at": "2026-10-25T11:23:05+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T11:23:06+00:00",
      "event": "note",
      "text": "диспатч post-checks (ревью → тесты → QA)"
    },
    {
      "at": "2026-10-25T11:28:32+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-004"
      ],
      "notes": [
        "cargo clippy --workspace --all-targets -- -D warnings: Finished, без предупреждений",
        "Smoke-прогон detect (временный крейт в tmp_dir, удалён): ожидания AC-9..AC-13 совпали; 10 000 уровней > не роняют поток со стеком 512 КБ (debug)",
        "crates/nimmd-core/tests/detect.rs и фикстуры из карты пока отсутствуют — это зона этапа тестов, не проверялось"
      ]
    },
    {
      "at": "2026-10-25T11:28:32+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "BLOCKER",
      "findings": [
        "SEC-002"
      ],
      "notes": [
        "SEC-002 BLOCKER (OWASP API4): detect() разбирает недоверенный вход comrak 0.56, разбор квадратичен по глубине вложенности '>'. Замер (release, отдельный крейт в tmp_dir): 10k '>' 37 мс, 30k 359 мс, 60k 1,30 с; экстраполяция: 1 МБ — минуты, MAX_BYTES 10 МБ — часы. ФД («Данные, права, ошибки») требует линейной сложности, а Quick Look на таком файле зависнет. Ожидается: предел глубины вложенности контейнеров до parse_document, тест на 60k '>' с ограничением по времени",
        "cargo deny check (вне песочницы: lock в ~/.cargo/advisory-dbs только для чтения): advisories ok, bans ok, licenses ok, sources ok",
        "deny.toml [graph] targets проверен: без него cargo deny находит RUSTSEC-2026-0194/0195 (quick-xml), RUSTSEC-2026-0009 (time), RUSTSEC-2024-0320 (yaml-rust), все через plist/yaml-load из syntect; cargo tree -i plist на aarch64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc пуст, путь появляется только на iOS. Сужение по targets корректно; если добавят iOS/wasm-цель, эти advisories снова станут актуальны",
        "ignore RUSTSEC-2025-0141 (bincode unmaintained, не уязвимость) принят: syntect декодирует bincode только из встроенных дампов, документ туда не попадает",
        "semgrep (.semgrep.yml, p/rust, p/typescript, p/javascript, p/secrets) по parse.rs, lib.rs, detect.rs: 0 находок; gitleaks dir по 6 файлам из changed-files.txt: no leaks found",
        "Ручная проверка: в parse.rs render.unsafe_ = false; обход AST итеративный (descendants), SEC-001 закрыт; построчные проверки в detect.rs линейные (has_footnote обрывает id на '['); нового ввода-вывода, unsafe и паник в рабочем коде нет",
        "Вне границы: syntect (через syntect-fancy в workspace) тянет walkdir/std::fs, правило nimmd-core-no-io покрывает только собственный код ядра. Изменение уже было, не блокирует"
      ]
    },
    {
      "at": "2026-10-25T11:28:32+00:00",
      "event": "note",
      "text": "SEC-002: принято автором ФД без скептика — разработчик и ИБ независимо замерили квадратичный разбор comrak по глубине '>'; в ФД нет предела вложенности (спек 5.2 его требует)"
    },
    {
      "at": "2026-10-25T11:28:51+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "rework_required",
      "reason": "SEC-002: пробел ФД — нет предела вложенности"
    },
    {
      "at": "2026-10-25T11:28:52+00:00",
      "event": "transition",
      "from": "rework_required",
      "to": "requirements_rework",
      "reason": "SEC-002"
    },
    {
      "at": "2026-10-25T11:29:08+00:00",
      "event": "transition",
      "from": "requirements_rework",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T11:29:15+00:00",
      "event": "note",
      "text": "FD-круг после SEC-002: sec rework, arch+qa confirm"
    },
    {
      "at": "2026-10-25T11:32:09+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-005"
      ],
      "notes": [
        "Режим дельты: проверены только FR-9, FR-10, AC-14, «Данные, права, ошибки», «Карта реализации». FR-10 в parse.rs рядом с options() — без нового слоя; deny.toml внесён в карту (ARCH-004 учтён). ARCH-002/ARCH-003 свёрнуты как закрытые. ARCH-005 — новый NOTICE: предел FR-10 построчный, вложенность списков через отступ продолжения его обходит; замер comrak на такой лестнице не делался."
      ]
    },
    {
      "at": "2026-10-25T11:32:09+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "SEC-002 закрыт в ФД: FR-10 ограничивает префикс маркеров контейнеров (`>` и маркеры списка) до MAX_NESTING = 64 до разбора comrak, без потери символов; AC-14 проверяет 60 000 `>` и `- ` быстрее 1 с и обратимость вставленного `\\`.",
        "Сверено с comrak 0.56: списки и сноски уже ограничены MAX_LIST_DEPTH = 100, у blockquote предела нет. Каждый уровень цитаты требует свой `>` в префиксе строки, поэтому FR-10 ограничивает глубину и при смешанной вложенности списков и цитат по отступам.",
        "Реализация FR-10 в коде ещё не проверена: SEC-002 в разделе «Безопасность кода» остаётся до повторного security-code-review.",
        "reviews.md: 1797 из 1800 оценочных токенов, запаса почти нет."
      ]
    },
    {
      "at": "2026-10-25T11:32:10+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "BLOCKER",
      "findings": [
        "QA-003",
        "QA-004"
      ],
      "notes": [
        "QA-003 (BLOCKER): AC-14(в) противоречит FR-6. В FR-6 маркер задачи стоит «после пробелов», поэтому строка `> > > - [ ] a` (она начинается с `>`) задачей не считается: `is_task` в crates/nimmd-core/src/detect.rs делает только `trim_start()` и не снимает `>`. Значит `detect` даст `CommonMark` и все features false, а AC ждёт `Gfm` и `tasklists` true. Два выхода: в FR-6 пропускать префикс контейнеров FR-10 или переписать ожидание (в).",
        "QA-004 (NOTICE): функция FR-10 объявлена `pub(crate)`, поэтому из `tests/detect.rs` её не вызвать. Для AC-14 «64 маркера до `\\`», обратимость и `Cow::Borrowed` проверит только unit-тест `#[cfg(test)]` в parse.rs. В карте реализации сейчас указан только tests/detect.rs.",
        "FR-9, FR-10 и «Данные, права, ошибки»: правки наблюдаемы, с остальными пунктами не конфликтуют. Пункты AC-14(а) и (б) проверяемы и детерминированы: при пределе 64 разбор линеен, так что порог 1 с идёт с большим запасом.",
        "test-cases.md: добавлен SC-015 (AC-14, unit). Чтобы уложиться в лимит, сокращены формулировки SC-001, SC-002, SC-008, SC-010, SC-013, SC-014 и метаданных. Объём 2699 из 2700, validate-tests: OK. В reviews.md закрытые QA-001 и QA-002 свёрнуты в одну строку, объём 1434 из 1800."
      ]
    },
    {
      "at": "2026-10-25T11:32:10+00:00",
      "event": "note",
      "text": "QA-003: принято автором ФД без скептика — AC-14(в) противоречит FR-6 (задача только после пробелов); QA-004 учтён переносом функции FR-10 в pub text.rs; ARCH-005 noted: списки ограничены MAX_LIST_DEPTH=100 comrak (сверено SEC)"
    },
    {
      "at": "2026-10-25T11:32:21+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "requirements_rework",
      "reason": "QA-003"
    },
    {
      "at": "2026-10-25T11:32:21+00:00",
      "event": "transition",
      "from": "requirements_rework",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T11:32:22+00:00",
      "event": "note",
      "text": "FD-круг 3: qa rework QA-003, sec+arch delta"
    },
    {
      "at": "2026-10-25T11:34:06+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001",
        "ARCH-005"
      ],
      "notes": [
        "Дельта: limit_nesting/MAX_NESTING перенесены в pub-модуль text.rs (предобработка без comrak, правила тотальности text.rs выполнимы через split_at_checked/обход char_indices); AC-14(в) теперь CommonMark — согласовано с FR-6. Новых находок нет."
      ]
    },
    {
      "at": "2026-10-25T11:34:06+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [
        "SEC-001"
      ],
      "notes": [
        "Дельта FR-10/AC-14: limit_nesting перенесена в text.rs как pub (чистая функция без I/O под правилами тотальности text.rs), ограничение 64 маркера и тест 60k (а)/(б) сохранены; AC-14(в) — смена ожиданий детектора, вне области ИБ. reviews.md не менялся."
      ]
    },
    {
      "at": "2026-10-25T11:34:06+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "QA-003 закрыт: AC-14(в) теперь `> ` x3 + `текст` с ожиданием `CommonMark`, все features false, `Cow::Borrowed`. Это согласуется с FR-6: строка без маркера задачи, признаков FR-4/5/6 нет.",
        "QA-004 подтверждён закрытым: FR-10 объявляет `pub fn limit_nesting` и `pub const MAX_NESTING` в `text.rs`, а lib.rs содержит `pub mod text`. Поэтому AC-14 проверяется интеграционным тестом.",
        "test-cases.md: SC-015 переписан под новую AC-14 (integration, `text::limit_nesting`, (в) — `Profile::default()`). Перед правкой файл весил 2772 из 2700 и был уже сверх лимита, поэтому формулировки SC-001/002/004/005/007/013/014 и метаданных сокращены. Теперь 2698 из 2700, validate-tests: OK. reviews.md — 1473 из 1800."
      ]
    },
    {
      "at": "2026-10-25T11:34:06+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:34:06+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:34:06+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:34:07+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:34:07+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:34:07+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T11:34:07+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T11:34:07+00:00",
      "event": "note",
      "text": "ФД одобрен (круг 3); backend rework SEC-002 (FR-10 limit_nesting)"
    },
    {
      "at": "2026-10-25T11:40:41+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [
        "SEC-002"
      ],
      "notes": [
        "ARCH-005 (NOTICE) не реализован: он расходится с FR-10, где предел считается по маркерам одной строки. Лестница подтверждена замером в debug: строка k = k табов + 64 «- » + x; 200 строк (46 КБ) — 0,35 с, 400 строк (132 КБ) — 1,18 с, 800 строк (424 КБ, ~51 000 уровней) — 4,5 с. Время растёт квадратично от глубины, 10 МБ — минуты. Чтобы закрыть, нужна правка ФД: ограничить и глубину продолжения, например колонку отступа (≤ 2·MAX_NESTING) перед `\\`",
        "SEC-002: в text.rs добавлены MAX_NESTING = 64 и limit_nesting (FR-10): без индексации и срезов, только strip_prefix/get; detect.rs разбирает comrak текст после limit_nesting(src). Сканер признаков работает по исходному тексту",
        "AC-14 проверен временным тестом (удалён): (а) 60 000 `>` + x — detect 7,2 мс; (б) 60 000 `- ` + x — 7,1 мс; в обоих до `\\` ровно 64 маркера, а удаление `\\` даёт исходник байт-в-байт; (в) `> > > текст` — Cow::Borrowed, Profile::default()",
        "Маркер `N.`/`N)`: перед цифрой `\\` не экранирует, поэтому в рендере сверх 64 уровней `\\` останется виден как текст. Так прописано в FR-10 (`\\` перед маркером), текст не теряется",
        "changed-files.backend.txt: crates/nimmd-core/Cargo.toml, Cargo.lock, crates/nimmd-core/src/parse.rs, crates/nimmd-core/src/lib.rs, crates/nimmd-core/src/detect.rs, deny.toml, crates/nimmd-core/src/text.rs",
        "self-checks: cargo fmt --check ok; build ok; clippy -D warnings ok; cargo test --workspace --all-targets — 17 passed, 0 failed; cargo deny check — advisories/bans/licenses/sources ok (4 предупреждения license-exception-not-encountered; этот проход deny.toml и зависимости не трогал); diff-lint backend exit 0; just lint exit 0; semgrep по text.rs/detect.rs — 0",
        "migrations: нет; isolation: не объявлена",
        "ponytail: лестница через отступ продолжения остаётся квадратичной — rejected: противоречит per-line правилу FR-10, нужна правка ФД (см. ARCH-005 выше)",
        "ponytail: `\\` перед `N.` виден в рендере — rejected: место вставки прописано в FR-10",
        "ponytail: 0/2 accepted"
      ]
    },
    {
      "at": "2026-10-25T11:40:41+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T11:40:41+00:00",
      "event": "note",
      "text": "ARCH-005: вне объёма → #41 (глубина списков через отступ продолжения)"
    },
    {
      "at": "2026-10-25T11:40:42+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T11:40:42+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 7,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "documentation"
      ],
      "delta": {
        "code-review": [
          "crates/nimmd-core/src/detect.rs",
          "crates/nimmd-core/src/text.rs"
        ]
      }
    },
    {
      "at": "2026-10-25T11:40:47+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T11:40:48+00:00",
      "event": "note",
      "text": "post-checks круг 2: code-review delta, sec rework SEC-002"
    },
    {
      "at": "2026-10-25T11:55:06+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-004"
      ],
      "notes": [
        "Дельта: limit_nesting (FR-10) в text.rs и её вызов в detect.rs перед comrak; новых находок нет. cargo clippy -p nimmd-core -D warnings и cargo test -p nimmd-core — ok."
      ]
    },
    {
      "at": "2026-10-25T11:55:06+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "SEC-002 закрыт: detect вызывает text::limit_nesting перед parse_document; замер debug: 60k '>' 7 мс, 60k '- ' 7 мс, 1 МБ '>' 43 мс, 10 МБ строк по 200 '>' 0,83 с",
        "cargo deny не перезапущен: песочница запрещает lock ~/.cargo/advisory-dbs; результат круга 1 — ok; правка SEC-002 зависимости не трогает (text.rs, detect.rs)",
        "Лестница списков через отступ продолжения (ARCH-005) не перепроверялась — вынесена в #41"
      ]
    },
    {
      "at": "2026-10-25T11:55:06+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "Покрытие: SC-001 plain_commonmark; SC-002 gfm_table_task_strike; SC-003 footnote_is_gfm_not_multimarkdown; SC-004 obsidian_unambiguous_signs; SC-005 obsidian_weak_signs; SC-006 multimarkdown_signs (+ multimarkdown_metadata_edges, heading_id_edges); SC-007 bare_autolink; SC-008 signs_inside_code_fences_are_ignored (+ fence_closing_rules, table_header_inside_fence_does_not_count); SC-009 issue_reference_and_heading_are_not_tags; SC-010 math_from_ast; SC-011 mermaid_from_ast; SC-012 frontmatter_overrides_dialect_only (+ frontmatter_content_is_not_scanned); SC-013 edge_inputs_are_total; SC-015 deep_nesting_is_fast (detect.rs) + limit_nesting_deep_prefix, limit_nesting_boundary (text.rs). SC-014 — static, тестом не автоматизируется.",
        "Дополнительно: callout_kind_is_validated, task_markers, table_needs_header_row_with_pipe, footnote_edges, dialect_priority, limit_nesting_mixed_markers_and_lines; рандомизированные с фиксированным seed: detect_is_total_on_random_input (тотальность, frontmatter == split_frontmatter, всё внутри незакрытого ~~~-блока даёт Profile::default()), limit_nesting_is_lossless_and_idempotent (без вставленных \\ — исходник, Borrowed <=> без изменений, не больше одного \\ на строку, повторное применение — Borrowed).",
        "Red-before-fix: задача feature (новый код), stash производственного кода дал бы только ошибку компиляции, поэтому проверено мутациями (бэкап в control/.tmp/backend-tests/*.rs.orig, восстановление сверено cmp и git diff --stat). M1 (строки внутри fenced-блока не пропускаются): cargo test -p nimmd-core --test detect -> FAILED signs_inside_code_fences_are_ignored, fence_closing_rules, detect_is_total_on_random_input; left: Profile { dialect: Gfm, ... tasklists: true, strikethrough: true } right: CommonMark/all false. M2 (callout || tag): FAILED obsidian_weak_signs, issue_reference_and_heading_are_not_tags, callout_kind_is_validated; left: Obsidian, right: CommonMark. M3 (предел 63 вместо 64): cargo test --test text -> FAILED limit_nesting_deep_prefix, limit_nesting_boundary, limit_nesting_mixed_markers_and_lines; left: 63 '>' right: 64 '>'. M4 (detect без limit_nesting): FAILED deep_nesting_is_fast, прогон 7.72s.",
        "Проверки: cargo fmt --all -- --check OK; cargo build --workspace --all-targets OK; cargo clippy --workspace --all-targets -- -D warnings — 0 замечаний; cargo test --workspace --all-targets — 47 passed (detect 26, profile 7, text 14), 0 failed, 0 ignored; diff-lint.sh backend — exit 0 (clippy-changed clean, cargo-machete clean, semgrep-project-rules без находок; semgrep запускался с SEMGREP_LOG_FILE в tmp_dir — песочница запрещает запись в ~/.semgrep). cargo deny check не запускался (нужна сеть; тесты зависимостей не меняют).",
        "ARCH-005 (NOTICE) подтверждён измерением, но тест не добавлен: он расходится с ФД. FR-10 явно задаёт подсчёт маркеров в префиксе одной строки, а AC-14 не содержит пункта (г). Лестница из 200 строк (строка k = 32*k табов + 64 '- ' + 'x', около 660 КБ, 12 800 уровней) выполняется в detect за 8.3 с в debug и 1.11 с в release; на 10 МБ (около 800 строк) это примерно в 16 раз дольше. Риск DoS для Quick Look — для решения на уровне ФД (считать глубину продолжения)."
      ]
    },
    {
      "at": "2026-10-25T11:55:06+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Полный набор не перезапускался: control/suite-run.backend-tests.json result=green, tree 1e72fd9c6fbbf9cb6813c0e8ad65574297b5875f = текущему tree-hash.sh",
        "Фокусный прогон: cargo test -p nimmd-core --test detect --test text — detect 26 passed, text 14 passed, 0 ignored, 0 skipped",
        "SC-014: just clippy rc=0; semgrep --config .semgrep.yml по файлам правки — 0 findings (HOME перенаправлен в tmp_dir: песочница запрещает запись в ~/.semgrep; обращение к semgrep.dev заблокировано, на результат не влияет); git grep unwrap/expect/panic!/срезов по detect.rs и parse.rs — пусто",
        "SC-001…SC-015: все pass; changed-files.docs.txt пуст — проверка руководства не требуется"
      ]
    },
    {
      "at": "2026-10-25T11:55:06+00:00",
      "event": "note",
      "text": "QA-005: дефект тестов (нет фикстур) — rework backend-tests без перехода"
    },
    {
      "at": "2026-10-25T11:58:25+00:00",
      "event": "handback_accepted",
      "phase": "backend-tests",
      "verdict": "DONE",
      "findings": [
        "QA-005"
      ],
      "notes": [
        "QA-005: добавлен раннер fixtures_match_profiles в crates/nimmd-core/tests/detect.rs: обходит tests/fixtures/<группа>/*.md (std::fs, без новых крейтов; serde_json уже в dev-dependencies), читает парный <имя>.profile.json как Profile (deny_unknown_fields, все 9 полей Features обязательны), сверяет с nimmd_core::detect; в сообщении assert — путь фикстуры относительно fixtures/ (например features/callouts.md); отсутствующий или битый .profile.json — panic с именем фикстуры; assert checked > 0 против пустого каталога. Inline-тесты detect.rs не тронуты.",
        "Перед detect текст проходит text::normalize_newlines: checkout с autocrlf на Windows CI даёт \\r\\n, а предусловие detect — \\n.",
        "Фикстуры (12 пар) и ожидания по AC: commonmark/plain (AC-1, CommonMark, все false); commonmark/issue_ref (AC-6 без callout: #123/#42 не теги, CommonMark, все false); gfm/table_tasks (AC-2, Gfm, tables+tasklists+strikethrough); gfm/footnote_only (AC-3, Gfm, footnotes); obsidian/wikilinks ([[Note]] и ![[image.png]], Obsidian, wikilinks); obsidian/callout_in_code (callout, #tag, [[Note]], %%c%% внутри ```markdown-блока — AC-5, CommonMark, все false); multimarkdown/header_ids (AC-8б, MultiMarkdown, все false); features/math (AC-9а, CommonMark, math); features/not_math (AC-9б, CommonMark, все false); features/mermaid_unclosed (AC-10а, CommonMark, mermaid); features/frontmatter_override (AC-11 второй, Gfm, frontmatter+wikilinks); features/callouts (AC-7б, два callout без тега — CommonMark, callouts).",
        "Red-проверка раннера: (1) в features/callouts.profile.json временно callouts=false -> cargo test -p nimmd-core --test detect fixtures: FAILED, 'assertion `left == right` failed: features/callouts.md'; (2) каталог fixtures временно пуст -> FAILED 'в .../tests/fixtures нет фикстур'. Обе правки откачены (фикстура восстановлена из копии в tmp_dir, каталог возвращён на место), полный прогон после этого зелёный.",
        "Проверки: cargo fmt --all -- --check OK; cargo clippy --workspace --all-targets -- -D warnings — 0 замечаний; cargo test --workspace --all-targets — 48 passed (detect 27, profile 7, text 14), 0 failed, 0 ignored; suite-run записан после последнего прогона.",
        "Линтеры по .md-фикстурам: semgrep scan --config .semgrep.yml полный (как в just lint) и явно по tests/fixtures — rc=0 (запуск с SEMGREP_LOG_FILE в tmp_dir: песочница запрещает ~/.semgrep); в фикстурах нет trust/unsafe_/std:: (git grep пуст); gitleaks dir по fixtures и detect.rs — no leaks; diff-lint.sh backend — exit 0; фикстуры не попадают под .gitignore (git ls-files --others --exclude-standard видит все 24). markdownlint и подобных в just lint нет. cargo deny / vendor / npm-licenses / actionlint не запускались (сеть; тесты их не затрагивают).",
        "crates/nimmd-core/tests/text.rs не менялся в этом цикле (в списке с прошлого цикла), поэтому в changed_files не включён."
      ]
    },
    {
      "at": "2026-10-25T11:58:25+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "done"
    },
    {
      "at": "2026-10-25T11:58:26+00:00",
      "event": "note",
      "text": "backend-tests QA-005 принят; повторный testing"
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "QA-005: `cargo test -p nimmd-core --test detect fixtures_match_profiles` ok; весь detect 27 passed, 0 ignored",
        "12 пар фикстур сверены с AC-1…AC-11: ожидания .profile.json совпадают с Ожиданием AC",
        "Полный набор: suite-run.backend-tests.json green, tree 10de5d6 совпадает с текущим; не перезапускался",
        "`cargo clippy -p nimmd-core --all-targets -- -D warnings` ok, `cargo fmt --all --check` ok"
      ]
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T11:59:50+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T12:00:13+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "8a4f711c52933008c0e9b799255910a577d966a0761f58cd626a964bf8a836b4",
      "sections": {
        "Метаданные": "986e5304347ab9b02587c97bd2486177240d883729084dc802f2b7251351f8dd",
        "Цель": "082d665055ac2180475cefe9d7e870a16102aaf05377d120815d29124519e1cb",
        "Границы": "1baf85b66f1ec70aae4af4744cafa23bab69b706725db75a8ddfad44193aa1a8",
        "Целевое поведение": "55160e2dcf78f2a3f06f9b75c09696e12651feace1c0740da581a821c09da189",
        "Требования": "f4ba5cabd2e4550cce10ee8f3abcf03abe8b6451a03dc82d0d71eb4b121d2f24",
        "Данные, права, ошибки": "c59ec2da1cfb5cfbdbb5616d4823625f028507d39c4b84dcb77293ab78fcf8b9",
        "Карта реализации": "d515f713d79560c8c258ab279762f3cf32a0fc18615dcc5681cb30b4cb5966ef",
        "Критерии приёмки": "867bbc8c4ce3e361d6ff1f697fcae6223e69ea90e808ad277c856191c80f4066",
        "Влияние на руководство": "9a639c0515e6c7ad2f825d3b98f3ba5b6b6394acc2033cbdef74906362ea3654"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "6b0fc0477e82d6bb92ca043a6e6ba1df791d2e60154206be406818695df7c708"
        },
        "Цель": {
          "Цель": "17ae0f3461a29bc30ce60db7eb1113f7b9d3335204756a033f0bb914d85f360f"
        },
        "Границы": {
          "Границы": "6314ac006189c83b06a0165525e05cd91b99f6cd7faea6d7735757117c7cc969"
        },
        "Целевое поведение": {
          "Целевое поведение": "995bb7438f942a9ab8e5d004cfcbb52aa60f94cfcefe9840b6b1cf8f9e104738"
        },
        "Требования": {
          "FR-1": "817a0fecc260370629ad28a39d5f1b7361bd9fd9d0325bff149673749139d93d",
          "FR-2": "baac62548965771ef2bb7373cb79b1065fd7b451c4449f64dc7372ed904cf5b3",
          "FR-3": "00ae7a31521c2a2b0fa3d1f53cfbcd5d3c67171e293a562c32f207336a217005",
          "FR-4": "401fe28a9db505066b2b3c0044714855608387ec935c05e6268ca09ff6910cad",
          "FR-5": "f221dfba8d8ab58e5ded5e7113c5665374ad9cdc0cb1651a445ad534bb518706",
          "FR-6": "4a67a56d0b7dfc57a6653f55ca640460caca598aa14c23f3d7d12ef7328bac74",
          "FR-7": "d77b4a1abfe2e957a15e40a27f56164617171d8f9fb612ff03f959741007083e",
          "FR-8": "730bcf67eb1eeb2f29249d5b176b53462d359037923545a49ee75c128207eff8",
          "FR-9": "fb66d95b1d42292d0df0968b9b54d70032144c0f228eaf583e5bd3bbab7037bf",
          "FR-10": "9fa5373f29cbdf9b533378ab9902d1fc2b53a559f7df285145ab9a25e93dd57f"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "5f8901def9925c6d8f714b63527aee1d9ace1431a37f0c86d33dc3992f0e4a7c"
        },
        "Карта реализации": {
          "Карта реализации": "13cf6196b1a31117ec9f0281697431ea29a6575f3c993a795227a65887cccbb3"
        },
        "Критерии приёмки": {
          "AC-1": "22d8a418ebfe78bbb6c5b9af237db9ad5094c65f409c6060bdff31d796765fe3",
          "AC-2": "0c79fc2cd86a8d94c52b24ad8f7d0209554bf8f42a50cadd49a583b725ee322b",
          "AC-3": "79dd36b4995fdc81df59a1482cdb652593bfa2f2fcf78d56fd8bbb681ca733ce",
          "AC-4": "248727507251650544f700e2e3c8e4e4d9b5c5943e7ff1a7532b10b040705f10",
          "AC-5": "17d54aae472c28178ef61340c1d2502b6f08cc1bcc5e5e294de60f8c204bfcd0",
          "AC-6": "453e01f1e9f77909bd6458ca9805266000c8991fdddc856636c8535abd60fb47",
          "AC-7": "4dd9f2237f2350745e45a5586cc4ad6803364254e024e0691bc3cadc64ae2737",
          "AC-8": "04feb8a4136f9848a0173651d6eef1ba120c91b476685dbf9e2f0dbe3f90c330",
          "AC-9": "bcd9f3d675b1853c865a8e61f996d648cfc7291ba2a8c89eca290ba9677e3908",
          "AC-10": "3ca87e0dcef5e12ad4f7b36f6b5da34b26bf8158eaebd7eb713c33b0fe8d4d66",
          "AC-11": "bd3e98ffff8dcb7119173f1fc25fd6c0e49753133d676ab9b52b56ead33e2262",
          "AC-12": "62173b6abadcbd42fe88a01789fd2cfcd5d11ec856966a7dca87b905429d8197",
          "AC-13": "ea0cd30cd4fd8a77e30156bb910c399744b8c4e3df3ee9feb01acf6eb0523f75",
          "AC-14": "2445686cb7d3249c6ca516d2d6fd3cc1b3ebd04c60d8bc567ea7e2641e4c5cc2"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "3b736d29e1827b33dacb14dc68ad7f8fcec3170a65525f059472f7032883bd83"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "8a4f711c52933008c0e9b799255910a577d966a0761f58cd626a964bf8a836b4",
      "sections": {
        "Метаданные": "986e5304347ab9b02587c97bd2486177240d883729084dc802f2b7251351f8dd",
        "Цель": "082d665055ac2180475cefe9d7e870a16102aaf05377d120815d29124519e1cb",
        "Границы": "1baf85b66f1ec70aae4af4744cafa23bab69b706725db75a8ddfad44193aa1a8",
        "Целевое поведение": "55160e2dcf78f2a3f06f9b75c09696e12651feace1c0740da581a821c09da189",
        "Требования": "f4ba5cabd2e4550cce10ee8f3abcf03abe8b6451a03dc82d0d71eb4b121d2f24",
        "Данные, права, ошибки": "c59ec2da1cfb5cfbdbb5616d4823625f028507d39c4b84dcb77293ab78fcf8b9",
        "Карта реализации": "d515f713d79560c8c258ab279762f3cf32a0fc18615dcc5681cb30b4cb5966ef",
        "Критерии приёмки": "867bbc8c4ce3e361d6ff1f697fcae6223e69ea90e808ad277c856191c80f4066",
        "Влияние на руководство": "9a639c0515e6c7ad2f825d3b98f3ba5b6b6394acc2033cbdef74906362ea3654"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "6b0fc0477e82d6bb92ca043a6e6ba1df791d2e60154206be406818695df7c708"
        },
        "Цель": {
          "Цель": "17ae0f3461a29bc30ce60db7eb1113f7b9d3335204756a033f0bb914d85f360f"
        },
        "Границы": {
          "Границы": "6314ac006189c83b06a0165525e05cd91b99f6cd7faea6d7735757117c7cc969"
        },
        "Целевое поведение": {
          "Целевое поведение": "995bb7438f942a9ab8e5d004cfcbb52aa60f94cfcefe9840b6b1cf8f9e104738"
        },
        "Требования": {
          "FR-1": "817a0fecc260370629ad28a39d5f1b7361bd9fd9d0325bff149673749139d93d",
          "FR-2": "baac62548965771ef2bb7373cb79b1065fd7b451c4449f64dc7372ed904cf5b3",
          "FR-3": "00ae7a31521c2a2b0fa3d1f53cfbcd5d3c67171e293a562c32f207336a217005",
          "FR-4": "401fe28a9db505066b2b3c0044714855608387ec935c05e6268ca09ff6910cad",
          "FR-5": "f221dfba8d8ab58e5ded5e7113c5665374ad9cdc0cb1651a445ad534bb518706",
          "FR-6": "4a67a56d0b7dfc57a6653f55ca640460caca598aa14c23f3d7d12ef7328bac74",
          "FR-7": "d77b4a1abfe2e957a15e40a27f56164617171d8f9fb612ff03f959741007083e",
          "FR-8": "730bcf67eb1eeb2f29249d5b176b53462d359037923545a49ee75c128207eff8",
          "FR-9": "fb66d95b1d42292d0df0968b9b54d70032144c0f228eaf583e5bd3bbab7037bf",
          "FR-10": "9fa5373f29cbdf9b533378ab9902d1fc2b53a559f7df285145ab9a25e93dd57f"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "5f8901def9925c6d8f714b63527aee1d9ace1431a37f0c86d33dc3992f0e4a7c"
        },
        "Карта реализации": {
          "Карта реализации": "13cf6196b1a31117ec9f0281697431ea29a6575f3c993a795227a65887cccbb3"
        },
        "Критерии приёмки": {
          "AC-1": "22d8a418ebfe78bbb6c5b9af237db9ad5094c65f409c6060bdff31d796765fe3",
          "AC-2": "0c79fc2cd86a8d94c52b24ad8f7d0209554bf8f42a50cadd49a583b725ee322b",
          "AC-3": "79dd36b4995fdc81df59a1482cdb652593bfa2f2fcf78d56fd8bbb681ca733ce",
          "AC-4": "248727507251650544f700e2e3c8e4e4d9b5c5943e7ff1a7532b10b040705f10",
          "AC-5": "17d54aae472c28178ef61340c1d2502b6f08cc1bcc5e5e294de60f8c204bfcd0",
          "AC-6": "453e01f1e9f77909bd6458ca9805266000c8991fdddc856636c8535abd60fb47",
          "AC-7": "4dd9f2237f2350745e45a5586cc4ad6803364254e024e0691bc3cadc64ae2737",
          "AC-8": "04feb8a4136f9848a0173651d6eef1ba120c91b476685dbf9e2f0dbe3f90c330",
          "AC-9": "bcd9f3d675b1853c865a8e61f996d648cfc7291ba2a8c89eca290ba9677e3908",
          "AC-10": "3ca87e0dcef5e12ad4f7b36f6b5da34b26bf8158eaebd7eb713c33b0fe8d4d66",
          "AC-11": "bd3e98ffff8dcb7119173f1fc25fd6c0e49753133d676ab9b52b56ead33e2262",
          "AC-12": "62173b6abadcbd42fe88a01789fd2cfcd5d11ec856966a7dca87b905429d8197",
          "AC-13": "ea0cd30cd4fd8a77e30156bb910c399744b8c4e3df3ee9feb01acf6eb0523f75",
          "AC-14": "2445686cb7d3249c6ca516d2d6fd3cc1b3ebd04c60d8bc567ea7e2641e4c5cc2"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "3b736d29e1827b33dacb14dc68ad7f8fcec3170a65525f059472f7032883bd83"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "8a4f711c52933008c0e9b799255910a577d966a0761f58cd626a964bf8a836b4",
      "sections": {
        "Метаданные": "986e5304347ab9b02587c97bd2486177240d883729084dc802f2b7251351f8dd",
        "Цель": "082d665055ac2180475cefe9d7e870a16102aaf05377d120815d29124519e1cb",
        "Границы": "1baf85b66f1ec70aae4af4744cafa23bab69b706725db75a8ddfad44193aa1a8",
        "Целевое поведение": "55160e2dcf78f2a3f06f9b75c09696e12651feace1c0740da581a821c09da189",
        "Требования": "f4ba5cabd2e4550cce10ee8f3abcf03abe8b6451a03dc82d0d71eb4b121d2f24",
        "Данные, права, ошибки": "c59ec2da1cfb5cfbdbb5616d4823625f028507d39c4b84dcb77293ab78fcf8b9",
        "Карта реализации": "d515f713d79560c8c258ab279762f3cf32a0fc18615dcc5681cb30b4cb5966ef",
        "Критерии приёмки": "867bbc8c4ce3e361d6ff1f697fcae6223e69ea90e808ad277c856191c80f4066",
        "Влияние на руководство": "9a639c0515e6c7ad2f825d3b98f3ba5b6b6394acc2033cbdef74906362ea3654"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "6b0fc0477e82d6bb92ca043a6e6ba1df791d2e60154206be406818695df7c708"
        },
        "Цель": {
          "Цель": "17ae0f3461a29bc30ce60db7eb1113f7b9d3335204756a033f0bb914d85f360f"
        },
        "Границы": {
          "Границы": "6314ac006189c83b06a0165525e05cd91b99f6cd7faea6d7735757117c7cc969"
        },
        "Целевое поведение": {
          "Целевое поведение": "995bb7438f942a9ab8e5d004cfcbb52aa60f94cfcefe9840b6b1cf8f9e104738"
        },
        "Требования": {
          "FR-1": "817a0fecc260370629ad28a39d5f1b7361bd9fd9d0325bff149673749139d93d",
          "FR-2": "baac62548965771ef2bb7373cb79b1065fd7b451c4449f64dc7372ed904cf5b3",
          "FR-3": "00ae7a31521c2a2b0fa3d1f53cfbcd5d3c67171e293a562c32f207336a217005",
          "FR-4": "401fe28a9db505066b2b3c0044714855608387ec935c05e6268ca09ff6910cad",
          "FR-5": "f221dfba8d8ab58e5ded5e7113c5665374ad9cdc0cb1651a445ad534bb518706",
          "FR-6": "4a67a56d0b7dfc57a6653f55ca640460caca598aa14c23f3d7d12ef7328bac74",
          "FR-7": "d77b4a1abfe2e957a15e40a27f56164617171d8f9fb612ff03f959741007083e",
          "FR-8": "730bcf67eb1eeb2f29249d5b176b53462d359037923545a49ee75c128207eff8",
          "FR-9": "fb66d95b1d42292d0df0968b9b54d70032144c0f228eaf583e5bd3bbab7037bf",
          "FR-10": "9fa5373f29cbdf9b533378ab9902d1fc2b53a559f7df285145ab9a25e93dd57f"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "5f8901def9925c6d8f714b63527aee1d9ace1431a37f0c86d33dc3992f0e4a7c"
        },
        "Карта реализации": {
          "Карта реализации": "13cf6196b1a31117ec9f0281697431ea29a6575f3c993a795227a65887cccbb3"
        },
        "Критерии приёмки": {
          "AC-1": "22d8a418ebfe78bbb6c5b9af237db9ad5094c65f409c6060bdff31d796765fe3",
          "AC-2": "0c79fc2cd86a8d94c52b24ad8f7d0209554bf8f42a50cadd49a583b725ee322b",
          "AC-3": "79dd36b4995fdc81df59a1482cdb652593bfa2f2fcf78d56fd8bbb681ca733ce",
          "AC-4": "248727507251650544f700e2e3c8e4e4d9b5c5943e7ff1a7532b10b040705f10",
          "AC-5": "17d54aae472c28178ef61340c1d2502b6f08cc1bcc5e5e294de60f8c204bfcd0",
          "AC-6": "453e01f1e9f77909bd6458ca9805266000c8991fdddc856636c8535abd60fb47",
          "AC-7": "4dd9f2237f2350745e45a5586cc4ad6803364254e024e0691bc3cadc64ae2737",
          "AC-8": "04feb8a4136f9848a0173651d6eef1ba120c91b476685dbf9e2f0dbe3f90c330",
          "AC-9": "bcd9f3d675b1853c865a8e61f996d648cfc7291ba2a8c89eca290ba9677e3908",
          "AC-10": "3ca87e0dcef5e12ad4f7b36f6b5da34b26bf8158eaebd7eb713c33b0fe8d4d66",
          "AC-11": "bd3e98ffff8dcb7119173f1fc25fd6c0e49753133d676ab9b52b56ead33e2262",
          "AC-12": "62173b6abadcbd42fe88a01789fd2cfcd5d11ec856966a7dca87b905429d8197",
          "AC-13": "ea0cd30cd4fd8a77e30156bb910c399744b8c4e3df3ee9feb01acf6eb0523f75",
          "AC-14": "2445686cb7d3249c6ca516d2d6fd3cc1b3ebd04c60d8bc567ea7e2641e4c5cc2"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "3b736d29e1827b33dacb14dc68ad7f8fcec3170a65525f059472f7032883bd83"
        }
      }
    }
  },
  "impl_fd_hash": {
    "backend": "8a4f711c52933008c0e9b799255910a577d966a0761f58cd626a964bf8a836b4",
    "backend_tests": "8a4f711c52933008c0e9b799255910a577d966a0761f58cd626a964bf8a836b4"
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "034e71aa81ad529c3a1eb4b630a0210278a81f10776d59720f668fea21761dec",
      "files": [
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/detect.rs",
        "deny.toml",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/tests/detect.rs",
        "crates/nimmd-core/tests/text.rs"
      ],
      "production_hash": "fa1c0b9351ae2a3eee793f09df4484b29d2e19bff154a929187a5cf996ffb6b4",
      "production_files": [
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/detect.rs",
        "deny.toml",
        "crates/nimmd-core/src/text.rs"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "034e71aa81ad529c3a1eb4b630a0210278a81f10776d59720f668fea21761dec",
      "files": [
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/detect.rs",
        "deny.toml",
        "crates/nimmd-core/src/text.rs",
        "crates/nimmd-core/tests/detect.rs",
        "crates/nimmd-core/tests/text.rs"
      ],
      "production_hash": "fa1c0b9351ae2a3eee793f09df4484b29d2e19bff154a929187a5cf996ffb6b4",
      "production_files": [
        "crates/nimmd-core/Cargo.toml",
        "Cargo.lock",
        "crates/nimmd-core/src/parse.rs",
        "crates/nimmd-core/src/lib.rs",
        "crates/nimmd-core/src/detect.rs",
        "deny.toml",
        "crates/nimmd-core/src/text.rs"
      ]
    }
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства в репозитории ещё нет, поведение видно хосту только через `Rendered.profile` (#43)."
  },
  "accepted_code_hash": {
    "backend-tests": "5a26b964b2e4194b50fda682ea4d362a904a75d0e2bbf23173fd5c31679d4ecf",
    "testing": "5a26b964b2e4194b50fda682ea4d362a904a75d0e2bbf23173fd5c31679d4ecf"
  }
}
```
<!-- afd-state-snapshot:end -->
