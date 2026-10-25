# Тестовые сценарии: Завендорить KaTeX и Mermaid без elkjs

## Метаданные
- Issue: #42
- Стек: `just`, bash, `node` (`node:vm`), pnpm; сеть нужна для установки.

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001, SC-002 |
| AC-2 | SC-003, SC-004 |
| AC-3 | SC-005, SC-006 |
| AC-4 | SC-007 |
| AC-5 | SC-008, SC-009 |

## Сценарии
### SC-001: Чистая сборка
- **Покрывает:** AC-1
- **Уровень:** integration
- **Предусловия:** нет `packages/vendor/node_modules`, `dist/`.
- **Шаги:** `just vendor`; `ls packages/vendor/dist packages/vendor/dist/fonts`; `pnpm -C packages/vendor why elkjs`.
- **Ожидание:** код 0, есть `ok: elkjs не найден`; в `dist/` ровно пять позиций FR-4; `fonts/` непуст; `why` пуст.
- **Результат выполнения:** pass

### SC-002: Сборка не пачкает дерево
- **Покрывает:** AC-1
- **Уровень:** static
- **Предусловия:** после SC-001.
- **Шаги:** `git status --short`.
- **Ожидание:** вывод пуст.
- **Результат выполнения:** pass

### SC-003: Нет ELK, бандл меньше штатного
- **Покрывает:** AC-2
- **Уровень:** static
- **Предусловия:** `dist/` из SC-001.
- **Шаги:** `grep -c org.eclipse.elk` по нашему и штатному `mermaid.min.js`; `wc -c` обоих.
- **Ожидание:** 0 и >0; наш меньше.
- **Результат выполнения:** pass

### SC-004: API в `node:vm`
- **Покрывает:** AC-2
- **Уровень:** unit
- **Предусловия:** `dist/` из SC-001.
- **Шаги:** `vm.runInContext(<dist/mermaid.min.js>, vm.createContext({}))`.
- **Ожидание:** без исключения; `typeof ctx.mermaid.initialize` и `.run` — `function`.
- **Результат выполнения:** pass

### SC-005: Гейт сборки
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** копия `packages/vendor` в `tmp_dir`; убраны плагин заглушки и override.
- **Шаги:** `pnpm install`; `node build-mermaid.mjs`.
- **Ожидание:** код 1, stderr содержит `node_modules/elkjs/`.
- **Результат выполнения:** pass

### SC-006: Гейт проверки
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** копия собранного `packages/vendor`; в `dist/mermaid.min.js` дописано `org.eclipse.elk`.
- **Шаги:** `node check-mermaid.mjs`.
- **Ожидание:** код 1, сообщение называет `org.eclipse.elk`; git-дерево не изменено.
- **Результат выполнения:** pass

### SC-007: Проверки проекта
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** ветка задачи.
- **Шаги:** `just lint`; `just test`.
- **Ожидание:** оба код 0; `npm-licenses` по `packages/vendor` без нарушений; `actionlint`, `zizmor` зелёные.
- **Результат выполнения:** pass

### SC-008: Закреплённые версии
- **Покрывает:** AC-5
- **Уровень:** static
- **Предусловия:** ветка задачи.
- **Шаги:** прочитать `package.json`, `ci.yml`; найти в `pnpm-lock.yaml` записи пакетов `elkjs` (ключи `elkjs@…` в `packages`/`snapshots`).
- **Ожидание:** `katex` `0.19.1`, `mermaid` `12.2.0`, `esbuild` `0.28.2` без `^`/`~`; `pnpm@12.15.0`; пакетов `elkjs` в lockfile нет (только запись `overrides`).
- **Результат выполнения:** pass

### SC-009: Lockfile неизменен
- **Покрывает:** AC-5
- **Уровень:** integration
- **Предусловия:** ветка задачи.
- **Шаги:** `pnpm -C packages/vendor install --frozen-lockfile`; `git diff --exit-code packages/vendor/pnpm-lock.yaml`.
- **Ожидание:** оба код 0.
- **Результат выполнения:** pass
