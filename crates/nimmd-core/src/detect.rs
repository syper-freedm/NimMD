//! Детектор диалекта и расширений документа.
//!
//! Функции тотальны: на любой строке возвращают значение без паники.
#![deny(clippy::indexing_slicing, clippy::string_slice)]

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, parse_document};

use crate::parse;
use crate::profile::{Dialect, Features, Profile};
use crate::text::{frontmatter_dialect, limit_nesting, split_frontmatter};

/// Диалект и расширения документа.
///
/// Вход — текст без BOM и с переносами `\n` (как у [`split_frontmatter`]).
/// Ключ `markdown:` во frontmatter заменяет вычисленный диалект и не меняет расширения.
pub fn detect(src: &str) -> Profile {
    let (frontmatter, body) = split_frontmatter(src);
    let signs = Signs::scan(body);
    let arena = Arena::new();
    let limited = limit_nesting(src);
    let (math, mermaid) = ast_features(parse_document(&arena, &limited, &parse::options()));
    let features = Features {
        frontmatter: frontmatter.is_some(),
        math,
        mermaid,
        footnotes: signs.footnote,
        tables: signs.table,
        tasklists: signs.task,
        strikethrough: signs.strike,
        wikilinks: signs.wikilink,
        callouts: signs.callout,
    };
    let dialect = frontmatter
        .and_then(frontmatter_dialect)
        .unwrap_or_else(|| signs.dialect());
    Profile { dialect, features }
}

/// `(math, mermaid)` по дереву comrak; обход итеративный — глубина вложенности не грозит стеку.
pub(crate) fn ast_features<'a>(root: &'a AstNode<'a>) -> (bool, bool) {
    let (mut math, mut mermaid) = (false, false);
    for node in root.descendants() {
        match &node.data().value {
            NodeValue::Math(_) => math = true,
            NodeValue::CodeBlock(block) if block.fenced => {
                mermaid |= block
                    .info
                    .split_whitespace()
                    .next()
                    .is_some_and(|word| word.eq_ignore_ascii_case("mermaid"));
            }
            _ => {}
        }
        if math && mermaid {
            break;
        }
    }
    (math, mermaid)
}

/// Текстовые признаки тела вне блоков кода.
#[allow(clippy::struct_excessive_bools)] // независимые признаки
#[derive(Default)]
struct Signs {
    wikilink: bool,
    comment: bool,
    callout: bool,
    tag: bool,
    heading_id: bool,
    merged_cells: bool,
    metadata: bool,
    table: bool,
    task: bool,
    strike: bool,
    autolink: bool,
    footnote: bool,
}

/// Состояние блока метаданных `MultiMarkdown` в начале тела.
enum Meta {
    /// Пока идут строки `Ключ: значение`; `true` — встретился `Title` или `Author`.
    Open(bool),
    Closed,
}

impl Signs {
    /// Один проход по строкам тела.
    fn scan(body: &str) -> Self {
        let mut s = Self::default();
        let mut fence: Option<(char, usize)> = None;
        let mut meta = Meta::Open(false);
        let mut prev_has_pipe = false;
        for line in body.lines() {
            if let Meta::Open(named) = meta {
                if line.trim().is_empty() {
                    s.metadata |= named;
                    meta = Meta::Closed;
                } else if let Some(key) = meta_key(line) {
                    meta = Meta::Open(
                        named
                            || key.eq_ignore_ascii_case("title")
                            || key.eq_ignore_ascii_case("author"),
                    );
                } else {
                    meta = Meta::Closed;
                }
            }
            if let Some((ch, len)) = fence {
                if is_closing_fence(line, ch, len) {
                    fence = None;
                }
                continue;
            }
            if let Some(open) = opening_fence(line) {
                fence = Some(open);
                prev_has_pipe = false;
                continue;
            }
            s.line(line, prev_has_pipe);
            prev_has_pipe = line.contains('|');
        }
        if let Meta::Open(named) = meta {
            s.metadata |= named;
        }
        s
    }

    fn line(&mut self, line: &str, prev_has_pipe: bool) {
        self.wikilink |= line
            .split_once("[[")
            .is_some_and(|(_, rest)| rest.contains("]]"));
        self.comment |= enclosed(line, "%%");
        self.strike |= enclosed(line, "~~");
        self.callout |= is_callout(line);
        self.tag |= line
            .strip_prefix('#')
            .and_then(|rest| rest.chars().next())
            .is_some_and(char::is_alphabetic);
        self.heading_id |= is_heading_with_id(line);
        let row = line.trim_start();
        self.merged_cells |= row.starts_with('|') && row.trim_end().ends_with("||");
        self.table |= prev_has_pipe && is_table_delimiter(line);
        self.task |= is_task(line);
        self.autolink |= has_bare_autolink(line);
        self.footnote |= has_footnote(line);
    }

