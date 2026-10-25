//! Полный HTML-документ: тема, ресурсы по профилю, баннеры.

use serde::{Deserialize, Serialize};

use crate::data::{FileKind, push_data};
use crate::detect::profile_of;
use crate::profile::Profile;
use crate::render::{MathMode, highlight_yaml, push_escaped, render_body};
use crate::text::{
    LineEnding, MAX_BYTES, detect_line_ending, normalize_newlines, split_frontmatter, strip_bom,
    truncate_at_char_boundary,
};

const THEME_CSS: &str = include_str!("../../../packages/theme/nimmd.css");
const HIGHLIGHT_CSS: &str = include_str!("../../../packages/theme/highlight.css");
const DATA_CSS: &str = include_str!("../../../packages/theme/data.css");
const KATEX_INIT: &str = include_str!("../../../packages/theme/katex-init.js");
const MERMAID_INIT: &str = include_str!("../../../packages/theme/mermaid-init.js");

const MIB: usize = 1024 * 1024;

/// Тема оформления; `Auto` следует `prefers-color-scheme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Auto,
    Light,
    Dark,
}

/// Подключение KaTeX и Mermaid в `<head>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AssetMode {
    /// Без `<script>` и `<link>`.
    #[default]
    None,
    /// Файлы `katex.min.css`, `katex.min.js`, `mermaid.min.js` по адресу `base_url`.
    Link { base_url: String },
    /// Содержимое бандлов прямо в документе.
    Inline {
        katex_css: String,
        katex_js: String,
        mermaid_js: String,
    },
}

/// Опции [`render`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOptions {
    pub theme: Theme,
    pub assets: AssetMode,
    pub math: MathMode,
    /// `false` — удалённые картинки заменяются плейсхолдером `remote-image`.
    pub remote_images: bool,
    /// Хост заменил нераспознанные байты на `U+FFFD`: показать баннер.
    pub not_utf8: bool,
    pub max_bytes: usize,
    /// `None` — профиль определяет детектор.
    pub profile: Option<Profile>,
    /// Не `Markdown` — файл данных: без frontmatter, профиля и скриптов.
    pub kind: FileKind,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            theme: Theme::Auto,
            assets: AssetMode::None,
            math: MathMode::Katex,
            remote_images: false,
            not_utf8: false,
            max_bytes: MAX_BYTES,
            profile: None,
            kind: FileKind::Markdown,
        }
    }
}

/// Результат [`render`].
#[allow(clippy::struct_excessive_bools)] // независимые признаки исходника
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Rendered {
    /// Документ целиком.
    pub html: String,
    /// `<main class="nimmd">…</main>` (у данных — `nimmd nimmd-data`) — та же подстрока, что в `html`.
    pub body: String,
    pub profile: Profile,
    /// Frontmatter с разделителями, переносы `\n`.
    pub frontmatter: Option<String>,
    pub line_ending: LineEnding,
    pub had_bom: bool,
    pub truncated: bool,
    /// Текст первого заголовка уровня 1.
    pub title: Option<String>,
    /// Файл данных длиннее `HIGHLIGHT_LIMIT`: подсвечена только голова.
    pub highlight_limited: bool,
}

