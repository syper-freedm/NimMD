# Состояние разработки: Зафиксировать LF в рабочей копии, чтобы тесты ядра проходили на Windows

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #91 (syper-freedm/NimMD)
- Размер: S; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-26T20:50:12+00:00

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
| implementation.backend_tests | not_required |
| implementation.frontend_tests | not_required |
| verification.code_review | approved |
| verification.security_review | approved |
| verification.testing | approved |

## Подзадачи

нет

## Журнал (последние события)

- 2026-10-26T20:44:08+00:00 `transition` — {"from": "implementation", "to": "code_review", "reason": null}
- 2026-10-26T20:44:08+00:00 `routed` — {"post_checks_parallel": true, "files_count": 2, "has_migrations": false, "touches_isolation": false, "skip": ["backend-tests", "documentation"]}
- 2026-10-26T20:44:13+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-26T20:44:14+00:00 `marked` — {"key": "implementation.backend_tests", "value": "not_required"}
- 2026-10-26T20:44:14+00:00 `note` — {"text": "backend принят; post-checks: ревью → QA (tests и docs пропущены route)"}
- 2026-10-26T20:49:58+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": []}
- 2026-10-26T20:49:58+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": []}
- 2026-10-26T20:49:58+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["control/suite-run.*.json нет — полный набор прогнан заново: cargo test --workspace --all-targets exit 0; just test exit 0 (document 40 passed, node 8/8; render_one_mib_within_budget — ignored до правки, вне её)", "SC-003: клон main с core.autocrlf=true — 152 w/crlf, 4 failed (class_contract, math_source_mode, bom_crlf_frontmatter_and_title, profile_matches_detect_on_fixtures); в ФД «3» — свойство базы, учтено QA-001", "SC-004: ветка не закоммичена — .gitattributes и document.rs закоммичены в клон main в tmp_dir, затем повторный клон с autocrlf=true: w/(crlf|mixed) нет, 153 w/lf, test result: ok. 40 passed", "Клоны в песочнице — только с --template= и --separate-git-dir (запись в .git/config и .git/hooks запрещена); клоны удалены, логи в control/.tmp/testing", "just lint в песочнице упал на PermissionError ~/.semgrep/semgrep.log (песочница, не дефект) — повтор вне песочницы: exit 0 (deny ok, gitleaks no leaks, semgrep No findings)"]}
- 2026-10-26T20:49:58+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-26T20:49:58+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-26T20:49:59+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-26T20:49:59+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-26T20:49:59+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-26T20:49:59+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-26T20:50:12+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 91,
  "title": "Зафиксировать LF в рабочей копии, чтобы тесты ядра проходили на Windows",
  "slug": "zafiksirovat-lf-v-rabochey-kopii-chtoby-testy-yadra",
  "branch": "fix/91-zafiksirovat-lf-v-rabochey-kopii-chtoby",
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
    "backend_tests": "not_required",
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
      "at": "2026-10-26T20:39:56+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-26T20:40:34+00:00",
      "event": "classified",
      "size": "S",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-26T20:40:35+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-26T20:40:35+00:00",
      "event": "note",
      "text": "ФД готов; старт ревью ФД"
    },
    {
      "at": "2026-10-26T20:42:04+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Утверждения ФД сверены: .gitattributes нет; git ls-files --eol — 152 i/lf + 1 i/-text (syntaxes.packdump); include_str! темы в src/document.rs и include_bytes! дампа в highlight.rs; сырой src.replace('\\n', \"\\r\\n\") — единственный в tests/document.rs:222. architecture.md не создан: долговременного решения нет."
      ]
    },
    {
      "at": "2026-10-26T20:42:05+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "Правка только .gitattributes и тестового кода: новой поверхности атаки, прав, данных нет; *.packdump binary сохраняет целостность include_bytes! (crates/nimmd-core/src/highlight.rs:108). security.md не нужен."
      ]
    },
    {
      "at": "2026-10-26T20:42:05+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001",
        "QA-002"
      ],
      "notes": [
        "Все AC-1…AC-4 проверяемы; test-cases.md: SC-001…SC-007, каждый AC покрыт.",
        "Сверено с кодом: файлы из AC-1 есть в индексе, дамп уже i/-text; profile_matches_detect_on_fixtures читает фикстуры через fs во время теста (AC-3 достижим), CRLF строится из сырого src (document.rs:222).",
        "QA-001, QA-002 — NOTICE к AC-2 (база main и w/none), вердикт не блокируют.",
        "Контроля доступа нет — сценарии объектного доступа неприменимы."
      ]
    },
    {
      "at": "2026-10-26T20:42:05+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:42:05+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:42:06+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:42:07+00:00",
      "event": "write_scope_extended",
      "phase": "backend",
      "paths": [
        ".gitattributes"
      ],
      "reason": "ФД FR-1, Карта реализации"
    },
    {
      "at": "2026-10-26T20:42:07+00:00",
      "event": "note",
      "text": "ФД одобрен (NOTICE QA-001/002 к AC-2); старт implementation"
    },
    {
      "at": "2026-10-26T20:44:08+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "changed-files.backend.txt: .gitattributes, crates/nimmd-core/tests/document.rs",
        "FR-1: .gitattributes — `* text=auto eol=lf`, `*.packdump binary`, с комментариями-причинами (спек 9)",
        "FR-2: git ls-files --eol без i/crlf и i/mixed — индекс уже в LF, renormalize не меняет содержимое; индекс не трогал (git add не выполнял, коммит — за оркестратором)",
        "FR-3: document.rs:222 — normalize_newlines(&src).replace('\\n', \"\\r\\n\")",
        "AC-1: check-attr — text: auto, eol: lf у трёх файлов; packdump binary: set, text: unset, i/-text",
        "AC-3: фикстура gfm/table_tasks.md в CRLF — profile_matches_detect_on_fixtures ok (1 passed); фикстура возвращена, git status чист по fixtures",
        "self-checks: cargo fmt --check ok; clippy -D warnings ok; cargo test --workspace --all-targets — все test result ok (document: 40 passed); cargo deny check — advisories/bans/licenses/sources ok (запуск вне песочницы: lock ~/.cargo/advisory-dbs read-only)",
        "diff-lint: clippy-changed clean, cargo-machete clean, semgrep-project-rules exit 0 (SEMGREP_LOG_FILE перенаправлен в tmp_dir: ~/.semgrep не пишется в песочнице)",
        "AC-2 (клон с autocrlf=true) не прогонял — это E2E QA (SC-003/SC-004); QA-001/QA-002 касаются формулировки проверки AC-2, в коде учитывать нечего",
        "just lint целиком не запускал (нужна сеть для vendor/npm); его шаги clippy/semgrep/deny/machete прогнаны",
        "ponytail: 0/0 accepted"
      ]
    },
    {
      "at": "2026-10-26T20:44:08+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-26T20:44:08+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-26T20:44:08+00:00",
      "event": "routed",
      "post_checks_parallel": true,
      "files_count": 2,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "backend-tests",
        "documentation"
      ]
    },
    {
      "at": "2026-10-26T20:44:13+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:44:14+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-26T20:44:14+00:00",
      "event": "note",
      "text": "backend принят; post-checks: ревью → QA (tests и docs пропущены route)"
    },
    {
      "at": "2026-10-26T20:49:58+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": []
    },
    {
      "at": "2026-10-26T20:49:58+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": []
    },
    {
      "at": "2026-10-26T20:49:58+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "control/suite-run.*.json нет — полный набор прогнан заново: cargo test --workspace --all-targets exit 0; just test exit 0 (document 40 passed, node 8/8; render_one_mib_within_budget — ignored до правки, вне её)",
        "SC-003: клон main с core.autocrlf=true — 152 w/crlf, 4 failed (class_contract, math_source_mode, bom_crlf_frontmatter_and_title, profile_matches_detect_on_fixtures); в ФД «3» — свойство базы, учтено QA-001",
        "SC-004: ветка не закоммичена — .gitattributes и document.rs закоммичены в клон main в tmp_dir, затем повторный клон с autocrlf=true: w/(crlf|mixed) нет, 153 w/lf, test result: ok. 40 passed",
        "Клоны в песочнице — только с --template= и --separate-git-dir (запись в .git/config и .git/hooks запрещена); клоны удалены, логи в control/.tmp/testing",
        "just lint в песочнице упал на PermissionError ~/.semgrep/semgrep.log (песочница, не дефект) — повтор вне песочницы: exit 0 (deny ok, gitleaks no leaks, semgrep No findings)"
      ]
    },
    {
      "at": "2026-10-26T20:49:58+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:49:58+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:49:59+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-26T20:49:59+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-26T20:49:59+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-26T20:49:59+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-26T20:50:12+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "5d8e8aba6b48d0a18f3f7b9ecc196f945aca683220133da7491684f9b369b4f5",
      "sections": {
        "Метаданные": "86a358a30bdfd265ab8e39eb23681f65583044dc61a9dd2b3eead6339d62f5ed",
        "Цель": "a00340e3894b7bed12b151b33984b7ba2a81abff16e83ac9b901d3a1e4113186",
        "Границы": "f4bd540a023b402b96df04031895e220260fef0382a08ec55b9b45fbc00f5943",
        "Целевое поведение": "d916cd0a62795db985bf9fe2e5950f34c82579db04567a95441e02656057ad2c",
        "Требования": "dd7dd6cc8221378ba984f59cf11179284191a0fb4d75da86d255acd81c7e5945",
        "Данные, права, ошибки": "c310d75441cd0e7d6c9650fd9f3c9ce5f05297367f15b4a8a9847c84bf945a3b",
        "Карта реализации": "eb9b49b2f5b6d60d006dfe0d20bec6cbb156d4edfae5065dd738262593b45e92",
        "Критерии приёмки": "e44e540c7729b7987dba17ad4a3a7f434d5077ac82c9c6ca8c7813e85f1ea3b8",
        "Влияние на руководство": "45254709581e4cea05801255d560d1c3d9d7c8826e124e3185c7c9e73b9bd3c1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "b0a74accd245e32020ade8ff21fb72ce54259da5e87d9fefa248bad6a7d38ef8"
        },
        "Цель": {
          "Цель": "e140e5756a0890de868d65409a4f90999dbb8c443815baa18e86899ac40c54d8"
        },
        "Границы": {
          "Границы": "dd49b303c13a2d6f8037243efb8a22afa2be51102937092c0cc54f587adc3a8d"
        },
        "Целевое поведение": {
          "Целевое поведение": "8385288defc823d5bb07c01e10a3e48c332cab983a7d8937b74899d853f1a167"
        },
        "Требования": {
          "FR-1": "2882eca3afc39033f040d951e05b9e2b41a6cec7901bf591644faafed91501ff",
          "FR-2": "4d732a779bb76d87d48fc260c2b5e57af6048b006c57b1e1ebd278383645935f",
          "FR-3": "3b92a20286252b17291692f230f171ab2bd10a5ec83e62855fd82926f80e76aa"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "4fb04794d97e8104270c3994a907e871918d9be6c6c1f6aedfb19abb04e52164"
        },
        "Карта реализации": {
          "Карта реализации": "fb9cb6e7152d8df62c53f944c16f77aae38a6e05df634408cd3aeae069a8e924"
        },
        "Критерии приёмки": {
          "AC-1": "e56486adf2cf87ed9129aecb26c4b7c0d220dd741699baa68ffb3e4dc7bdbca6",
          "AC-2": "3a602c9620b1d90556f6c36ced947232607709dcca50b00d069c7a5a80e0655b",
          "AC-3": "f9957d7ca638da6188372265d1896f9f5cf82c07aa4591c5506cf4434fab36b6",
          "AC-4": "c59ec6a5e3e3bedbc8922f6bb2b538a41af2cc2129e9069354b56cf54d09873a"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "efb622532b2cab39ff51c18c89847a902865c77629f70dcbca4707029013a196"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "5d8e8aba6b48d0a18f3f7b9ecc196f945aca683220133da7491684f9b369b4f5",
      "sections": {
        "Метаданные": "86a358a30bdfd265ab8e39eb23681f65583044dc61a9dd2b3eead6339d62f5ed",
        "Цель": "a00340e3894b7bed12b151b33984b7ba2a81abff16e83ac9b901d3a1e4113186",
        "Границы": "f4bd540a023b402b96df04031895e220260fef0382a08ec55b9b45fbc00f5943",
        "Целевое поведение": "d916cd0a62795db985bf9fe2e5950f34c82579db04567a95441e02656057ad2c",
        "Требования": "dd7dd6cc8221378ba984f59cf11179284191a0fb4d75da86d255acd81c7e5945",
        "Данные, права, ошибки": "c310d75441cd0e7d6c9650fd9f3c9ce5f05297367f15b4a8a9847c84bf945a3b",
        "Карта реализации": "eb9b49b2f5b6d60d006dfe0d20bec6cbb156d4edfae5065dd738262593b45e92",
        "Критерии приёмки": "e44e540c7729b7987dba17ad4a3a7f434d5077ac82c9c6ca8c7813e85f1ea3b8",
        "Влияние на руководство": "45254709581e4cea05801255d560d1c3d9d7c8826e124e3185c7c9e73b9bd3c1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "b0a74accd245e32020ade8ff21fb72ce54259da5e87d9fefa248bad6a7d38ef8"
        },
        "Цель": {
          "Цель": "e140e5756a0890de868d65409a4f90999dbb8c443815baa18e86899ac40c54d8"
        },
        "Границы": {
          "Границы": "dd49b303c13a2d6f8037243efb8a22afa2be51102937092c0cc54f587adc3a8d"
        },
        "Целевое поведение": {
          "Целевое поведение": "8385288defc823d5bb07c01e10a3e48c332cab983a7d8937b74899d853f1a167"
        },
        "Требования": {
          "FR-1": "2882eca3afc39033f040d951e05b9e2b41a6cec7901bf591644faafed91501ff",
          "FR-2": "4d732a779bb76d87d48fc260c2b5e57af6048b006c57b1e1ebd278383645935f",
          "FR-3": "3b92a20286252b17291692f230f171ab2bd10a5ec83e62855fd82926f80e76aa"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "4fb04794d97e8104270c3994a907e871918d9be6c6c1f6aedfb19abb04e52164"
        },
        "Карта реализации": {
          "Карта реализации": "fb9cb6e7152d8df62c53f944c16f77aae38a6e05df634408cd3aeae069a8e924"
        },
        "Критерии приёмки": {
          "AC-1": "e56486adf2cf87ed9129aecb26c4b7c0d220dd741699baa68ffb3e4dc7bdbca6",
          "AC-2": "3a602c9620b1d90556f6c36ced947232607709dcca50b00d069c7a5a80e0655b",
          "AC-3": "f9957d7ca638da6188372265d1896f9f5cf82c07aa4591c5506cf4434fab36b6",
          "AC-4": "c59ec6a5e3e3bedbc8922f6bb2b538a41af2cc2129e9069354b56cf54d09873a"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "efb622532b2cab39ff51c18c89847a902865c77629f70dcbca4707029013a196"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "5d8e8aba6b48d0a18f3f7b9ecc196f945aca683220133da7491684f9b369b4f5",
      "sections": {
        "Метаданные": "86a358a30bdfd265ab8e39eb23681f65583044dc61a9dd2b3eead6339d62f5ed",
        "Цель": "a00340e3894b7bed12b151b33984b7ba2a81abff16e83ac9b901d3a1e4113186",
        "Границы": "f4bd540a023b402b96df04031895e220260fef0382a08ec55b9b45fbc00f5943",
        "Целевое поведение": "d916cd0a62795db985bf9fe2e5950f34c82579db04567a95441e02656057ad2c",
        "Требования": "dd7dd6cc8221378ba984f59cf11179284191a0fb4d75da86d255acd81c7e5945",
        "Данные, права, ошибки": "c310d75441cd0e7d6c9650fd9f3c9ce5f05297367f15b4a8a9847c84bf945a3b",
        "Карта реализации": "eb9b49b2f5b6d60d006dfe0d20bec6cbb156d4edfae5065dd738262593b45e92",
        "Критерии приёмки": "e44e540c7729b7987dba17ad4a3a7f434d5077ac82c9c6ca8c7813e85f1ea3b8",
        "Влияние на руководство": "45254709581e4cea05801255d560d1c3d9d7c8826e124e3185c7c9e73b9bd3c1"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "b0a74accd245e32020ade8ff21fb72ce54259da5e87d9fefa248bad6a7d38ef8"
        },
        "Цель": {
          "Цель": "e140e5756a0890de868d65409a4f90999dbb8c443815baa18e86899ac40c54d8"
        },
        "Границы": {
          "Границы": "dd49b303c13a2d6f8037243efb8a22afa2be51102937092c0cc54f587adc3a8d"
        },
        "Целевое поведение": {
          "Целевое поведение": "8385288defc823d5bb07c01e10a3e48c332cab983a7d8937b74899d853f1a167"
        },
        "Требования": {
          "FR-1": "2882eca3afc39033f040d951e05b9e2b41a6cec7901bf591644faafed91501ff",
          "FR-2": "4d732a779bb76d87d48fc260c2b5e57af6048b006c57b1e1ebd278383645935f",
          "FR-3": "3b92a20286252b17291692f230f171ab2bd10a5ec83e62855fd82926f80e76aa"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "4fb04794d97e8104270c3994a907e871918d9be6c6c1f6aedfb19abb04e52164"
        },
        "Карта реализации": {
          "Карта реализации": "fb9cb6e7152d8df62c53f944c16f77aae38a6e05df634408cd3aeae069a8e924"
        },
        "Критерии приёмки": {
          "AC-1": "e56486adf2cf87ed9129aecb26c4b7c0d220dd741699baa68ffb3e4dc7bdbca6",
          "AC-2": "3a602c9620b1d90556f6c36ced947232607709dcca50b00d069c7a5a80e0655b",
          "AC-3": "f9957d7ca638da6188372265d1896f9f5cf82c07aa4591c5506cf4434fab36b6",
          "AC-4": "c59ec6a5e3e3bedbc8922f6bb2b538a41af2cc2129e9069354b56cf54d09873a"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "efb622532b2cab39ff51c18c89847a902865c77629f70dcbca4707029013a196"
        }
      }
    }
  },
  "write_scope_extensions": {
    "backend": [
      {
        "path": ".gitattributes",
        "reason": "ФД FR-1, Карта реализации"
      }
    ]
  },
  "impl_fd_hash": {
    "backend": "5d8e8aba6b48d0a18f3f7b9ecc196f945aca683220133da7491684f9b369b4f5"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского руководства нет.",
    "backend-tests": "ФД, «Карта реализации»: Новые тесты: нет (правка конфигурации git и одного существующего теста; проверка — клоном с `core.autocrlf=true`, AC-2)."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "10670b83fdd0ab079ac39c5b07b56645235fcbda3f64d31f04b241e486f5101e",
      "files": [
        ".gitattributes",
        "crates/nimmd-core/tests/document.rs"
      ],
      "production_hash": "4cef74f4312b97479e035131cdf557a6cb87884d45b1bc6a177b95798dc740db",
      "production_files": [
        ".gitattributes"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "10670b83fdd0ab079ac39c5b07b56645235fcbda3f64d31f04b241e486f5101e",
      "files": [
        ".gitattributes",
        "crates/nimmd-core/tests/document.rs"
      ],
      "production_hash": "4cef74f4312b97479e035131cdf557a6cb87884d45b1bc6a177b95798dc740db",
      "production_files": [
        ".gitattributes"
      ]
    }
  },
  "accepted_code_hash": {
    "testing": "10670b83fdd0ab079ac39c5b07b56645235fcbda3f64d31f04b241e486f5101e"
  }
}
```
<!-- afd-state-snapshot:end -->
