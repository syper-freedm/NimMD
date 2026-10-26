//! Подсветка синтаксиса: встроенный набор грамматик, адаптер comrak, спаны для редактора.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html;
use serde::Serialize;
use syntect::dumps::from_uncompressed_data;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

use crate::text::truncate_at_char_boundary;

/// Предел подсветки файла TOML/YAML/JSON и спанов [`highlight`] в байтах; дальше — текст без подсветки.
pub const HIGHLIGHT_LIMIT: usize = 64 * 1024;

/// Предел раскраски CSV в байтах; дальше — текст без раскраски.
pub const CSV_COLOR_LIMIT: usize = 2 * 1024 * 1024;

/// Суммарный объём подсвечиваемого кода (frontmatter и блоки) одного документа Markdown в байтах.
pub const MARKDOWN_HIGHLIGHT_LIMIT: usize = 8 * 1024;

/// Участок кода с неизменным стеком scope; смещения — в UTF-16 от начала кода.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    /// Атомы scope стека снаружи внутрь через пробел, без повторов.
    pub class: String,
}

/// Спаны подсветки кода для редактора.
///
/// Тотальна: при ошибке syntect или без грамматики — пустой `Vec`. Подсвечивается голова
/// не длиннее [`HIGHLIGHT_LIMIT`] (по правилу [`split_head`]), дальше спанов нет.
pub fn highlight(lang: &str, code: &str) -> Vec<Span> {
    spans(lang, split_head(code, HIGHLIGHT_LIMIT).0).unwrap_or_default()
}

fn spans(lang: &str, code: &str) -> Option<Vec<Span>> {
    let set = syntaxes();
    let mut state = ParseState::new(find_syntax(set, lang, code)?);
    let mut stack = ScopeStack::new();
    let mut out: Vec<Span> = Vec::new();
    let mut offset = 0u32;
    let mut push = |stack: &ScopeStack, text: &str| {
        let start = offset;
        offset = offset.saturating_add(u32::try_from(text.encode_utf16().count()).ok()?);
        if text.is_empty() || stack.is_empty() {
            return Some(());
        }
        let class = class_of(stack);
        match out.last_mut() {
            Some(last) if last.end == start && last.class == class => last.end = offset,
            _ => out.push(Span {
                start,
                end: offset,
                class,
            }),
        }
        Some(())
    };
    for line in LinesWithEndings::from(code) {
        let mut pos = 0;
        for (at, op) in state.parse_line(line, set).ok()? {
            push(&stack, line.get(pos..at)?)?;
            pos = at;
            stack.apply(&op).ok()?;
        }
        push(&stack, line.get(pos..)?)?;
    }
    Some(out)
}

fn class_of(stack: &ScopeStack) -> String {
    let mut atoms: Vec<String> = Vec::new();
    for scope in stack.as_slice() {
        for atom in scope.build_string().split('.') {
            if !atoms.iter().any(|a| a == atom) {
                atoms.push(atom.to_owned());
            }
        }
    }
    atoms.join(" ")
}

/// Голова и хвост кода: голова — до последнего `\n` (включительно) в первых `limit` байтах,
/// без `\n` — до границы символа не дальше предела. Короткий код — целиком голова.
pub(crate) fn split_head(code: &str, limit: usize) -> (&str, &str) {
    let (head, cut) = truncate_at_char_boundary(code, limit);
    if !cut {
        return (code, "");
    }
    let end = head.rfind('\n').map_or(head.len(), |i| i + 1);
    code.split_at_checked(end).unwrap_or((code, ""))
}

/// Встроенный набор грамматик (`assets/syntaxes.packdump`, рецепт `just syntaxes`);
/// дамп не прочитан — пустой набор, код выводится без подсветки.
fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(|| {
        from_uncompressed_data(include_bytes!("../assets/syntaxes.packdump"))
            .unwrap_or_else(|_| SyntaxSet::new())
    })
}

