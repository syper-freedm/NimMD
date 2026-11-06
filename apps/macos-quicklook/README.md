# NimMDQuickLook.appex

View-based Quick Look (ADR-003 / #50): `NSViewController` + `WKWebView`, ядро через `libnimmd_ffi.a`.

## Сборка (macOS)

```bash
just ql-prep          # vendor + libnimmd_ffi.a + xcodegen
xcodebuild -project apps/macos-quicklook/NimMDQuickLook.xcodeproj -scheme NimMD -configuration Release build
codesign -s - --force --deep …/NimMD.app
```

`project.yml` также гоняет `build-ffi.sh` / `sync-vendor.sh` как pre-build. Контейнер `NimMD.app` регистрирует appex; UI приложения — вне #50.

## Контракт

| Тема | Реализация |
| --- | --- |
| Рендер | `nimmd_render_html`, чтение файла ≤ 10 МиБ, `AssetMode::Link` → `nimmd-asset://bundle` |
| Картинки | `nimmd-asset://doc/…` → `nimmd_read_asset`; SVG только `image/svg+xml`; навигация после первого HTML отменена |
| Сеть | `network.client` + `WKContentRuleList`; ошибка компиляции правил → fail closed |
| Тема | `effectiveAppearance` → Light/Dark до render |
| UTI | свои + markdown/json/csv; **без** `public.plain-text` |

### Entitlement `/`

`temporary-exception.files.absolute-path.read-only = ["/"]` — намеренно для v1: каталог документа неизвестен на этапе подписи, а `nimmd_read_asset` читает соседние картинки в любом пути Finder. Узкий путь в entitlements невыразим; сужение — follow-up, когда появится другой IPC.

### `.txt` UTI

`org.nimmd.plain-text` экспортируется только под `.txt` и не conforms to `public.plain-text`. Установка контейнера может повлиять на Launch Services для `.txt` — проверить на macOS перед широкой раздачей.

Проверка без Xcode: `just ql-verify`.
