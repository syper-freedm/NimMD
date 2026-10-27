# nimmd-core

Ядро NimMD: Markdown и файлы данных (TOML, YAML, JSON, CSV, TXT) в готовый HTML-документ.
Одно ядро на два хоста — `NimMD.app` (Tauri, в том же процессе) и Quick Look-расширение
(через C ABI `nimmd-ffi`), поэтому документ выглядит одинаково везде.

## Пример

```rust
use nimmd_core::text::decode_lossy;
use nimmd_core::{AssetMode, FileKind, RenderOptions, render};

// Байты файла читает хост; ядро получает только строку.
let bytes = "# Заметка\n\nФормула $E = mc^2$.\n".as_bytes();
let (src, not_utf8) = decode_lossy(bytes);

let opts = RenderOptions {
    kind: FileKind::from_extension("md").unwrap_or_default(),
    assets: AssetMode::Link { base_url: "/vendor".into() },
    not_utf8,
    ..RenderOptions::default()
};
let out = render(&src, &opts);

assert_eq!(out.title.as_deref(), Some("Заметка"));
assert!(out.profile.features.math);
assert!(out.html.contains("/vendor/katex.min.js"));
assert!(out.html.contains(&out.body));
```

## API

- `render(src, &RenderOptions) -> Rendered` — полный документ (`html`) и его `<main>` (`body`)
  для первого кадра редактора, плюс профиль, frontmatter, стиль переносов, наличие BOM,
  заголовок и признаки обрезки (`truncated`, `highlight_limited`).
- `detect(src) -> Profile` — диалект и расширения без рендера.
- `render_fragment(body) -> String` — HTML тела Markdown без frontmatter и обвязки.
- `highlight(lang, code) -> Vec<Span>` — спаны подсветки для редактора, смещения в UTF-16.
- `FileKind::from_extension(ext)` — тип файла по расширению; неизвестное — `None`.
- `text::decode_lossy(bytes)` — байты в строку с заменой битых последовательностей на `U+FFFD`.
- Модули `text` (BOM, переносы, frontmatter, лимиты), `csv`, `profile` — для хостов и тестов.

## Опции `RenderOptions`

Все поля публичные. Хосту без JS (`assets: None`) нужен `math: MathML`: разметку для KaTeX
без скрипта никто не отрисует.

| Поле | По умолчанию | Смысл |
|------|--------------|-------|
| `kind` | `Markdown` | Тип файла; не `Markdown` — данные без frontmatter, профиля и скриптов |
| `theme` | `Auto` | `Auto` по `prefers-color-scheme`, `Light`, `Dark` |
| `assets` | `None` | KaTeX и Mermaid: `None` — без скриптов, `Link { base_url }` — `<script src>` того же origin, `Inline` — содержимое в документе |
| `math` | `Katex` | `Katex` — разметка для KaTeX, `MathML` — без скриптов, `Source` — исходный TeX |
| `remote_images` | `false` | `false` — `http(s)`-картинки заменяются плейсхолдером |
| `not_utf8` | `false` | Хост заменял битые байты — показать баннер |
| `max_bytes` | 10 МБ | Длиннее — рендерится префикс по границе символа, `truncated = true` |
| `profile` | `None` | Переопределить детектор; `None` — профиль определяет `detect` |

Файлы бандлов для `assets` собираются в `packages/vendor/dist` (`just vendor`).

## Гарантии

- Нет ввода-вывода: ни `std::fs`, ни сети, ни процессов — строка на входе, результат на выходе.
  Картинки документа хост отдаёт сам (крейт `nimmd-assets`).
- Нет паники: функции тотальны на любой строке, `unsafe` запрещён (`#![forbid(unsafe_code)]`).
- Документ недоверенный: сырой HTML выводится текстом, опасные адреса ссылок и картинок
  отбрасываются, формулы и диаграммы рисуются только в безопасных режимах KaTeX и Mermaid.
- HTML не зависит от профиля: от него зависят только подключённые скрипты, поэтому ошибка
  детектора не портит отображение. Ключ `markdown:` во frontmatter побеждает эвристику.
- BOM отрезается, переносы `\r\n` нормализуются; исходный стиль возвращается в `line_ending`,
  чтобы хост сохранил файл как был.
- Рендер файла 1 МБ — не дольше 100 мс на Apple Silicon (`just bench-check`).

## Правила для хостов

- Определять `kind` по расширению через `FileKind::from_extension`, байты декодировать
  `text::decode_lossy` и передавать признак замен в `not_utf8`.
- Не ослаблять CSP `WebView` и не давать ему плагины `fs/shell/process/http`: ядро рассчитывает,
  что разметку выполняет изолированная страница.
- В Quick Look и режиме просмотра `remote_images` всегда `false`.
- `base_url` в `AssetMode::Link` — того же origin, что страница: в Quick Look своя схема
  только для файлов бандла, в Tauri — `/vendor/` фронтенда.