    fn dialect(&self) -> Dialect {
        if self.wikilink || self.comment || (self.callout && self.tag) {
            Dialect::Obsidian
        } else if self.heading_id || self.merged_cells || self.metadata {
            Dialect::MultiMarkdown
        } else if self.table || self.task || self.strike || self.autolink || self.footnote {
            Dialect::Gfm
        } else {
            Dialect::CommonMark
        }
    }
}

/// Строка без отступа больше трёх пробелов; `None`, если отступ больше.
fn strip_indent(line: &str) -> Option<&str> {
    let rest = line.trim_start_matches(' ');
    (line.len() - rest.len() <= 3).then_some(rest)
}

/// Открывающее ограждение: символ и длина серии.
fn opening_fence(line: &str) -> Option<(char, usize)> {
    let rest = strip_indent(line)?;
    let ch = rest.chars().next().filter(|&c| c == '`' || c == '~')?;
    let info = rest.trim_start_matches(ch);
    let len = rest.len() - info.len();
    // В info-строке ограждения из обратных кавычек их быть не может (CommonMark 4.5)
    (len >= 3 && !(ch == '`' && info.contains('`'))).then_some((ch, len))
}

fn is_closing_fence(line: &str, ch: char, open_len: usize) -> bool {
    strip_indent(line).is_some_and(|rest| {
        let tail = rest.trim_start_matches(ch);
        rest.len() - tail.len() >= open_len && tail.trim().is_empty()
    })
}

/// Между первой парой `delim` в строке есть хотя бы один символ.
fn enclosed(line: &str, delim: &str) -> bool {
    line.split_once(delim)
        .and_then(|(_, rest)| rest.split_once(delim))
        .is_some_and(|(inner, _)| !inner.is_empty())
}

/// `> [!тип]` — тип из букв, цифр и `-`.
fn is_callout(line: &str) -> bool {
    line.trim_start()
        .strip_prefix('>')
        .and_then(|rest| rest.trim_start_matches(' ').strip_prefix("[!"))
        .and_then(|rest| rest.split_once(']'))
        .is_some_and(|(kind, _)| {
            !kind.is_empty() && kind.chars().all(|c| c.is_alphanumeric() || c == '-')
        })
}

/// ATX-заголовок, оканчивающийся на `{#id}` с непустым id без пробелов.
fn is_heading_with_id(line: &str) -> bool {
    let Some(rest) = strip_indent(line) else {
        return false;
    };
    let text = rest.trim_start_matches('#');
    let level = rest.len() - text.len();
    (1..=6).contains(&level)
        && text.starts_with(' ')
        && text
            .trim_end()
            .strip_suffix('}')
            .and_then(|head| head.rsplit_once("{#"))
            .is_some_and(|(_, id)| !id.is_empty() && !id.contains(char::is_whitespace))
}

/// Ключ строки метаданных `Ключ: значение`.
fn meta_key(line: &str) -> Option<&str> {
    let (key, _) = line.split_once(':')?;
    let valid = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-'));
    valid.then_some(key.trim())
}

/// Строка-разделитель таблицы GFM: только `|`, `-`, `:` и пробелы, есть `-` и `|`.
fn is_table_delimiter(line: &str) -> bool {
    line.contains('-')
        && line.contains('|')
        && line
            .chars()
            .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'))
}

/// Пункт списка-задачи: маркер, пробел, `[ ]` / `[x]` / `[X]`, затем пробел или конец строки.
fn is_task(line: &str) -> bool {
    let item = line.trim_start();
    let after_marker = item.strip_prefix(['-', '*', '+']).or_else(|| {
        let rest = item.trim_start_matches(|c: char| c.is_ascii_digit());
        (rest.len() < item.len())
            .then(|| rest.strip_prefix(['.', ')']))
            .flatten()
    });
    let Some(after_marker) = after_marker else {
        return false;
    };
    let body = after_marker.trim_start_matches([' ', '\t']);
    body.len() < after_marker.len()
        && ["[ ]", "[x]", "[X]"].iter().any(|mark| {
            body.strip_prefix(mark)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
        })
}

/// `http://` или `https://`, перед которым нет `(` или `<`.
fn has_bare_autolink(line: &str) -> bool {
    line.match_indices("http").any(|(i, _)| {
        let starts_url = line
            .get(i..)
            .is_some_and(|rest| rest.starts_with("http://") || rest.starts_with("https://"));
        let wrapped = line
            .get(..i)
            .and_then(|before| before.chars().next_back())
            .is_some_and(|c| c == '(' || c == '<');
        starts_url && !wrapped
    })
}

/// `[^id]` — id из непробельных символов.
fn has_footnote(line: &str) -> bool {
    line.match_indices("[^").any(|(i, mark)| {
        line.get(i + mark.len()..).is_some_and(|rest| {
            // Разбор id обрывается на `[`: следующую `[^` проверит своя итерация, проход линейный
            let id_len = rest
                .find(|c: char| c.is_whitespace() || c == ']' || c == '[')
                .unwrap_or(rest.len());
            id_len > 0 && rest.get(id_len..).is_some_and(|tail| tail.starts_with(']'))
        })
    })
}
