//! Рендер тела документа в HTML-фрагмент.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use comrak::adapters::{CodefenceRendererAdapter, SyntaxHighlighterAdapter};
use comrak::nodes::{AstNode, NodeMath, NodeValue, Sourcepos};
use comrak::options::Plugins;
use comrak::{Arena, format_html_with_plugins, html, parse_document};
use latex2mathml::token::Token;
use latex2mathml::{DisplayStyle, latex_to_mathml};
use serde::{Deserialize, Serialize};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::detect::ast_features;
use crate::parse;
use crate::text::{MAX_NESTING, limit_nesting};

/// Предел длины формулы для `latex2mathml` в байтах.
const MAX_TEX_BYTES: usize = 10_000;

/// Вывод формул.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MathMode {
    /// `<span data-math-style>` для KaTeX на стороне хоста.
    #[default]
    Katex,
    /// MathML без скриптов.
    MathML,
    /// Экранированный TeX.
    Source,
}

/// HTML-фрагмент тела документа (без frontmatter).
///
/// Тотальна: на любой строке возвращает HTML без паники. Сырой HTML показывается текстом,
/// опасные адреса ссылок и картинок отбрасывает comrak, код подсвечивается CSS-классами,
/// блоки `mermaid` выводятся как `<pre class="mermaid">` для хоста.
pub fn render_fragment(body: &str) -> String {
    render_body(body, MathMode::Katex, true).html
}

/// Результат одного разбора тела: фрагмент, флаги профиля и заголовок.
pub(crate) struct Body {
    pub html: String,
    pub math: bool,
    pub mermaid: bool,
    pub title: Option<String>,
}

/// Разбирает тело один раз: из того же дерева берутся флаги `math`/`mermaid` и заголовок.
pub(crate) fn render_body(body: &str, math: MathMode, remote_images: bool) -> Body {
    let limited = limit_nesting(body);
    let arena = Arena::new();
    let root = parse_document(&arena, &limited, parse::options());
    // Флаги — до замены формул на `Raw`
    let (has_math, mermaid) = ast_features(root);
    let mut title = None;
    let nodes: Vec<_> = root.descendants().collect();
    for node in nodes {
        if title.is_none() && matches!(&node.data().value, NodeValue::Heading(h) if h.level == 1) {
            title = Some(heading_text(node));
        }
        let raw = match &mut node.data_mut().value {
            NodeValue::CodeBlock(block) => {
                normalize_mermaid(&mut block.info);
                None
            }
            NodeValue::Image(link) if !remote_images && is_remote(&link.url) => {
                let mut out = String::from("<span class=\"remote-image\">");
                push_escaped(&mut out, &link.url);
                out.push_str("</span>");
                Some(out)
            }
            NodeValue::Math(formula) => math_html(formula, math),
            _ => None,
        };
        if let Some(raw) = raw {
            node.data_mut().value = NodeValue::Raw(raw);
            let children: Vec<_> = node.children().collect();
            for child in children {
                child.detach();
            }
        }
    }
    let mut out = String::new();
    // Запись в String не падает, а адаптеры ошибок не возвращают: при ошибке остаётся уже выведенный HTML
    let _ = format_html_with_plugins(root, parse::options(), &mut out, plugins());
    Body {
        html: out,
        math: has_math,
        mermaid,
        title: title.filter(|text: &String| !text.is_empty()),
    }
}

/// YAML frontmatter, подсвеченный CSS-классами syntect, как блок кода `yaml`.
pub(crate) fn highlight_yaml(out: &mut String, yaml: &str) {
    // Запись в String не падает
    let _ = Highlighter.write_highlighted(out, Some("yaml"), yaml);
}

pub(crate) fn push_escaped(out: &mut String, text: &str) {
    // Запись в String не падает
    let _ = html::escape(out, text);
}

/// Текст заголовка без разметки: текстовые узлы и код.
fn heading_text<'a>(heading: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for node in heading.descendants() {
        match &node.data().value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(code) => text.push_str(&code.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            _ => {}
        }
    }
    text
}

