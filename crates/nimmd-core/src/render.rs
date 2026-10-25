//! Рендер тела документа в HTML-фрагмент.

use std::fmt;
use std::sync::OnceLock;

use comrak::adapters::CodefenceRendererAdapter;
use comrak::nodes::{NodeValue, Sourcepos};
use comrak::options::Plugins;
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{Arena, format_html_with_plugins, html, parse_document};

use crate::parse;
use crate::text::limit_nesting;

/// HTML-фрагмент тела документа (без frontmatter).
///
/// Тотальна: на любой строке возвращает HTML без паники. Сырой HTML показывается текстом,
/// опасные адреса ссылок и картинок отбрасывает comrak, код подсвечивается CSS-классами,
/// блоки `mermaid` выводятся как `<pre class="mermaid">` для хоста.
pub fn render_fragment(body: &str) -> String {
    let limited = limit_nesting(body);
    let arena = Arena::new();
    let root = parse_document(&arena, &limited, parse::options());
    for node in root.descendants() {
        if let NodeValue::CodeBlock(block) = &mut node.data_mut().value {
            normalize_mermaid(&mut block.info);
        }
    }
    let mut out = String::new();
    // Запись в String не падает, а адаптеры ошибок не возвращают: при ошибке остаётся уже выведенный HTML
    let _ = format_html_with_plugins(root, parse::options(), &mut out, plugins());
    out
}

/// Первое слово info-строки `mermaid` в любом регистре приводится к `mermaid`:
/// comrak ищет рендерер блока по точному ключу.
fn normalize_mermaid(info: &mut String) {
    let end = info.find(char::is_whitespace).unwrap_or(info.len());
    if info
        .get(..end)
        .is_some_and(|lang| lang.eq_ignore_ascii_case("mermaid"))
    {
        info.replace_range(..end, "mermaid");
    }
}

fn plugins() -> &'static Plugins<'static> {
    static SYNTECT: OnceLock<SyntectAdapter> = OnceLock::new();
    static PLUGINS: OnceLock<Plugins<'static>> = OnceLock::new();
    PLUGINS.get_or_init(|| {
        let mut plugins = Plugins::default();
        // Без темы адаптер выводит CSS-классы, а не `style=`
        plugins.render.codefence_syntax_highlighter =
            Some(SYNTECT.get_or_init(|| SyntectAdapter::new(None)));
        plugins
            .render
            .codefence_renderers
            .insert("mermaid".to_owned(), &Mermaid);
        plugins
    })
}

/// Блок диаграммы: экранированный текст без подсветки, рисует его Mermaid на стороне хоста.
struct Mermaid;

impl CodefenceRendererAdapter for Mermaid {
    fn write(
        &self,
        output: &mut dyn fmt::Write,
        _lang: &str,
        _meta: &str,
        code: &str,
        _sourcepos: Option<Sourcepos>,
    ) -> fmt::Result {
        output.write_str("<pre class=\"mermaid\">")?;
        html::escape(output, code)?;
        output.write_str("</pre>\n")
    }
}
