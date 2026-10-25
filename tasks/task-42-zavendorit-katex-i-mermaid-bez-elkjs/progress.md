# Состояние разработки: Завендорить KaTeX и Mermaid без elkjs

> Генерируется `afd.py` при каждой смене состояния — не редактировать вручную.
> Возобновление после прерывания: `afd.py resume --task <dir>`.
> Восстановление утерянного `control/status.yaml`: `afd.py restore --task <dir>`.

- Issue: #42 (syper-freedm/NimMD)
- Размер: S; слои: backend
- Состояние: **completion**
- Обновлено: 2026-10-25T09:41:04+00:00

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

- 2026-10-25T09:34:01+00:00 `transition` — {"from": "implementation", "to": "code_review", "reason": null}
- 2026-10-25T09:34:01+00:00 `routed` — {"post_checks_parallel": false, "files_count": 12, "has_migrations": false, "touches_isolation": false, "skip": ["backend-tests", "documentation"]}
- 2026-10-25T09:34:07+00:00 `marked` — {"key": "documents.user_guide", "value": "not_required"}
- 2026-10-25T09:34:07+00:00 `marked` — {"key": "implementation.backend_tests", "value": "not_required"}
- 2026-10-25T09:34:07+00:00 `note` — {"text": "post-checks: code-review, security-code-review, testing; skip documentation, backend-tests"}
- 2026-10-25T09:40:05+00:00 `handback_accepted` — {"phase": "code-review", "verdict": "APPROVED", "findings": [], "notes": ["git status/diff совпадает с control/changed-files.txt (12 файлов); node check-mermaid.mjs -> ok: elkjs не найден (rc=0); pnpm why elkjs пуст; just lint/test и сетевой just vendor не перезапускались"]}
- 2026-10-25T09:40:05+00:00 `handback_accepted` — {"phase": "security-code-review", "verdict": "APPROVED", "findings": [], "notes": ["cargo deny check запущен вне песочницы (lock advisory-db в ~/.cargo read-only); semgrep — с доступом к semgrep.dev и логами в tmp_dir (~/.semgrep не записываем)", "Pre-existing: cargo deny предупреждает unmatched license exception cbindgen в deny.toml — к задаче не относится"]}
- 2026-10-25T09:40:05+00:00 `handback_accepted` — {"phase": "testing", "verdict": "APPROVED", "findings": [], "notes": ["suite-run.*.json отсутствуют — just lint, just test, cargo test --workspace --all-targets прогнаны заново: все код 0, 0 skipped", "pnpm install в песочнице падает ERR_PNPM_STORE_DIR_OPEN_OPERATION_LOCK (Operation not permitted) — команды pnpm/just vendor/just lint запущены вне песочницы", "SC-001/005/006 выполнены на копиях packages/vendor в control/.tmp/testing; рабочие node_modules и dist не удалялись", "SC-002: задача не закоммичена, поэтому сверялся git status --short до и после just vendor — совпадает (dist/ и node_modules/ игнорируются)"]}
- 2026-10-25T09:40:05+00:00 `marked` — {"key": "verification.code_review", "value": "approved"}
- 2026-10-25T09:40:05+00:00 `marked` — {"key": "verification.security_review", "value": "approved"}
- 2026-10-25T09:40:05+00:00 `transition` — {"from": "code_review", "to": "testing", "reason": null}
- 2026-10-25T09:40:05+00:00 `marked` — {"key": "verification.testing", "value": "approved"}
- 2026-10-25T09:40:06+00:00 `transition` — {"from": "testing", "to": "documentation", "reason": null}
- 2026-10-25T09:40:06+00:00 `transition` — {"from": "documentation", "to": "completion", "reason": null}
- 2026-10-25T09:41:04+00:00 `note` — {"text": "гейт шага 17"}

## Снимок состояния (для restore)