/// Адрес с сетевой схемой: `http:`/`https:` в любом регистре или `//` (браузер считает `\` за `/`).
fn is_remote(url: &str) -> bool {
    // Ведущие управляющие символы и пробелы парсер URL отбрасывает
    let url = url.trim_start_matches(|c: char| c <= ' ');
    let scheme = url.split_once(':').map(|(scheme, _)| scheme);
    scheme.is_some_and(|s| s.eq_ignore_ascii_case("http") || s.eq_ignore_ascii_case("https"))
        || url
            .chars()
            .take(2)
            .filter(|&c| c == '/' || c == '\\')
            .count()
            == 2
}

/// HTML формулы в режиме `mode`; `None` — оставить `<span data-math-style>` comrak.
fn math_html(formula: &NodeMath, mode: MathMode) -> Option<String> {
    match mode {
        MathMode::Katex => None,
        MathMode::MathML => {
            Some(mathml(&formula.literal, formula.display_math).unwrap_or_else(|| source(formula)))
        }
        MathMode::Source => Some(source(formula)),
    }
}

/// Экранированный TeX: inline — `<code>`, display — `<code class="math-display">`.
///
/// Без `<pre>`: формула стоит внутри `<p>`, а `<pre>` в `<p>` браузер выносит из абзаца.
fn source(formula: &NodeMath) -> String {
    let mut out = String::from(if formula.display_math {
        "<code class=\"math-display\">"
    } else {
        "<code>"
    });
    push_escaped(&mut out, &formula.literal);
    out.push_str("</code>");
    out
}

/// MathML формулы; `None` — формула слишком длинная или глубокая, не поддерживается или не разобрана.
fn mathml(tex: &str, display: bool) -> Option<String> {
    // Рекурсивный разбор latex2mathml переполнил бы стек Quick Look: проверка до вызова
    if tex.len() > MAX_TEX_BYTES || !within_depth(tex) {
        return None;
    }
    let style = if display {
        DisplayStyle::Block
    } else {
        DisplayStyle::Inline
    };
    let out = latex_to_mathml(tex, style).ok()?;
    // Неизвестные команды latex2mathml выводит текстом отладки — лучше показать исходник
    if out.contains("[PARSE ERROR") {
        return None;
    }
    Some(escape_stray(&out))
}

/// Число токенов, с которых разбор latex2mathml уходит в рекурсию, не больше `MAX_NESTING`.
///
/// Каждый уровень рекурсии разбора и вывода начинается с отдельного такого токена (`{`, `^`, `_`,
/// команда с аргументом), поэтому их общее число — оценка глубины сверху. `\middle` отвергается:
/// на аргументе не из скобок latex2mathml паникует (`unimplemented!`).
fn within_depth(tex: &str) -> bool {
    let mut count = 0usize;
    let mut chars = tex.chars().peekable();
    while let Some(c) = chars.next() {
        let nests = match c {
            '{' | '^' | '_' => true,
            '\\' => {
                let mut name = String::new();
                if let Some(first) = chars.next() {
                    name.push(first);
                    if first.is_ascii_alphabetic() {
                        while let Some(next) = chars.next_if(char::is_ascii_alphabetic) {
                            name.push(next);
                        }
                    }
                }
                if name == "middle" {
                    return false;
                }
                !matches!(
                    Token::from_command(&name),
                    Token::Letter(..)
                        | Token::Operator(_)
                        | Token::Function(_)
                        | Token::Space(_)
                        | Token::Paren(_)
                        | Token::Number(_)
                        | Token::NewLine
                        | Token::Ampersand
                        | Token::Right
                        | Token::End
                        | Token::Text
                        | Token::OperatorName
                        | Token::Big(_)
                        | Token::Command(_)
                )
            }
            _ => false,
        };
        count += usize::from(nests);
        if count > MAX_NESTING {
            return false;
        }
    }
    true
}

