//! Файлы данных: тип файла по расширению и тело `<pre class="data">`.

use serde::{Deserialize, Serialize};

use crate::csv::push_colored;
use crate::highlight::{CSV_COLOR_LIMIT, HIGHLIGHT_LIMIT, split_head, write_highlighted};
use crate::render::push_escaped;

/// Тип исходного файла; всё, кроме `Markdown`, показывается как данные.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    /// Документ Markdown.
    #[default]
    Markdown,
    /// TOML с подсветкой.
    Toml,
    /// YAML с подсветкой.
    Yaml,
    /// JSON с подсветкой.
    Json,
    /// CSV с раскраской столбцов.
    Csv,
    /// Простой текст без подсветки.
    Text,
}

impl FileKind {
    /// Тип по расширению без точки, без учёта регистра ASCII; неизвестное — `None`.
    pub fn from_extension(ext: &str) -> Option<FileKind> {
        Some(match ext.to_ascii_lowercase().as_str() {
            "md" | "markdown" | "mdown" | "mkd" => Self::Markdown,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "json" => Self::Json,
            "csv" => Self::Csv,
            "txt" => Self::Text,
            _ => return None,
        })
    }

    /// Значение `data-kind` и токен грамматики.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Text => "text",
        }
    }
}

/// `<pre class="data">` файла данных и баннер лимита подсветки; `true` — подсвечена только голова.
pub(crate) fn push_data(out: &mut String, kind: FileKind, text: &str) -> bool {
    out.push_str("<pre class=\"data\">");
    let limit = if kind == FileKind::Csv {
        CSV_COLOR_LIMIT
    } else {
        HIGHLIGHT_LIMIT
    };
    let (head, tail) = match kind {
        FileKind::Text | FileKind::Markdown => ("", text),
        _ => split_head(text, limit),
    };
    if kind == FileKind::Csv {
        push_colored(out, head);
    } else if !head.is_empty() {
        // Запись в String не падает
        let _ = write_highlighted(out, kind.name(), head);
    }
    push_escaped(out, tail);
    out.push_str("</pre>");
    let limited = !head.is_empty() && !tail.is_empty();
    if limited {
        out.push_str("<div class=\"banner\">Подсветка отключена после ");
        out.push_str(&size(limit));
        out.push_str("</div>");
    }
    limited
}

/// Размер лимита для баннера: до мебибайта — в КБ, иначе в МБ.
fn size(bytes: usize) -> String {
    const MIB: usize = 1024 * 1024;
    if bytes < MIB {
        format!("{} КБ", bytes / 1024)
    } else {
        format!("{} МБ", bytes / MIB)
    }
}
