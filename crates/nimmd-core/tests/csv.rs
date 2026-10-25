use std::fmt::Write;

use nimmd_core::csv::detect_delimiter;
use nimmd_core::{FileKind, RenderOptions, render};

const DELIMITERS: [u8; 4] = *b",;\t|";

#[test]
fn detect_delimiter_cases() {
    let cases = [
        ("a,b,c\n1,2,3\n", b','),
        ("имя;цена\nчай;1,5\nкофе;2,5\n", b';'),
        ("a\tb\n1\t2\n", b'\t'),
        ("a|b\n1|2\n", b'|'),
        ("\"a;b\",c\n\"1;2\",3\n", b','),
        ("один\nдва\n", b','),
        ("", b','),
    ];
    for (src, delim) in cases {
        assert_eq!(
            char::from(detect_delimiter(src)),
            char::from(delim),
            "{src:?}"
        );
    }
}

#[test]
fn detect_delimiter_ties_and_mode() {
    // Равное число записей — порядок `,` `;` `\t` `|`
    assert_eq!(detect_delimiter("a,b;c\n"), b',');
    assert_eq!(detect_delimiter("a;b|c\n"), b';');
    assert_eq!(detect_delimiter("a|b\tc\n"), b'\t');
    // Мода при равной частоте — большее число полей: у `;` числа [1, 2] → мода 2
    assert_eq!(detect_delimiter("a\nb;c\n"), b';');
    // Больше записей с модой побеждает порядок
    assert_eq!(detect_delimiter("a,b\n1;2\n3;4\n"), b';');
    // Без `\n` в конце последняя запись считается
    assert_eq!(detect_delimiter("a|b"), b'|');
    // Разделитель внутри кавычек не считается, `""` не закрывает поле
    assert_eq!(detect_delimiter("\"a\"\";b\"|c\n\"1;2\"|3\n"), b'|');
    // Незакрытая кавычка тянет поле до конца
    assert_eq!(detect_delimiter("a;\"b\nc;d\ne;f\n"), b';');
    assert_eq!(detect_delimiter("\"a;b\nc;d\n"), b',');
    for src in ["\n", "\n\n", "\"", "\"\"", ";", "ж"] {
        let _ = detect_delimiter(src);
    }
    assert_eq!(detect_delimiter(";"), b';');
}

#[test]
fn detect_delimiter_reads_first_50_records() {
    let head = "a;b\n".repeat(50);
    let tail = "a,b,c\n".repeat(100);
    assert_eq!(detect_delimiter(&format!("{head}{tail}")), b';');
    // В выборке 49 записей `;` и одна `a,b,c`: у `,` мода 1
    let head = "a;b\n".repeat(49);
    assert_eq!(detect_delimiter(&format!("{head}{tail}")), b';');
    // Записи в кавычках с переносом — одна запись
    let quoted = "\"x\ny\";1\n".repeat(50);
    assert_eq!(detect_delimiter(&format!("{quoted}{tail}")), b';');
}

/// Токен эталонного разбора: поле с кавычками, разделитель, конец записи.
#[derive(Debug, PartialEq)]
enum T {
    F(String),
    D,
    N,
}

/// Эталонный токенизатор по FR-7, посимвольно.
fn tokens(src: &str, delim: char) -> Vec<T> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    if chars.is_empty() {
        return out;
    }
    let mut i = 0;
    loop {
        let start = i;
        if chars.get(i) == Some(&'"') {
            i += 1;
            loop {
                match chars.get(i) {
                    None => break,
                    Some('"') if chars.get(i + 1) == Some(&'"') => i += 2,
                    Some('"') => {
                        i += 1;
                        break;
                    }
                    Some(_) => i += 1,
                }
            }
        }
        while chars.get(i).is_some_and(|&c| c != delim && c != '\n') {
            i += 1;
        }
        out.push(T::F(chars[start..i].iter().collect()));
        match chars.get(i) {
            None => break,
            Some('\n') => {
                out.push(T::N);
                i += 1;
                if i == chars.len() {
                    break;
                }
            }
            Some(_) => {
                out.push(T::D);
                i += 1;
            }
        }
    }
    out
}