/// Грамматика: первое слово до `,` по токену, иначе по первой строке кода, иначе Plain Text;
/// без языка — Plain Text.
fn find_syntax<'a>(set: &'a SyntaxSet, lang: &str, code: &str) -> Option<&'a SyntaxReference> {
    let token = lang.split_once(',').map_or(lang, |(left, _)| left);
    (!token.is_empty())
        .then(|| {
            set.find_syntax_by_token(token)
                .or_else(|| set.find_syntax_by_first_line(code))
        })
        .flatten()
        .or_else(|| set.find_syntax_by_name("Plain Text"))
}

/// HTML кода с CSS-классами syntect; `None` — нет грамматики или ошибка syntect.
fn classed_html(lang: &str, code: &str) -> Option<String> {
    let set = syntaxes();
    let syntax = find_syntax(set, lang, code)?;
    let mut html = ClassedHTMLGenerator::new_with_class_style(syntax, set, ClassStyle::Spaced);
    for line in LinesWithEndings::from(code) {
        html.parse_html_for_line_which_includes_newline(line).ok()?;
    }
    Some(html.finalize())
}

/// Подсвеченный HTML кода или, без подсветки, весь его текст экранированным:
/// частичный вывод отбрасывается, сырой код в HTML не попадает.
pub(crate) fn write_highlighted(
    output: &mut dyn fmt::Write,
    lang: &str,
    code: &str,
) -> fmt::Result {
    write_html_or_escaped(output, classed_html(lang, code), code)
}

fn write_html_or_escaped(
    output: &mut dyn fmt::Write,
    highlighted: Option<String>,
    code: &str,
) -> fmt::Result {
    match highlighted {
        Some(html) => output.write_str(&html),
        None => html::escape(output, code),
    }
}

/// Адаптер подсветки блоков кода comrak со счётчиком [`MARKDOWN_HIGHLIGHT_LIMIT`]: один на
/// разбор документа. Атомики — потому что адаптер comrak обязан быть `Sync`.
#[derive(Default)]
pub(crate) struct Highlighter {
    used: AtomicUsize,
    limited: AtomicBool,
}

impl Highlighter {
    /// Подсвеченный HTML кода, а сверх лимита документа — экранированный текст.
    /// Код с грамматикой Plain Text лимит не тратит.
    pub(crate) fn write(&self, output: &mut dyn fmt::Write, lang: &str, code: &str) -> fmt::Result {
        let plain = find_syntax(syntaxes(), lang, code).is_none_or(|s| s.name == "Plain Text");
        if plain {
            return write_highlighted(output, lang, code);
        }
        let used = self.used.load(Ordering::Relaxed).saturating_add(code.len());
        if used <= MARKDOWN_HIGHLIGHT_LIMIT {
            self.used.store(used, Ordering::Relaxed);
            write_highlighted(output, lang, code)
        } else {
            self.limited.store(true, Ordering::Relaxed);
            html::escape(output, code)
        }
    }

    /// Хоть один фрагмент выведен без подсветки по лимиту.
    pub(crate) fn limited(&self) -> bool {
        self.limited.load(Ordering::Relaxed)
    }
}

impl SyntaxHighlighterAdapter for Highlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        self.write(output, lang.unwrap_or(""), code)
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

#[cfg(test)]
mod tests {
    use super::write_html_or_escaped;

    fn written(highlighted: Option<String>, code: &str) -> String {
        let mut out = String::new();
        write_html_or_escaped(&mut out, highlighted, code).unwrap();
        out
    }

    #[test]
    fn highlight_error_writes_escaped_code() {
        let code = "<script>alert(1)</script> & \"x\"";
        let out = written(None, code);
        assert_eq!(
            out,
            "&lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;x&quot;"
        );
        assert!(!out.contains("<script") && !out.contains("<span"), "{out}");
        assert_eq!(written(None, ""), "");
    }

    #[test]
    fn highlight_ok_written_unchanged() {
        let html = "<span class=\"source\">x</span>";
        assert_eq!(written(Some(html.to_owned()), "<b>other</b>"), html);
    }
}
