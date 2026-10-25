# Функциональный дизайн: Завендорить KaTeX и Mermaid без elkjs

## Метаданные [@for: all]
- Issue: #42
- Size: S
- Владелец: Системный аналитик

## Цель [@for: all]
- Положить в репозиторий воспроизводимую сборку ресурсов для хостов ядра: KaTeX и mermaid.js из npm, без CDN, с mermaid, собранным без `elkjs` (EPL-2.0 исключён, спека 12, Р29). Сборка падает, если `elkjs` попал в бандл или в установленные зависимости.
- Источники: спека `docs/superpowers/specs/2026-10-19-nimmd-design.md` разделы 5.1 (`AssetMode`), 7, 12; план ядра `docs/superpowers/plans/2026-10-19-nimmd-core.md` Task 6; решения шага 1 — `decision-log.md`.

## Границы [@for: all]
- Входит: самостоятельный pnpm-пакет `packages/vendor` (свой lockfile, без корневого workspace), сборка `packages/vendor/dist/`, рецепт `just vendor`, `vendor` в `just lint`, установка pnpm в job `lint` CI, правило `.gitignore` для `dist/`.
- Не входит (decision-log п. 1, 4): замена `layout: elk*` → dagre — инлайн-скрипт ядра (#43) и фронтенд `--view` (#53); фикстуры `features/mermaid.md`, `features/mermaid_elk.md` и ручная проверка рендера (шаг 5 Task 6) — #43; подключение `dist/` к хостам (#48/#50/#53/#54); `contrib/` KaTeX; `THIRD_PARTY_LICENSES.html` (#67); job `test` CI.

## Целевое поведение [@for: all]
- Разработчик или CI выполняет `just vendor` и получает в `packages/vendor/dist/` ровно пять позиций с именами, которые ждёт `AssetMode::Link { base_url }` (Task 4): `katex.min.css`, `katex.min.js`, `fonts/`, `mermaid.min.js` (своя сборка, глобальный `mermaid`, API штатный) и `mermaid.min.js.LEGAL.txt` (лицензионные комментарии для страницы «Лицензии»). Для `AssetMode::Inline` хост читает те же файлы. `just lint` выполняет `vendor` и падает, если `elkjs` вернулся в зависимости или в бандл.

## Требования [@for: all]
- **FR-1:** `packages/vendor/package.json`: `"private": true`, без `dependencies`; `devDependencies` закреплены точно: `katex` `0.19.1`, `mermaid` `12.2.0`, `esbuild` `0.28.2`; `"type": "module"`. Рядом `pnpm-lock.yaml` (коммитится), `pnpm-workspace.yaml` с `overrides: { elkjs: "-" }` (pnpm 12 читает override отсюда, а не из `package.json`) и `allowBuilds: { esbuild: false }` (иначе pnpm 12 падает на `ERR_PNPM_IGNORED_BUILDS`), а также `.npmrc` с `ignore-scripts=true`. После установки `pnpm -C packages/vendor why elkjs` ничего не находит, в `pnpm-lock.yaml` нет пакета `elkjs` (допустима только запись секции `overrides`).
- **FR-2:** `elk-stub.js` — `export default class ELK { layout() { return Promise.reject(new Error("ELK layout is not available in NimMD")); } }` с комментарием: причина (EPL-2.0, спека 12) и контракт для хостов — бандл без ELK, хост переписывает `layout: elk*` → dagre до вызова mermaid (#43, #53). `mermaid-entry.js` — `import mermaid from "mermaid"; export default mermaid;`.
- **FR-3:** `build-mermaid.mjs` собирает esbuild из `mermaid-entry.js`: `bundle`, `minify`, `format: "iife"`, `globalName: "mermaid"`, `footer` `mermaid=mermaid.default;`, `outfile: dist/mermaid.min.js`, `metafile: true`, `legalComments: "external"`, плагин `onResolve` с фильтром `/^elkjs(\/|$)/` → абсолютный путь `elk-stub.js`. Пути считаются от каталога скрипта, не от текущего. После сборки: если среди ключей `metafile.inputs` есть путь, содержащий `node_modules/elkjs/`, скрипт печатает в stderr эти пути и завершается с кодом 1.
- **FR-4:** `check-mermaid.mjs` (пути от каталога скрипта) завершается с кодом 1 и сообщением о первой нарушенной проверке, если: в `dist/mermaid.min.js` есть строка `org.eclipse.elk`; его размер не меньше `node_modules/mermaid/dist/mermaid.min.js`; существует `node_modules/.pnpm/elkjs@*`; содержимое `dist/` не равно ровно `fonts`, `katex.min.css`, `katex.min.js`, `mermaid.min.js`, `mermaid.min.js.LEGAL.txt`; `dist/fonts/` пуст. Иначе печатает `ok: elkjs не найден` и завершается с кодом 0.
- **FR-5:** `fetch.sh` (bash, `set -euo pipefail`, работает из любого текущего каталога): `pnpm install --frozen-lockfile` в `packages/vendor`, пересоздание `dist/`, копирование `node_modules/katex/dist/{katex.min.css,katex.min.js,fonts}` в `dist/`, затем `node build-mermaid.mjs` и `node check-mermaid.mjs`. Любой шаг с ошибкой роняет скрипт.
- **FR-6:** `justfile`: рецепт `vendor` запускает `bash packages/vendor/fetch.sh` (комментарий в стиле соседних рецептов); `lint` содержит `vendor` перед `npm-licenses` (лицензии проверяются по установленным зависимостям). `.gitignore`: `packages/vendor/dist/`.
- **FR-7:** `.github/workflows/ci.yml`, job `lint`: в шаге установки инструментов ставится pnpm точной версии `npm install -g pnpm@12.15.0` (новых actions нет; находка zizmor `adhoc-packages` подавлена комментарием с причиной). `just lint` (включая `actionlint` и `zizmor`) и `just test` зелёные локально.

## Данные, права, ошибки [@for: backend, security, qa]
- Вход — пакеты npm-реестра по `pnpm-lock.yaml` (целостность проверяет pnpm по `integrity`); `--frozen-lockfile` не даёт молча обновить версии. Скрипты установки пакетов не выполняются (`ignore-scripts=true`); esbuild берёт бинарник из платформенного optional-пакета.
- Выход — файлы в `packages/vendor/dist/` (вне git) и коды возврата скриптов. Секретов, прав и сетевых вызовов, кроме установки пакетов, нет.
- Ошибки: `elkjs` во входах сборки (FR-3), признаки ELK, размер, лишний или недостающий файл в `dist/` (FR-4), рассинхрон `package.json` и lockfile, сбой сети при установке (FR-5) — всё ненулевой код и сообщение в stderr.
- Код `packages/vendor` исключён из DOM-правил `.semgrep.yml` (`/packages/vendor/**`); своего DOM-кода там нет.

## Карта реализации [@for: backend, frontend, qa]
- Backend (infra): `packages/vendor/package.json`, `packages/vendor/pnpm-lock.yaml`, `packages/vendor/pnpm-workspace.yaml`, `packages/vendor/.npmrc`, `packages/vendor/elk-stub.js`, `packages/vendor/mermaid-entry.js`, `packages/vendor/build-mermaid.mjs`, `packages/vendor/check-mermaid.mjs`, `packages/vendor/fetch.sh` (FR-1–FR-5); `justfile`, `.gitignore` (FR-6); `.github/workflows/ci.yml` (FR-7).
- Frontend: нет.
- Новые тесты: нет (проверка — сам `check-mermaid.mjs` и проверка metafile, их выполняет `just vendor` в `just lint`; Rust-кода нет).

## Критерии приёмки [@for: all]
### AC-1: Сборка ресурсов
- **Дано:** чистое дерево, `packages/vendor/node_modules` и `dist/` отсутствуют.
- **Действие:** `just vendor`, затем `ls packages/vendor/dist packages/vendor/dist/fonts` и `pnpm -C packages/vendor why elkjs`.
- **Ожидание:** код 0, вывод содержит `ok: elkjs не найден`; в `dist/` ровно пять позиций FR-4, `fonts/` непустой; `why elkjs` ничего не находит; `git status --short` после сборки пуст.

### AC-2: В бандле нет elkjs, API штатный
- **Дано:** `dist/` из AC-1.
- **Действие:** `grep -c org.eclipse.elk` по `dist/mermaid.min.js` и по `node_modules/mermaid/dist/mermaid.min.js`; сравнить размеры; выполнить `dist/mermaid.min.js` в пустом контексте `node:vm`.
- **Ожидание:** в нашем бандле 0 совпадений, в штатном больше 0; наш меньше штатного; в контексте появляется объект `mermaid` с функциями `initialize` и `run`.

### AC-3: Гейт сборки ловит elkjs
- **Дано:** копия `packages/vendor` во временном каталоге (git-дерево не меняется): в одной копии убран плагин заглушки из `build-mermaid.mjs` и override из `pnpm-workspace.yaml`, в другой в `dist/mermaid.min.js` дописана строка `org.eclipse.elk`.
- **Действие:** в первой копии `pnpm install` и `node build-mermaid.mjs`; во второй `node check-mermaid.mjs`.
- **Ожидание:** оба скрипта завершаются с кодом 1; первый называет путь `node_modules/elkjs/`, второй — найденную строку ELK.

### AC-4: Проверки проекта
- **Дано:** ветка с изменениями задачи.
- **Действие:** `just lint` и `just test`.
- **Ожидание:** оба зелёные; `npm-licenses` проверил `packages/vendor` без нарушений (EPL-2.0 нет — `elkjs` не установлен); `actionlint` и `zizmor` проходят по изменённому `ci.yml`.

### AC-5: Закреплённые версии
- **Дано:** `packages/vendor/package.json`, `pnpm-lock.yaml`, `ci.yml`.
- **Действие:** прочитать версии; `pnpm -C packages/vendor install --frozen-lockfile`.
- **Ожидание:** `katex` `0.19.1`, `mermaid` `12.2.0`, `esbuild` `0.28.2` без диапазонов; pnpm в CI — `12.15.0`; установка проходит без изменения lockfile.

## Влияние на руководство [@for: docs]
- не требуется: пользовательского поведения нет; команда `just vendor` попадёт в README в #46/#69.
