# ADR-003: Quick Look — view-based превью с WKWebView

- **Статус:** принято
- **Дата:** 2026-11-06
- **Issue:** #48 (эпик #29; разблокирует #50)
- **Контекст спеки:** `docs/superpowers/specs/2026-10-19-nimmd-design.md`, §5.3, §10 этап 3

## Контекст

До реализации `NimMDQuickLook.appex` (#50) нужно снять риски §5.3: какой API расширения брать, исполняется ли JS (KaTeX, Mermaid), как отдавать картинки и тему, какие UTI заявлять, укладывается ли первый кадр в бюджет скорости (В2).

Хост должен показывать тот же HTML, что ядро отдаёт через `render` (#43): `Theme`, `AssetMode::Link` / `Inline`, формулы и `pre.mermaid`. Картинки читает `nimmd-assets` (#47) через будущий FFI (#49).

## Решение

**Выбран view-based preview:** `NSViewController` + `QLPreviewingController.preparePreviewOfFile(at:completionHandler:)` и `WKWebView`.

Контракт для #50:

| Тема | Решение |
| --- | --- |
| API | view-based, `preparePreviewOfFile`; completion — только после `didFinish` (и `callAsyncJavaScript` при math/mermaid), не сразу после `loadHTMLString` / `loadFileURL` |
| JS | включён в `WKWebViewConfiguration`; KaTeX/Mermaid из бандла appex через `AssetMode::Link` (не CDN); Mermaid — только по профилю |
| Картинки | `WKURLSchemeHandler` (`nimmd-asset://` или аналог) → `nimmd_read_asset`; `WKContentRuleList` против сети; entitlement read-only к каталогу документа |
| Сеть | `com.apple.security.network.client` **нужен** даже для пустой офлайн-страницы (К20): без него Web Content-процесс WKWebView в sandbox appex часто не стартует; сеть режется правилами контента, не отсутствием entitlement |
| Тема | `data-theme` из ядра: `Auto` опирается на `prefers-color-scheme` в CSS; при необходимости хост задаёт `Theme::Light`/`Dark` по `effectiveAppearance` до `render` |
| UTI | свои типы для Markdown (`.md`, `.markdown`, `.mdown`, `.mkd`) и данных (`.toml` — проверить `mdls`, иначе экспортировать свой UTI; `.yaml`/`.yml`, `.json`, `.csv`, `.txt`); **без** `public.plain-text` (В3, Р21) |
| Подпись | ad-hoc для v1 (`codesign -s -`): сначала appex, затем `.app` (#66); Developer ID — #87 |
| Ресурсы | `AssetMode::Link { base_url }`, не `Inline`: mermaid.min.js ≈ 3,6 МиБ |

## Замеры первого кадра / бюджета

Среда агента — Linux x86_64 (без Finder/`xcodebuild`). Замеры ядра здесь; wall-clock Finder подтверждается на macOS при #50. Цифры Apple Silicon — из закрытого #45 (`just bench-check` на arm64). Бюджет В2 (1 МиБ ≤ 100 мс на AS) — рендер ядра (#45); QL добавляет WebKit поверх.

### Рендер ядра, 1 МиБ (порог В2 = 100 мс)

| Вход | Apple Silicon (#45) | Linux x86_64 (этот спайк, release) |
| --- | ---: | ---: |
| prose | 35,5 мс | 47,6 мс |
| code | 32,6 мс | 40,4 мс |
| json | 19,8 мс | 22,1 мс |
| csv | 5,0 мс | 7,4 мс |

На Linux порог ×3 (300 мс) зелёный; на AS — абсолютный 100 мс зелёный.

### HTML и `AssetMode` (fixture math + mermaid, release, Linux)

| Режим | Медиана `render` | Размер `html` |
| --- | ---: | ---: |
| без ассетов | ~6 µs | ≈ 5 КиБ |
| `Link` (`nimmd-asset://bundle`) | ~4 µs | ≈ 6 КиБ |
| `Inline` с реальным vendor | ~4,2 мс | ≈ 3,9 МиБ |

Размеры vendor: `katex.min.css` ≈ 25 КиБ, `katex.min.js` ≈ 267 КиБ, `mermaid.min.js` ≈ 3,6 МиБ.

Вывод: **Link + view-based**. Inline раздувает ответ до размера Mermaid и ухудшает передачу в WebKit и cold parse JS.

## Последствия

- #50: Xcode-таргет `apps/macos-quicklook/`, view-based, entitlements sandbox + `network.client` + read-only к каталогу файла, схема картинок, `QLSupportedContentTypes` без `public.plain-text`.
- #49: FFI достаточен (`nimmd_render_html`, `nimmd_read_asset`); отдельный «data-based» путь не нужен.
- Ручной прогон Finder (пробел на `.md` / `.toml` / `.mdown`, светлая/тёмная тема, Mermaid, локальная картинка) — в чек-листе #50 / #68; при расхождении с замерами уточнить ADR приложением, не меняя выбранный вариант без новой записи.

## Отклонённые альтернативы

1. **Только data-based HTML** (`QLPreviewProvider` / `QLPreviewReply`) — Finder не гарантирует JS; без него нет KaTeX/Mermaid, статический HTML не даёт паритет с `--view`.
2. **view-based + `AssetMode::Inline`** — работает, но ~4 МиБ HTML на любой документ с Mermaid; хуже первый кадр.
3. **Без `network.client`** — WKWebView в sandbox appex часто мёртв (К20).
4. **Заявить `public.plain-text`** — перехват чужих текстовых файлов (В3, Р21).