/// latex2mathml не экранирует текст формулы: `<` и `&` не из его разметки экранируются.
///
/// Свои теги он пишет как `<имя` и `</имя`, сущности — как `&#x…;`, `&lang;`, `&rang;`.
fn escape_stray(mathml: &str) -> String {
    let mut out = String::with_capacity(mathml.len());
    let mut rest = mathml;
    while let Some(c) = rest.chars().next() {
        rest = rest.get(c.len_utf8()..).unwrap_or("");
        match c {
            '<' if !rest.starts_with(|n: char| n.is_ascii_alphabetic() || n == '/') => {
                out.push_str("&lt;");
            }
            '&' if !(rest.starts_with('#')
                || rest.starts_with("lang;")
                || rest.starts_with("rang;")) =>
            {
                out.push_str("&amp;");
            }
            _ => out.push(c),
        }
    }
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

fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Подсветка кода CSS-классами syntect; выбор грамматики — как у адаптера syntect в comrak 0.56.
struct Highlighter;

impl Highlighter {
    fn highlight(lang: Option<&str>, code: &str) -> Result<String, syntect::Error> {
        let syntaxes = syntaxes();
        // Без языка — Plain Text, грамматика по первой строке не ищется
        let syntax = lang
            .filter(|l| !l.is_empty())
            .map(|l| l.split_once(',').map_or(l, |(left, _)| left))
            .and_then(|token| {
                syntaxes
                    .find_syntax_by_token(token)
                    .or_else(|| syntaxes.find_syntax_by_first_line(code))
            })
            .unwrap_or_else(|| syntaxes.find_syntax_plain_text());
        let mut html =
            ClassedHTMLGenerator::new_with_class_style(syntax, syntaxes, ClassStyle::Spaced);
        for line in LinesWithEndings::from(code) {
            html.parse_html_for_line_which_includes_newline(line)?;
        }
        Ok(html.finalize())
    }
}

/// Подсвеченный HTML блока или, при ошибке syntect, весь его текст экранированным:
/// частичный вывод отбрасывается, сырой код в HTML не попадает.
fn write_highlight_or_escaped(
    output: &mut dyn fmt::Write,
    highlighted: Result<String, syntect::Error>,
    code: &str,
) -> fmt::Result {
    match highlighted {
        Ok(html) => output.write_str(&html),
        Err(_) => html::escape(output, code),
    }
}

impl SyntaxHighlighterAdapter for Highlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        write_highlight_or_escaped(output, Self::highlight(lang, code), code)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        html::write_opening_tag(output, "pre", [("class", "syntax-highlighting")])
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        // Значения атрибутов (класс языка из документа) экранирует comrak
        html::write_opening_tag(output, "code", attributes)
    }
}

fn plugins() -> &'static Plugins<'static> {
    static PLUGINS: OnceLock<Plugins<'static>> = OnceLock::new();
    PLUGINS.get_or_init(|| {
        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&Highlighter);
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

#[cfg(test)]
mod tests {
    use super::write_highlight_or_escaped;

    fn written(highlighted: Result<String, syntect::Error>, code: &str) -> String {
        let mut out = String::new();
        write_highlight_or_escaped(&mut out, highlighted, code).unwrap();
        out
    }

    #[test]
    fn highlight_error_writes_escaped_code() {
        let code = "<script>alert(1)</script> & \"x\"";
        let out = written(Err(syntect::Error::Fmt(std::fmt::Error)), code);
        assert_eq!(
            out,
            "&lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;x&quot;"
        );
        assert!(!out.contains("<script") && !out.contains("<span"), "{out}");
        assert_eq!(written(Err(syntect::Error::Fmt(std::fmt::Error)), ""), "");
    }

    #[test]
    fn highlight_ok_written_unchanged() {
        let html = "<span class=\"source\">x</span>";
        assert_eq!(written(Ok(html.to_owned()), "<b>other</b>"), html);
    }
}
