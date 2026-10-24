//! Предобработка исходного текста: BOM, переносы строк, frontmatter, ограничение размера.
//!
//! Все функции тотальны: на любой строке возвращают значение без паники.
#![deny(clippy::indexing_slicing, clippy::string_slice)]

use std::borrow::Cow;

use crate::profile::Dialect;

/// Предел размера документа в байтах.
pub const MAX_BYTES: usize = 10 * 1024 * 1024;

/// Стиль переноса строк.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
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
