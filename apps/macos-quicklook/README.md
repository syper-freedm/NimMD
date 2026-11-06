# NimMDQuickLook.appex

View-based Quick Look (ADR-003 / #50): `NSViewController` + `WKWebView`, ядро через `libnimmd_ffi.a`.

## Сборка (macOS)

```bash
just vendor
apps/macos-quicklook/Scripts/sync-vendor.sh
apps/macos-quicklook/Scripts/build-ffi.sh
cd apps/macos-quicklook && xcodegen generate
xcodebuild -project NimMDQuickLook.xcodeproj -scheme NimMD -configuration Release build
codesign -s - --force --deep build/Release/NimMD.app   # пути Xcode могут отличаться
```

Контейнер `NimMD.app` нужен, чтобы система зарегистрировала appex. UI приложения — вне #50.

## Контракт

| Тема | Реализация |
| --- | --- |
| Рендер | `nimmd_render_html`, `max_bytes` = 10 МиБ, `AssetMode::Link` → `nimmd-asset://bundle` |
| Картинки | `nimmd-asset://doc/…` → `nimmd_read_asset`; SVG только как `image/svg+xml` |
| Сеть | entitlement `network.client` + `WKContentRuleList` (block http/https/ws/wss) |
| Тема | `effectiveAppearance` → Light/Dark до render |
| UTI | свои + markdown/json/csv; **без** `public.plain-text` |

Проверка без Xcode: `apps/macos-quicklook/Scripts/verify.sh`.
