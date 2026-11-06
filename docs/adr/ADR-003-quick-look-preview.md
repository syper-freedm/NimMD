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

Data-based (`QLPreviewProvider` / `QLPreviewReply` с HTML) **не выбран** для MVP: Finder не гарантирует исполнение JavaScript в ответе, а KaTeX и Mermaid без JS не работают. Статический HTML без скриптов покрыл бы только «голый» Markdown и файлы данных — этого мало для паритета с `--view`.

Контракт для #50:

| Тема | Решение |
| --- | --- |
| API | view-based, `preparePreviewOfFile`; completion — только после готовности кадра (см. ниже) |
| JS | включён в `WKWebViewConfiguration`; KaTeX/Mermaid из бандла appex через `AssetMode::Link` |
| Картинки | свой `WKURLSchemeHandler` (`nimmd-asset://` или аналог) → `nimmd_read_asset`; entitlement на чтение каталога документа (`temporary-exception…read-only` / read-access к URL файла) |
| Сеть | `com.apple.security.network.client` **нужен** даже для пустой офлайн-страницы (К20): без него Web Content-процесс WKWebView в sandbox appex часто не стартует |
| Тема | `data-theme` из ядра: `Auto` опирается на `prefers-color-scheme` в CSS; при необходимости хост задаёт `Theme::Light`/`Dark` по `effectiveAppearance` до `render` |
| UTI | свои типы для Markdown (`.md`, `.markdown`, `.mdown`, `.mkd`) и данных (`.toml` — проверить `mdls`, `.yaml`/`.yml`, `.json`, `.csv`, `.txt`); **без** `public.plain-text` (В3, Р21) |
| Подпись | ad-hoc для appex в v1 (как ADR-002 для приложения); Developer ID — #87 |
| Ресурсы | `AssetMode::Link { base_url }`, не `Inline`: mermaid.min.js ≈ 3,6 МиБ |

## Разбор плана спайка

### 1. view-based vs data-based

- **view-based:** свой `WKWebView`, полный контроль над JS, схемой картинок и моментом «превью готово».
- **data-based:** проще для статического HTML/PDF/изображения; для Mermaid/KaTeX в Finder ненадёжен (сообщество: Writer, MDViewer, отчёты о «пустом» превью).

Итог: view-based. Data-based можно вернуть позже только для файлов данных без JS — вне MVP.

### 2. JS из бандла и бюджет скорости (В2)

Ядро уже подключает KaTeX/Mermaid по профилю (#43) и вендорит их без сети (#42). В appex скрипты грузятся с `baseURL` бандла / custom scheme, не с CDN.

**Первый кадр:** не вызывать completion сразу после `loadHTMLString` / `loadFileURL` — иначе Quick Look снимает пустой `WKWebView`. Ждать `WKNavigationDelegate.didFinish`; при `features.math` / `features.mermaid` — дополнительно завершения init-скриптов (например `callAsyncJavaScript` после `DOMContentLoaded`).

Бюджет В2 (1 МиБ ≤ 100 мс на Apple Silicon) относится к **рендеру ядра**; его уже держит #45. Слой QL добавляет загрузку WebKit и разбор скриптов — поэтому Mermaid подключаем только когда профиль его запросил, а не в каждый превью.

### 3. Картинки

Пути проверяет `nimmd_read_asset` (#47). В WKWebView — `WKURLSchemeHandler` + `WKContentRuleList` против сети (как в #50). `http(s)` в документе ядро уже заменяет плейсхолдером при `remote_images = false`.

### 4. `network.client` (К20)

Нужен. Эмпирика sandbox Quick Look: без `com.apple.security.network.client` процесс Web Content не поднимается даже для `about:blank` / полностью офлайн HTML. Entitlement не отменяет CSP и запрет навигации: сеть режется правилами контента, а не отсутствием entitlement.

### 5. Тёмная тема

`packages/theme/nimmd.css` уже поддерживает `[data-theme="auto"]` + `prefers-color-scheme` и принудительные `light`/`dark`. В QL достаточно `Theme::Auto` либо явного значения с `NSApp.effectiveAppearance` / appearance превью-контроллера.

### 6. UTI

- Заявлять только типы, которые реально рендерим (`FileKind::from_extension`: `md|markdown|mdown|mkd`, `toml`, `yaml|yml`, `json`, `csv`, `txt`).
- TOML: системный UTI через `mdls` на целевой macOS; при отсутствии — экспортировать свой (`public.toml` / `org.nimmd.toml`) и привязать `.toml`.
- `.mdown` / `.mkd` — в экспортируемом Markdown UTI (conforms to `net.daringfireball.markdown` / аналог), не через `public.plain-text`.
- Не заявлять `public.plain-text`, чтобы не перехватывать чужие `.txt` без необходимости политики продукта (В3, Р21).

### 7. Ad-hoc подпись appex

Для локальной установки и CI v1 — ad-hoc (`codesign -s -`), сначала appex, затем `.app` (см. #66). Нотаризация — отдельно (#87).

## Замеры первого кадра / бюджета

Среда агента — Linux x86_64 (без Finder/`xcodebuild`). Замеры ядра здесь; wall-clock Finder подтверждается на macOS при #50. Цифры Apple Silicon — из закрытого #45 (`just bench-check` на arm64).

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

Вывод по первому кадру: **Link + view-based**. Inline раздувает ответ до размера Mermaid и ухудшает и передачу в WebKit, и cold parse JS. Data-based с Inline ещё хуже (весь HTML одним blob в `QLPreviewReply`).

Оценка состава первого кадра QL (порядок, не wall-clock Finder):

1. Чтение файла ≤ 10 МиБ + `nimmd_render_html` — в бюджете В2 на AS для 1 МиБ.
2. `WKWebView` navigation + CSS темы — миллисекунды…десятки мс после старта Web Content.
3. KaTeX (если есть формулы) — сотни КиБ JS.
4. Mermaid (если есть диаграммы) — доминирующая стоимость post-HTML; грузить лениво по профилю.

## Последствия

- #50: Xcode-таргет `apps/macos-quicklook/`, view-based, entitlements sandbox + `network.client` + read-only к каталогу файла, схема картинок, `QLSupportedContentTypes` без `public.plain-text`.
- #49: FFI достаточен (`nimmd_render_html`, `nimmd_read_asset`); отдельный «data-based» путь не нужен.
- #75 (кнопка копирования кода в QL): только если JS доступен — по этому ADR доступен.
- Ручной прогон Finder (пробел на `.md` / `.toml` / `.mdown`, светлая/тёмная тема, Mermaid, локальная картинка) — в чек-листе #50 / #68; при расхождении с оценкой первого кадра уточнить ADR приложением, не меняя выбранный вариант без новой записи.

## Отклонённые альтернативы

1. **Только data-based HTML** — нет надёжного KaTeX/Mermaid в Finder.
2. **view-based + `AssetMode::Inline`** — работает, но ~4 МиБ HTML на любой документ с Mermaid; хуже первый кадр.
3. **Без `network.client`** — WKWebView в sandbox appex часто мёртв (К20).
4. **Заявить `public.plain-text`** — перехват чужих текстовых файлов (В3, Р21).