/// Полный HTML-документ из исходного текста.
///
/// Тотальна: на любой строке возвращает документ без паники; ФС, сеть и процессы не трогает.
pub fn render(src: &str, opts: &RenderOptions) -> Rendered {
    let (had_bom, src) = strip_bom(src);
    let line_ending = detect_line_ending(src);
    let src = normalize_newlines(src);
    let (src, truncated) = truncate_at_char_boundary(&src, opts.max_bytes);
    let markdown = opts.kind == FileKind::Markdown;

    let mut body = String::from("<main class=\"nimmd");
    if !markdown {
        body.push_str(" nimmd-data\" data-kind=\"");
        body.push_str(opts.kind.name());
    }
    body.push_str("\">");
    if opts.not_utf8 {
        body.push_str(
            "<div class=\"banner not-utf8\">Файл не в UTF-8: нераспознанные байты заменены знаком \u{FFFD}</div>",
        );
    }
    let (frontmatter, profile, title, highlight_limited) = if markdown {
        let (frontmatter, text) = split_frontmatter(src);
        let parsed = render_body(text, opts.math, opts.remote_images);
        let profile = opts
            .profile
            .unwrap_or_else(|| profile_of(frontmatter, text, parsed.math, parsed.mermaid));
        if let Some(fm) = frontmatter {
            body.push_str("<pre class=\"frontmatter\">");
            highlight_yaml(&mut body, yaml_of(fm));
            body.push_str("</pre>");
        }
        body.push_str(&parsed.html);
        (frontmatter, profile, parsed.title, false)
    } else {
        let limited = push_data(&mut body, opts.kind, src);
        (None, Profile::default(), None, limited)
    };
    if truncated {
        let shown = if opts.max_bytes < MIB {
            format!("{} байт", opts.max_bytes)
        } else {
            format!("{} МБ", opts.max_bytes / MIB)
        };
        body.push_str("<div class=\"banner truncated\">Файл обрезан: показаны первые ");
        body.push_str(&shown);
        body.push_str("</div>");
    }
    body.push_str("</main>");

    let theme = match opts.theme {
        Theme::Auto => "auto",
        Theme::Light => "light",
        Theme::Dark => "dark",
    };
    let mut html = format!(
        "<!doctype html><html data-theme=\"{theme}\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
    );
    if let Some(title) = &title {
        html.push_str("<title>");
        push_escaped(&mut html, title);
        html.push_str("</title>");
    }
    html.push_str("<style>");
    html.push_str(THEME_CSS);
    html.push_str(HIGHLIGHT_CSS);
    if !markdown {
        html.push_str(DATA_CSS);
    }
    html.push_str("</style>");
    if markdown {
        let katex = profile.features.math && opts.math == MathMode::Katex;
        push_assets(&mut html, &opts.assets, katex, profile.features.mermaid);
    }
    html.push_str("</head><body>");
    html.push_str(&body);
    html.push_str("</body></html>");

    Rendered {
        html,
        body,
        profile,
        frontmatter: frontmatter.map(str::to_owned),
        line_ending,
        had_bom,
        truncated,
        title,
        highlight_limited,
    }
}

/// YAML между строками `---`.
fn yaml_of(frontmatter: &str) -> &str {
    let inner = frontmatter.strip_prefix("---\n").unwrap_or(frontmatter);
    inner
        .strip_suffix("---\n")
        .or_else(|| inner.strip_suffix("---"))
        .unwrap_or(inner)
}

fn push_assets(html: &mut String, assets: &AssetMode, katex: bool, mermaid: bool) {
    match assets {
        AssetMode::None => {}
        AssetMode::Link { base_url } => {
            let mut base = String::new();
            push_escaped(&mut base, base_url);
            if katex {
                for part in [
                    "<link rel=\"stylesheet\" href=\"",
                    &base,
                    "/katex.min.css\"><script defer src=\"",
                    &base,
                    "/katex.min.js\"></script>",
                ] {
                    html.push_str(part);
                }
                push_inline_script(html, KATEX_INIT);
            }
            if mermaid {
                for part in ["<script src=\"", &base, "/mermaid.min.js\"></script>"] {
                    html.push_str(part);
                }
                push_inline_script(html, MERMAID_INIT);
            }
        }
        AssetMode::Inline {
            katex_css,
            katex_js,
            mermaid_js,
        } => {
            if katex {
                html.push_str("<style>");
                html.push_str(&close_safe(katex_css));
                html.push_str("</style>");
                push_inline_script(html, katex_js);
                push_inline_script(html, KATEX_INIT);
            }
            if mermaid {
                push_inline_script(html, mermaid_js);
                push_inline_script(html, MERMAID_INIT);
            }
        }
    }
}

fn push_inline_script(html: &mut String, code: &str) {
    html.push_str("<script>");
    html.push_str(&close_safe(code));
    html.push_str("</script>");
}

/// `</` внутри `<script>`/`<style>` закрыл бы тег раньше времени.
fn close_safe(code: &str) -> String {
    code.replace("</", "<\\/")
}