fn reference_delimiter(src: &str) -> u8 {
    let mut best = (b',', 0);
    for delim in DELIMITERS {
        let mut counts = Vec::new();
        let mut fields = 0;
        for t in tokens(src, char::from(delim)) {
            match t {
                T::F(_) => fields += 1,
                T::D => {}
                T::N => {
                    counts.push(fields);
                    fields = 0;
                }
            }
        }
        if fields > 0 {
            counts.push(fields);
        }
        counts.truncate(50);
        let mut mode = (0, 0);
        for &n in &counts {
            let freq = counts.iter().filter(|&&m| m == n).count();
            if (freq, n) > mode {
                mode = (freq, n);
            }
        }
        if mode.1 > 1 && mode.0 > best.1 {
            best = (delim, mode.0);
        }
    }
    best.0
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Эталонная раскраска FR-8.
fn reference_colored(src: &str) -> String {
    let delim = char::from(reference_delimiter(src));
    let mut out = String::new();
    let mut column = 0;
    for t in tokens(src, delim) {
        match t {
            T::F(field) if field.is_empty() => {}
            T::F(field) => {
                let _ = write!(
                    out,
                    "<span class=\"c{}\">{}</span>",
                    column % 8,
                    escape(&field)
                );
            }
            T::D => {
                column += 1;
                let _ = write!(out, "<span class=\"d\">{delim}</span>");
            }
            T::N => {
                column = 0;
                out.push('\n');
            }
        }
    }
    out
}

fn rendered_pre(src: &str) -> String {
    let opts = RenderOptions {
        kind: FileKind::Csv,
        ..RenderOptions::default()
    };
    let out = render(src, &opts);
    let inner = out
        .body
        .split_once("<pre class=\"data\">")
        .and_then(|(_, rest)| rest.split_once("</pre>"));
    assert!(inner.is_some(), "нет <pre class=\"data\">: {}", out.body);
    inner.map_or_else(String::new, |(pre, _)| pre.to_owned())
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        usize::try_from(self.0 % 1_000_003).unwrap_or_default()
    }
}

#[test]
fn random_csv_matches_reference() {
    let alphabet = [
        "a", "b", "1", ",", ";", "\t", "|", "\"", "\"\"", "\n", "ж", "<", "&", " ",
    ];
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for _ in 0..3000 {
        let len = rng.next() % 40;
        let src: String = (0..len)
            .map(|_| alphabet[rng.next() % alphabet.len()])
            .collect();
        assert_eq!(
            char::from(detect_delimiter(&src)),
            char::from(reference_delimiter(&src)),
            "{src:?}"
        );
        assert_eq!(rendered_pre(&src), reference_colored(&src), "{src:?}");
    }
}

#[test]
fn random_tables_detect_their_delimiter() {
    let mut rng = Rng(0x0123_4567_89AB_CDEF);
    for _ in 0..500 {
        let delim = char::from(DELIMITERS[rng.next() % DELIMITERS.len()]);
        let cols = 2 + rng.next() % 6;
        let rows = 1 + rng.next() % 80;
        let mut src = String::new();
        for _ in 0..rows {
            let cells: Vec<String> = (0..cols)
                .map(|_| match rng.next() % 4 {
                    0 => String::new(),
                    // В кавычках — свой разделитель, перенос и `""`
                    1 => format!("\"x{delim}y\n\"\"q\"\"\""),
                    _ => "ж1".repeat(rng.next() % 3),
                })
                .collect();
            src.push_str(&cells.join(&delim.to_string()));
            src.push('\n');
        }
        assert_eq!(char::from(detect_delimiter(&src)), delim, "{src:?}");
        assert_eq!(rendered_pre(&src), reference_colored(&src), "{src:?}");
    }
}
