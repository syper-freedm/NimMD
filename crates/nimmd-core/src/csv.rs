//! CSV: определение разделителя и раскраска столбцов (RFC 4180, без разбора в значения).
//!
//! Функции тотальны: на любой строке возвращают значение без паники.

use std::fmt::Write;

use crate::render::push_escaped;

/// Кандидаты в разделители в порядке предпочтения при равенстве.
const DELIMITERS: [u8; 4] = *b",;\t|";

/// Сколько первых записей смотрит [`detect_delimiter`].
const SAMPLE_RECORDS: usize = 50;

/// Разделитель полей: кандидат, у которого больше всего из первых 50 записей имеют самое частое
/// число полей (при равной частоте — большее); такое число должно быть больше 1, иначе `,`.
pub fn detect_delimiter(src: &str) -> u8 {
    let mut best = (b',', 0usize);
    for delim in DELIMITERS {
        let mut counts = Vec::new();
        let mut fields = 0usize;
        for token in Tokens::new(src, delim) {
            match token {
                Token::Field(_) => fields += 1,
                Token::Delimiter => {}
                Token::Newline => {
                    counts.push(fields);
                    fields = 0;
                    if counts.len() == SAMPLE_RECORDS {
                        break;
                    }
                }
            }
        }
        if fields > 0 {
            counts.push(fields);
        }
        let mode = counts
            .iter()
            .map(|&n| (counts.iter().filter(|&&m| m == n).count(), n))
            .max();
        if let Some((records, _)) = mode.filter(|&(records, n)| n > 1 && records > best.1) {
            best = (delim, records);
        }
    }
    best.0
}

/// Раскрашенный текст CSV: поле — `<span class="c{N mod 8}">`, разделитель — `<span class="d">`.
pub(crate) fn push_colored(out: &mut String, src: &str) {
    let delim = detect_delimiter(src);
    let mut column = 0usize;
    for token in Tokens::new(src, delim) {
        match token {
            Token::Field("") => {}
            Token::Field(field) => {
                // Запись в String не падает
                let _ = write!(out, "<span class=\"c{}\">", column % 8);
                push_escaped(out, field);
                out.push_str("</span>");
            }
            Token::Delimiter => {
                column += 1;
                out.push_str("<span class=\"d\">");
                out.push(char::from(delim));
                out.push_str("</span>");
            }
            Token::Newline => {
                column = 0;
                out.push('\n');
            }
        }
    }
}

enum Token<'a> {
    /// Исходник поля вместе с кавычками.
    Field(&'a str),
    Delimiter,
    Newline,
}

/// Токенизатор: поле, разделитель, конец записи `\n`.
struct Tokens<'a> {
    rest: &'a str,
    delim: u8,
    /// Следующий токен — поле: начало записи или после разделителя.
    field_next: bool,
}

impl<'a> Tokens<'a> {
    fn new(src: &'a str, delim: u8) -> Self {
        Self {
            rest: src,
            delim,
            field_next: !src.is_empty(),
        }
    }

    /// Поле в начале `rest`: в кавычках — до `"`, за которой не следует `"`, и дальше до
    /// разделителя или `\n`; незакрытая кавычка тянет поле до конца текста.
    fn field(&self) -> (&'a str, &'a str) {
        let bytes = self.rest.as_bytes();
        let mut start = 0;
        if bytes.first() == Some(&b'"') {
            let mut i = 1;
            start = bytes.len();
            while let Some(&b) = bytes.get(i) {
                if b == b'"' {
                    if bytes.get(i + 1) == Some(&b'"') {
                        i += 2;
                        continue;
                    }
                    start = i + 1;
                    break;
                }
                i += 1;
            }
        }
        let end = bytes
            .iter()
            .skip(start)
            .position(|&b| b == self.delim || b == b'\n')
            .map_or(bytes.len(), |i| start + i);
        // Разделители и `"` — ASCII: смещения на границе символа
        self.rest.split_at_checked(end).unwrap_or((self.rest, ""))
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        if self.field_next {
            self.field_next = false;
            let (field, rest) = self.field();
            self.rest = rest;
            return Some(Token::Field(field));
        }
        let &first = self.rest.as_bytes().first()?;
        self.rest = self.rest.get(1..).unwrap_or("");
        if first == self.delim {
            self.field_next = true;
            Some(Token::Delimiter)
        } else {
            self.field_next = !self.rest.is_empty();
            Some(Token::Newline)
        }
    }
}
