//! Предобработка исходного текста: BOM, переносы строк, frontmatter, ограничение размера.
//!
//! Все функции тотальны: на любой строке возвращают значение без паники.
#![deny(clippy::indexing_slicing, clippy::string_slice)]

use std::borrow::Cow;

use serde::Serialize;

use crate::profile::Dialect;

/// Предел размера документа в байтах.
pub const MAX_BYTES: usize = 10 * 1024 * 1024;

/// Стиль переноса строк.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// `\n`.
    Lf,
    /// `\r\n`.
    CrLf,
}

/// Байты в текст с заменой невалидных последовательностей на `U+FFFD`; `true`, если замены были.
pub fn decode_lossy(bytes: &[u8]) -> (Cow<'_, str>, bool) {
    let text = String::from_utf8_lossy(bytes);
    let replaced = matches!(text, Cow::Owned(_));
    (text, replaced)
}

/// Отрезает один ведущий BOM (`U+FEFF`); `true`, если он был.
pub fn strip_bom(src: &str) -> (bool, &str) {
    src.strip_prefix('\u{FEFF}')
        .map_or((false, src), |rest| (true, rest))
}

/// Преобладающий стиль переносов; при равенстве — `Lf`. Одиночный `\r` не считается.
pub fn detect_line_ending(src: &str) -> LineEnding {
    let (mut crlf, mut lf) = (0usize, 0usize);
    let mut prev = 0u8;
    for &b in src.as_bytes() {
        if b == b'\n' {
            if prev == b'\r' {
                crlf += 1;
            } else {
                lf += 1;
            }
        }
        prev = b;
    }
    if crlf > lf {
        LineEnding::CrLf
    } else {
        LineEnding::Lf
    }
}

/// Приводит `\r\n` и одиночный `\r` к `\n`; без `\r` возвращает исходную строку без копирования.
pub fn normalize_newlines(src: &str) -> Cow<'_, str> {
    if src.contains('\r') {
        Cow::Owned(src.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(src)
    }
}

/// Делит текст на frontmatter (от строки `---` до следующей строки `---` включительно) и тело.
///
/// Вход — без BOM и с переносами `\n`. Всегда `frontmatter + тело == src`.
pub fn split_frontmatter(src: &str) -> (Option<&str>, &str) {
    const OPEN: &str = "---\n";
    let Some(rest) = src.strip_prefix(OPEN) else {
        return (None, src);
    };
    let mut end = OPEN.len();
    for line in rest.split_inclusive('\n') {
        end += line.len();
        if line.strip_suffix('\n').unwrap_or(line) == "---" {
            return src
                .split_at_checked(end)
                .map_or((None, src), |(fm, body)| (Some(fm), body));
        }
    }
    (None, src)
}

/// Диалект из первой строки frontmatter вида `markdown: <диалект>` с ключом в начале строки.
pub fn frontmatter_dialect(frontmatter: &str) -> Option<Dialect> {
    const BLANKS: [char; 2] = [' ', '\t'];
    frontmatter
        .lines()
        .find_map(|line| {
            let value = line
                .strip_prefix("markdown")?
                .trim_start_matches(BLANKS)
                .strip_prefix(':')?
                .trim_matches(BLANKS);
            let is_word =
                !value.is_empty() && value.chars().all(|c| c.is_alphanumeric() || c == '_');
            is_word.then_some(value)
        })?
        .parse()
        .ok()
}

/// Самый длинный префикс не длиннее `max` байт, кончающийся на границе символа; `true`, если текст обрезан.
pub fn truncate_at_char_boundary(src: &str, max: usize) -> (&str, bool) {
    if src.len() <= max {
        return (src, false);
    }
    let end = (0..=max)
        .rev()
        .find(|&i| src.is_char_boundary(i))
        .unwrap_or(0);
    (src.get(..end).unwrap_or(""), true)
}

/// Предел вложенности контейнеров (`>` и маркеров списка) в префиксе строки (спек 5.2).
pub const MAX_NESTING: usize = 64;

/// Предел ширины ведущего отступа строки в колонках; табуляция — до следующей кратной 4 колонки.
const MAX_INDENT: usize = 4 * MAX_NESTING;

/// Ограничивает глубину вложенности контейнеров в префиксе каждой строки.
///
/// Без разбора comrak квадратичен по глубине вложенности. Перед маркером контейнера сверх
/// [`MAX_NESTING`] вставляется `\` (у `N.`/`N)` — перед `.`/`)`), остаток строки становится текстом;
/// ведущие пробелы и табуляции шире 4 × [`MAX_NESTING`] колонок сокращаются до этой ширины.
/// Кроме сокращённого отступа символы исходника не теряются: удаление вставленных `\` и
/// восстановление отступа дают исходник. Без таких строк возвращает исходную строку без копирования.
pub fn limit_nesting(src: &str) -> Cow<'_, str> {
    let mut out = String::new();
    let (mut copied, mut line_start) = (0, 0);
    for line in src.split_inclusive('\n') {
        if let Some((keep, indent_end)) = indent_overflow(line) {
            out.push_str(src.get(copied..line_start + keep).unwrap_or(""));
            copied = line_start + indent_end;
        }
        if let Some(at) = overflow_marker(line) {
            out.push_str(src.get(copied..line_start + at).unwrap_or(""));
            out.push('\\');
            copied = line_start + at;
        }
        line_start += line.len();
    }
    if out.is_empty() {
        return Cow::Borrowed(src);
    }
    out.push_str(src.get(copied..).unwrap_or(""));
    Cow::Owned(out)
}

/// `(конец части отступа шириной MAX_INDENT, конец отступа)` в байтах, если отступ шире `MAX_INDENT`.
fn indent_overflow(line: &str) -> Option<(usize, usize)> {
    let (mut col, mut keep) = (0, None);
    for (i, b) in line.bytes().enumerate() {
        col = match b {
            b' ' => col + 1,
            b'\t' => col / 4 * 4 + 4,
            _ => return keep.zip((col > MAX_INDENT).then_some(i)),
        };
        // Табуляция не перескакивает кратную 4 границу, поэтому ширина MAX_INDENT достигается точно
        if col == MAX_INDENT {
            keep = Some(i + 1);
        }
    }
    keep.zip((col > MAX_INDENT).then_some(line.len()))
}

/// Байтовое смещение для `\` у маркера с номером `MAX_NESTING + 1` в префиксе строки:
/// начало маркера, а у `N.`/`N)` — позиция `.`/`)`, чтобы `\` экранировал пунктуацию и не был виден.
fn overflow_marker(line: &str) -> Option<usize> {
    let mut rest = line;
    for _ in 0..MAX_NESTING {
        rest = container_marker(rest.trim_start_matches([' ', '\t']))?;
    }
    let tail = rest.trim_start_matches([' ', '\t']);
    container_marker(tail)?;
    Some(line.len() - tail.trim_start_matches(|c: char| c.is_ascii_digit()).len())
}

/// Остаток после маркера контейнера в начале `s`: `>` или `-`, `*`, `+`, `N.`, `N)` с пробелом либо концом строки.
fn container_marker(s: &str) -> Option<&str> {
    if let Some(rest) = s.strip_prefix('>') {
        return Some(rest);
    }
    let rest = s.strip_prefix(['-', '*', '+']).or_else(|| {
        let digits = s.trim_start_matches(|c: char| c.is_ascii_digit());
        (digits.len() < s.len())
            .then(|| digits.strip_prefix(['.', ')']))
            .flatten()
    })?;
    (rest.is_empty() || rest.starts_with([' ', '\t', '\n', '\r'])).then_some(rest)
}