<!-- afd-state-snapshot:begin -->
```json
{
  "repo": "syper-freedm/NimMD",
  "issue": 42,
  "title": "Завендорить KaTeX и Mermaid без elkjs",
  "slug": "zavendorit-katex-i-mermaid-bez-elkjs",
  "branch": "chore/42-zavendorit-katex-i-mermaid-bez-elkjs",
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
      "at": "2026-10-25T09:20:10+00:00",
      "event": "initialized",
      "state": "requirements"
    },
    {
      "at": "2026-10-25T09:20:11+00:00",
      "event": "note",
      "text": "preflight OK, SMOKE_BUILD cached 0s, PREFLIGHT 12s; ветка харнесса claude/project-thread-s1ica3 вместо afd branch; режим делегированных решений (бриф координатора)"
    },
    {
      "at": "2026-10-25T09:27:36+00:00",
      "event": "classified",
      "size": "S",
      "areas": [
        "backend"
      ]
    },
    {
      "at": "2026-10-25T09:28:20+00:00",
      "event": "transition",
      "from": "requirements",
      "to": "document_review",
      "reason": null
    },
    {
      "at": "2026-10-25T09:28:20+00:00",
      "event": "note",
      "text": "ФД готов; запуск FD-ревью (arch, sec, qa)"
    },
    {
      "at": "2026-10-25T09:30:16+00:00",
      "event": "handback_accepted",
      "phase": "architecture-document-review",
      "verdict": "APPROVED",
      "findings": [
        "ARCH-001"
      ],
      "notes": [
        "Сверено с кодом: justfile (lint, npm-licenses уже вызывает packages/vendor), ci.yml job lint, .gitignore (node_modules/ уже игнорируется), .semgrep.yml (исключение /packages/vendor/** уже есть), scripts/check-npm-licenses.mjs; дублей packages/vendor нет.",
        "Не проверено: что mermaid 12.2.0 действительно тянет elkjs и что штатный mermaid.min.js содержит org.eclipse.elk (AC-2, размерная проверка FR-4) — node_modules в дереве нет.",
        "architecture.md не создан: долговременное решение (заглушка ELK, dagre в хостах) уже зафиксировано в спеке 12 и decision-log."
      ]
    },
    {
      "at": "2026-10-25T09:30:16+00:00",
      "event": "handback_accepted",
      "phase": "security-document-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "security.md не создан: новых прав, классов чувствительных данных и защищаемых потоков нет"
      ]
    },
    {
      "at": "2026-10-25T09:30:16+00:00",
      "event": "handback_accepted",
      "phase": "qa-document-review",
      "verdict": "APPROVED",
      "findings": [
        "QA-001"
      ],
      "notes": [
        "Все AC-1..AC-5 проверяемы; QA-001 (NOTICE, AC-2): уточнить, что бандл грузится в vm.createContext({}) без исключения",
        "Сверено с кодом: justfile npm-licenses уже проверяет packages/vendor, .semgrep.yml исключает /packages/vendor/**, node_modules/ в .gitignore, в ci.yml job lint pnpm пока не ставится",
        "Версии katex 0.19.1, mermaid 12.2.0, esbuild 0.28.2, pnpm 12.15.0 и наличие org.eclipse.elk в штатном бандле mermaid не проверял: пакета packages/vendor ещё нет"
      ]
    },
    {
      "at": "2026-10-25T09:30:27+00:00",
      "event": "marked",
      "key": "documents.functional_design",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:30:27+00:00",
      "event": "marked",
      "key": "documents.test_cases",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "marked",
      "key": "document_reviews.architecture",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "marked",
      "key": "document_reviews.security",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "marked",
      "key": "document_reviews.qa",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "marked",
      "key": "document_reviews.ux",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "transition",
      "from": "document_review",
      "to": "implementation",
      "reason": null
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "marked",
      "key": "implementation.frontend",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "marked",
      "key": "implementation.frontend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "note",
      "text": "FD-ревью APPROVED (NOTICE ARCH-001, QA-001 — заметки разработчику); запуск implementation backend"
    },
    {
      "at": "2026-10-25T09:30:28+00:00",
      "event": "write_scope_extended",
      "phase": "backend",
      "paths": [
        "packages/vendor/",
        ".gitignore"
      ],
      "reason": "Карта реализации FR-1–FR-6: packages/vendor и .gitignore"
    },
    {
      "at": "2026-10-25T09:34:01+00:00",
      "event": "handback_accepted",
      "phase": "backend",
      "verdict": "DONE",
      "findings": [],
      "notes": [
        "FD deviation (FR-1, SC-008): pnpm always writes the override into pnpm-lock.yaml as an 'overrides: elkjs: \"-\"' section, so 'grep -c elkjs pnpm-lock.yaml' returns 1, not 0. No elkjs package entry exists in the lockfile, 'pnpm why elkjs' prints nothing, and node_modules/.pnpm has no elkjs@*. SC-008 needs to check for package entries, not the bare string",
        "Addition outside the literal FD (FR-1): pnpm-workspace.yaml also has 'allowBuilds: { esbuild: false }'. Without it pnpm 12 exits non-zero with ERR_PNPM_IGNORED_BUILDS (esbuild), which breaks FR-5. Install scripts still do not run",
        "Addition outside the literal FD (FR-7): zizmor flags 'npm install -g pnpm@12.15.0' as adhoc-packages (low). Suppressed with a YAML comment on the step's 'run: |' line: '# zizmor: ignore[adhoc-packages] pnpm закреплён точной версией'. An inline comment on the command line inside the block scalar is not recognized",
        "just lint: exit 0 (fmt, clippy, semgrep, cargo deny, vendor -> 'ok: elkjs не найден', npm-licenses for packages/vendor, machete, gitleaks git, actionlint, zizmor)",
        "just test: exit 0 (cargo test --workspace, node --test scripts: 2/2 pass)",
        "AC-1/AC-2 checked by hand: fetch.sh run from $HOME -> exit 0; dist/ holds exactly the 5 FR-4 entries; fonts/ is non-empty; org.eclipse.elk count is 0 in our bundle vs 9 upstream; size 3749361 < 5494801 bytes; vm.createContext({}) -> mermaid.initialize/run are functions (QA-001); git status is clean apart from the new tracked files",
        "AC-3 checked by hand in tmp_dir (copies deleted afterwards): build without stub plugin and override -> exit 1, prints node_modules/.pnpm/elkjs@0.9.3/node_modules/elkjs/lib/elk.bundled.js; check with org.eclipse.elk appended -> exit 1 and names the string",
        "ARCH-001: ran 'pnpm -C packages/vendor audit' by hand -> No known vulnerabilities found. diff-lint backend: no changed Rust files",
        "pnpm install had to run outside the sandbox: the store lock in ~/Library/pnpm/store is not writable from inside it (ERR_PNPM_STORE_DIR_OPEN_OPERATION_LOCK). Use dangerouslyDisableSandbox for 'just vendor' / 'just lint'",
        "New files are untracked and not committed. packages/vendor/node_modules and dist are gitignored"
      ]
    },
    {
      "at": "2026-10-25T09:34:01+00:00",
      "event": "marked",
      "key": "implementation.backend",
      "value": "done"
    },
    {
      "at": "2026-10-25T09:34:01+00:00",
      "event": "note",
      "text": "backend DONE; ФД FR-1/FR-7 и SC-008 уточнены по фактам pnpm 12 (override в lockfile, allowBuilds, zizmor ignore) до accept"
    },
    {
      "at": "2026-10-25T09:34:01+00:00",
      "event": "transition",
      "from": "implementation",
      "to": "code_review",
      "reason": null
    },
    {
      "at": "2026-10-25T09:34:01+00:00",
      "event": "routed",
      "post_checks_parallel": false,
      "files_count": 12,
      "has_migrations": false,
      "touches_isolation": false,
      "skip": [
        "backend-tests",
        "documentation"
      ]
    },
    {
      "at": "2026-10-25T09:34:07+00:00",
      "event": "marked",
      "key": "documents.user_guide",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T09:34:07+00:00",
      "event": "marked",
      "key": "implementation.backend_tests",
      "value": "not_required"
    },
    {
      "at": "2026-10-25T09:34:07+00:00",
      "event": "note",
      "text": "post-checks: code-review, security-code-review, testing; skip documentation, backend-tests"
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "handback_accepted",
      "phase": "code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "git status/diff совпадает с control/changed-files.txt (12 файлов); node check-mermaid.mjs -> ok: elkjs не найден (rc=0); pnpm why elkjs пуст; just lint/test и сетевой just vendor не перезапускались"
      ]
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "handback_accepted",
      "phase": "security-code-review",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "cargo deny check запущен вне песочницы (lock advisory-db в ~/.cargo read-only); semgrep — с доступом к semgrep.dev и логами в tmp_dir (~/.semgrep не записываем)",
        "Pre-existing: cargo deny предупреждает unmatched license exception cbindgen в deny.toml — к задаче не относится"
      ]
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "handback_accepted",
      "phase": "testing",
      "verdict": "APPROVED",
      "findings": [],
      "notes": [
        "suite-run.*.json отсутствуют — just lint, just test, cargo test --workspace --all-targets прогнаны заново: все код 0, 0 skipped",
        "pnpm install в песочнице падает ERR_PNPM_STORE_DIR_OPEN_OPERATION_LOCK (Operation not permitted) — команды pnpm/just vendor/just lint запущены вне песочницы",
        "SC-001/005/006 выполнены на копиях packages/vendor в control/.tmp/testing; рабочие node_modules и dist не удалялись",
        "SC-002: задача не закоммичена, поэтому сверялся git status --short до и после just vendor — совпадает (dist/ и node_modules/ игнорируются)"
      ]
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "marked",
      "key": "verification.code_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "marked",
      "key": "verification.security_review",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "transition",
      "from": "code_review",
      "to": "testing",
      "reason": null
    },
    {
      "at": "2026-10-25T09:40:05+00:00",
      "event": "marked",
      "key": "verification.testing",
      "value": "approved"
    },
    {
      "at": "2026-10-25T09:40:06+00:00",
      "event": "transition",
      "from": "testing",
      "to": "documentation",
      "reason": null
    },
    {
      "at": "2026-10-25T09:40:06+00:00",
      "event": "transition",
      "from": "documentation",
      "to": "completion",
      "reason": null
    },
    {
      "at": "2026-10-25T09:41:04+00:00",
      "event": "note",
      "text": "гейт шага 17"
    }
  ],
  "fd_review_snapshots": {
    "architecture-document-review": {
      "fd_hash": "c3466608e54a356895017d89882088d6659f47aaae8c3818f2700613e0238f21",
      "sections": {
        "Метаданные": "74173588c59dc9c0644d53975db3ddbd08d974e619d97a102f97fb7c722fca7f",
        "Цель": "478ac3886f421c1dee52a44eb57e450472de72637899a43596f6b0115d6b9997",
        "Границы": "2199a6b6a7ac394e0ec5e6f13f77ebeb9dfa175d015b2d467a1e19153f65d0b8",
        "Целевое поведение": "f560b86fd25324cc46e826d02fed72979609ed310838eb207ac0431ddf3c2190",
        "Требования": "86d78a12ec85eec314e1297513cabeb463d914cbeaf6cee2082336eeec25aa23",
        "Данные, права, ошибки": "e202ec63b461f70055abb32cdd285de179b9ef323a06f0acf2cf6a14213f24e9",
        "Карта реализации": "a21ed64acb65812f5cf8ead9d5521e509ff914b2d62d10e6cefd259d276a04bb",
        "Критерии приёмки": "1d6b211863d43cef58f8a3624f148b12ad67962adcedd6068f4c8e95832a7aad",
        "Влияние на руководство": "0f34bb2a5286c51f04d75c1a3e01fade1a626e052540186e9688cf244000b6b7"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "2cbdb762e5140a050d93aa9ad3e38774c7d490f3136517677a8c57258f67c96a"
        },
        "Цель": {
          "Цель": "470d8fc19660b118bb090c025784d15c78680264675c47c15ccf589dd8e3dd71"
        },
        "Границы": {
          "Границы": "a45c1e3fb58f6bca7a2fae2d0ed35fbf0b07ea8dbbdfd2afec4d16a09f3893d3"
        },
        "Целевое поведение": {
          "Целевое поведение": "b5200cbd998acba2ef0bcc1a4ea62bc415172c0ef464bd57d85ebcd0e6e74b24"
        },
        "Требования": {
          "FR-1": "6f9bb14130f41ed43932efbe24ab9ba4a559859d76a8a2fcadcded5df83ce5e7",
          "FR-2": "3c1fe9f84ed0c7119791d385e3bd4d6c6c15a3569976f6ee0c3437f9a72cb87d",
          "FR-3": "4acb6ff3790183b9d5683defb9473a3e41f67101e41b59782104d953898f5adf",
          "FR-4": "185336b76c7746cff2a6e001a6b8aa7f856b68574374b613f27ade12d698df77",
          "FR-5": "801fef37008de5c99c72332f6e431770c156f856748c5663ceae8106f14ea978",
          "FR-6": "18c13c29dd41ad9bec055bc96b6b12d1377788312445a00427bdf38741d47d0a",
          "FR-7": "062bd07bbfdca08ea6a14442dd82174860fba40d01cd4b9521a150c0f296b958"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "4a37dcb6543f3055de63d7270749b3770448e4f8a8ed7a615370bd365ea74a6a"
        },
        "Карта реализации": {
          "Карта реализации": "1e9001f9f866abf884e3fbd9e50d2907e16387248f236bbf7e89398a29312708"
        },
        "Критерии приёмки": {
          "AC-1": "ebcd2a44670ced6aee9b53c76860b308f78c8987d1bcd8d1556b2f8e08c6cc2b",
          "AC-2": "3d0947b286103ab19758bf24de265db4363a065348303fa316da34f55433618c",
          "AC-3": "06849a076e852f7892507db4e04e7c3533f2d2dca3691ce9b9b33cd3e302cc24",
          "AC-4": "00c43fdd48221cac2ad975b6943d1a5b57b475ec9377686fc2f1e5708e1840ba",
          "AC-5": "1cfcb2fe2f3e50e1ab965a33be1a6f9e8ff3a52b155b2012cea2aefdaf0a7135"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "1f4f0155b9cdd6e2e059aac61778805fec5dca145bf5ca5d0fbe2b31deca537a"
        }
      }
    },
    "security-document-review": {
      "fd_hash": "c3466608e54a356895017d89882088d6659f47aaae8c3818f2700613e0238f21",
      "sections": {
        "Метаданные": "74173588c59dc9c0644d53975db3ddbd08d974e619d97a102f97fb7c722fca7f",
        "Цель": "478ac3886f421c1dee52a44eb57e450472de72637899a43596f6b0115d6b9997",
        "Границы": "2199a6b6a7ac394e0ec5e6f13f77ebeb9dfa175d015b2d467a1e19153f65d0b8",
        "Целевое поведение": "f560b86fd25324cc46e826d02fed72979609ed310838eb207ac0431ddf3c2190",
        "Требования": "86d78a12ec85eec314e1297513cabeb463d914cbeaf6cee2082336eeec25aa23",
        "Данные, права, ошибки": "e202ec63b461f70055abb32cdd285de179b9ef323a06f0acf2cf6a14213f24e9",
        "Карта реализации": "a21ed64acb65812f5cf8ead9d5521e509ff914b2d62d10e6cefd259d276a04bb",
        "Критерии приёмки": "1d6b211863d43cef58f8a3624f148b12ad67962adcedd6068f4c8e95832a7aad",
        "Влияние на руководство": "0f34bb2a5286c51f04d75c1a3e01fade1a626e052540186e9688cf244000b6b7"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "2cbdb762e5140a050d93aa9ad3e38774c7d490f3136517677a8c57258f67c96a"
        },
        "Цель": {
          "Цель": "470d8fc19660b118bb090c025784d15c78680264675c47c15ccf589dd8e3dd71"
        },
        "Границы": {
          "Границы": "a45c1e3fb58f6bca7a2fae2d0ed35fbf0b07ea8dbbdfd2afec4d16a09f3893d3"
        },
        "Целевое поведение": {
          "Целевое поведение": "b5200cbd998acba2ef0bcc1a4ea62bc415172c0ef464bd57d85ebcd0e6e74b24"
        },
        "Требования": {
          "FR-1": "6f9bb14130f41ed43932efbe24ab9ba4a559859d76a8a2fcadcded5df83ce5e7",
          "FR-2": "3c1fe9f84ed0c7119791d385e3bd4d6c6c15a3569976f6ee0c3437f9a72cb87d",
          "FR-3": "4acb6ff3790183b9d5683defb9473a3e41f67101e41b59782104d953898f5adf",
          "FR-4": "185336b76c7746cff2a6e001a6b8aa7f856b68574374b613f27ade12d698df77",
          "FR-5": "801fef37008de5c99c72332f6e431770c156f856748c5663ceae8106f14ea978",
          "FR-6": "18c13c29dd41ad9bec055bc96b6b12d1377788312445a00427bdf38741d47d0a",
          "FR-7": "062bd07bbfdca08ea6a14442dd82174860fba40d01cd4b9521a150c0f296b958"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "4a37dcb6543f3055de63d7270749b3770448e4f8a8ed7a615370bd365ea74a6a"
        },
        "Карта реализации": {
          "Карта реализации": "1e9001f9f866abf884e3fbd9e50d2907e16387248f236bbf7e89398a29312708"
        },
        "Критерии приёмки": {
          "AC-1": "ebcd2a44670ced6aee9b53c76860b308f78c8987d1bcd8d1556b2f8e08c6cc2b",
          "AC-2": "3d0947b286103ab19758bf24de265db4363a065348303fa316da34f55433618c",
          "AC-3": "06849a076e852f7892507db4e04e7c3533f2d2dca3691ce9b9b33cd3e302cc24",
          "AC-4": "00c43fdd48221cac2ad975b6943d1a5b57b475ec9377686fc2f1e5708e1840ba",
          "AC-5": "1cfcb2fe2f3e50e1ab965a33be1a6f9e8ff3a52b155b2012cea2aefdaf0a7135"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "1f4f0155b9cdd6e2e059aac61778805fec5dca145bf5ca5d0fbe2b31deca537a"
        }
      }
    },
    "qa-document-review": {
      "fd_hash": "c3466608e54a356895017d89882088d6659f47aaae8c3818f2700613e0238f21",
      "sections": {
        "Метаданные": "74173588c59dc9c0644d53975db3ddbd08d974e619d97a102f97fb7c722fca7f",
        "Цель": "478ac3886f421c1dee52a44eb57e450472de72637899a43596f6b0115d6b9997",
        "Границы": "2199a6b6a7ac394e0ec5e6f13f77ebeb9dfa175d015b2d467a1e19153f65d0b8",
        "Целевое поведение": "f560b86fd25324cc46e826d02fed72979609ed310838eb207ac0431ddf3c2190",
        "Требования": "86d78a12ec85eec314e1297513cabeb463d914cbeaf6cee2082336eeec25aa23",
        "Данные, права, ошибки": "e202ec63b461f70055abb32cdd285de179b9ef323a06f0acf2cf6a14213f24e9",
        "Карта реализации": "a21ed64acb65812f5cf8ead9d5521e509ff914b2d62d10e6cefd259d276a04bb",
        "Критерии приёмки": "1d6b211863d43cef58f8a3624f148b12ad67962adcedd6068f4c8e95832a7aad",
        "Влияние на руководство": "0f34bb2a5286c51f04d75c1a3e01fade1a626e052540186e9688cf244000b6b7"
      },
      "items": {
        "Метаданные": {
          "Метаданные": "2cbdb762e5140a050d93aa9ad3e38774c7d490f3136517677a8c57258f67c96a"
        },
        "Цель": {
          "Цель": "470d8fc19660b118bb090c025784d15c78680264675c47c15ccf589dd8e3dd71"
        },
        "Границы": {
          "Границы": "a45c1e3fb58f6bca7a2fae2d0ed35fbf0b07ea8dbbdfd2afec4d16a09f3893d3"
        },
        "Целевое поведение": {
          "Целевое поведение": "b5200cbd998acba2ef0bcc1a4ea62bc415172c0ef464bd57d85ebcd0e6e74b24"
        },
        "Требования": {
          "FR-1": "6f9bb14130f41ed43932efbe24ab9ba4a559859d76a8a2fcadcded5df83ce5e7",
          "FR-2": "3c1fe9f84ed0c7119791d385e3bd4d6c6c15a3569976f6ee0c3437f9a72cb87d",
          "FR-3": "4acb6ff3790183b9d5683defb9473a3e41f67101e41b59782104d953898f5adf",
          "FR-4": "185336b76c7746cff2a6e001a6b8aa7f856b68574374b613f27ade12d698df77",
          "FR-5": "801fef37008de5c99c72332f6e431770c156f856748c5663ceae8106f14ea978",
          "FR-6": "18c13c29dd41ad9bec055bc96b6b12d1377788312445a00427bdf38741d47d0a",
          "FR-7": "062bd07bbfdca08ea6a14442dd82174860fba40d01cd4b9521a150c0f296b958"
        },
        "Данные, права, ошибки": {
          "Данные, права, ошибки": "4a37dcb6543f3055de63d7270749b3770448e4f8a8ed7a615370bd365ea74a6a"
        },
        "Карта реализации": {
          "Карта реализации": "1e9001f9f866abf884e3fbd9e50d2907e16387248f236bbf7e89398a29312708"
        },
        "Критерии приёмки": {
          "AC-1": "ebcd2a44670ced6aee9b53c76860b308f78c8987d1bcd8d1556b2f8e08c6cc2b",
          "AC-2": "3d0947b286103ab19758bf24de265db4363a065348303fa316da34f55433618c",
          "AC-3": "06849a076e852f7892507db4e04e7c3533f2d2dca3691ce9b9b33cd3e302cc24",
          "AC-4": "00c43fdd48221cac2ad975b6943d1a5b57b475ec9377686fc2f1e5708e1840ba",
          "AC-5": "1cfcb2fe2f3e50e1ab965a33be1a6f9e8ff3a52b155b2012cea2aefdaf0a7135"
        },
        "Влияние на руководство": {
          "Влияние на руководство": "1f4f0155b9cdd6e2e059aac61778805fec5dca145bf5ca5d0fbe2b31deca537a"
        }
      }
    }
  },
  "write_scope_extensions": {
    "backend": [
      {
        "path": "packages/vendor/",
        "reason": "Карта реализации FR-1–FR-6: packages/vendor и .gitignore"
      },
      {
        "path": ".gitignore",
        "reason": "Карта реализации FR-1–FR-6: packages/vendor и .gitignore"
      }
    ]
  },
  "impl_fd_hash": {
    "backend": "90711e910ad3a8032889519d2f476f9984803c1c625f9b31055289dbafff962a"
  },
  "skipped_phases": {
    "documentation": "раздел ФД «Влияние на руководство»: не требуется: пользовательского поведения нет; команда `just vendor` попадёт в README в #46/#69.",
    "backend-tests": "ФД, «Карта реализации»: Новые тесты: нет (проверка — сам `check-mermaid.mjs` и проверка metafile, их выполняет `just vendor` в `just lint`; Rust-кода нет)."
  },
  "postcheck_snapshots": {
    "code-review": {
      "verdict": "APPROVED",
      "input_hash": "d7aec85bd5be2ed56ca3a17126a85c5971e1e2c3e35dea341b62899bc46586cb",
      "files": [
        "packages/vendor/package.json",
        "packages/vendor/pnpm-workspace.yaml",
        "packages/vendor/.npmrc",
        "packages/vendor/elk-stub.js",
        "packages/vendor/mermaid-entry.js",
        "packages/vendor/build-mermaid.mjs",
        "packages/vendor/check-mermaid.mjs",
        "packages/vendor/fetch.sh",
        "justfile",
        ".gitignore",
        ".github/workflows/ci.yml",
        "packages/vendor/pnpm-lock.yaml"
      ],
      "production_hash": "d7aec85bd5be2ed56ca3a17126a85c5971e1e2c3e35dea341b62899bc46586cb",
      "production_files": [
        "packages/vendor/package.json",
        "packages/vendor/pnpm-workspace.yaml",
        "packages/vendor/.npmrc",
        "packages/vendor/elk-stub.js",
        "packages/vendor/mermaid-entry.js",
        "packages/vendor/build-mermaid.mjs",
        "packages/vendor/check-mermaid.mjs",
        "packages/vendor/fetch.sh",
        "justfile",
        ".gitignore",
        ".github/workflows/ci.yml",
        "packages/vendor/pnpm-lock.yaml"
      ]
    },
    "security-code-review": {
      "verdict": "APPROVED",
      "input_hash": "d7aec85bd5be2ed56ca3a17126a85c5971e1e2c3e35dea341b62899bc46586cb",
      "files": [
        "packages/vendor/package.json",
        "packages/vendor/pnpm-workspace.yaml",
        "packages/vendor/.npmrc",
        "packages/vendor/elk-stub.js",
        "packages/vendor/mermaid-entry.js",
        "packages/vendor/build-mermaid.mjs",
        "packages/vendor/check-mermaid.mjs",
        "packages/vendor/fetch.sh",
        "justfile",
        ".gitignore",
        ".github/workflows/ci.yml",
        "packages/vendor/pnpm-lock.yaml"
      ],
      "production_hash": "d7aec85bd5be2ed56ca3a17126a85c5971e1e2c3e35dea341b62899bc46586cb",
      "production_files": [
        "packages/vendor/package.json",
        "packages/vendor/pnpm-workspace.yaml",
        "packages/vendor/.npmrc",
        "packages/vendor/elk-stub.js",
        "packages/vendor/mermaid-entry.js",
        "packages/vendor/build-mermaid.mjs",
        "packages/vendor/check-mermaid.mjs",
        "packages/vendor/fetch.sh",
        "justfile",
        ".gitignore",
        ".github/workflows/ci.yml",
        "packages/vendor/pnpm-lock.yaml"
      ]
    }
  },
  "accepted_code_hash": {
    "testing": "913605cbe04a51d13b2709f034bd668af4023c175c65037eda6a4b93dbd621ff"
  }
}
```
<!-- afd-state-snapshot:end -->
