use std::borrow::Cow;

use nimmd_core::profile::Dialect;
use nimmd_core::text::{
    LineEnding, MAX_BYTES, detect_line_ending, frontmatter_dialect, normalize_newlines,
    split_frontmatter, strip_bom, truncate_at_char_boundary,
};

#[test]
fn strip_bom_removes_one_leading_bom() {
    assert_eq!(strip_bom("\u{FEFF}# x"), (true, "# x"));
    assert_eq!(strip_bom("# x"), (false, "# x"));
    assert_eq!(strip_bom("\u{FEFF}\u{FEFF}x"), (true, "\u{FEFF}x"));
    assert_eq!(strip_bom("x\u{FEFF}"), (false, "x\u{FEFF}"));
    assert_eq!(strip_bom(""), (false, ""));
}

#[test]
fn detect_line_ending_picks_majority() {
    assert_eq!(detect_line_ending("a\r\nb"), LineEnding::CrLf);
    assert_eq!(detect_line_ending("abc"), LineEnding::Lf);
    assert_eq!(detect_line_ending("a\r\nb\r\nc\nd"), LineEnding::CrLf);
    assert_eq!(detect_line_ending("a\r\nb\n"), LineEnding::Lf);
    assert_eq!(detect_line_ending("a\rb\rc\n"), LineEnding::Lf);
    assert_eq!(detect_line_ending(""), LineEnding::Lf);
    assert_eq!(detect_line_ending("\r\r\n"), LineEnding::CrLf);
}

#[test]
fn normalize_newlines_converts_cr_and_crlf() {
    assert!(matches!(normalize_newlines("a\nb"), Cow::Borrowed("a\nb")));
    assert_eq!(normalize_newlines("a\r\nb\rc"), "a\nb\nc");
    assert_eq!(normalize_newlines("\r\r\n"), "\n\n");
    assert!(matches!(normalize_newlines(""), Cow::Borrowed("")));
}

#[test]
fn split_frontmatter_cases() {
    assert_eq!(
        split_frontmatter("---\ntitle: x\n---\n# H\n"),
        (Some("---\ntitle: x\n---\n"), "# H\n")
    );
    assert_eq!(split_frontmatter("---\n---\n"), (Some("---\n---\n"), ""));
    assert_eq!(
        split_frontmatter("---\na: 1\n---"),
        (Some("---\na: 1\n---"), "")
    );
    for src in [
        "---\ntitle: x\n# H\n",
        "text\n---\na\n---\n",
        "+++\na=1\n+++\nbody",
        "--- \na\n---\n",
        "---\na\n--- \nb\n",
        "",
    ] {
        assert_eq!(split_frontmatter(src), (None, src), "{src:?}");
    }
}

#[test]
fn split_frontmatter_stops_at_first_closing_line() {
    assert_eq!(
        split_frontmatter("---\na\n---\nb\n---\n"),
        (Some("---\na\n---\n"), "b\n---\n")
    );
    assert_eq!(split_frontmatter("---"), (None, "---"));
    assert_eq!(split_frontmatter("---\n"), (None, "---\n"));
    assert_eq!(split_frontmatter("---\n----\n"), (None, "---\n----\n"));
}

#[test]
fn split_frontmatter_concatenates_back() {
    for src in [
        "---\ntitle: x\n---\n# H\n",
        "---\n---\n",
        "---\na: 1\n---",
        "---\ntitle: x\n# H\n",
        "text\n---\na\n---\n",
        "+++\na=1\n+++\nbody",
        "--- \na\n---\n",
        "---\na\n--- \nb\n",
        "",
    ] {
        let (fm, body) = split_frontmatter(src);
        assert_eq!(fm.unwrap_or("").to_owned() + body, src, "{src:?}");
    }
}

#[test]
fn frontmatter_dialect_cases() {
    assert_eq!(
        frontmatter_dialect("---\nmarkdown:  GFM\n---\n"),
        Some(Dialect::Gfm)
    );
    assert_eq!(
        frontmatter_dialect("---\nmarkdown : obsidian \n---\n"),
        Some(Dialect::Obsidian)
    );
    for fm in [
        "---\ntitle: markdown: gfm\n---\n",
        "---\n  markdown: gfm\n---\n",
        "---\nmarkdown: kramdown\n---\n",
        "---\nMarkdown: gfm\n---\n",
    ] {
        assert_eq!(frontmatter_dialect(fm), None, "{fm:?}");
    }
}

#[test]
fn frontmatter_dialect_edge_cases() {
    assert_eq!(
        frontmatter_dialect("---\nmarkdown:\tmultimarkdown\t\n---\n"),
        Some(Dialect::MultiMarkdown)
    );
    assert_eq!(
        frontmatter_dialect("---\nmarkdown:gfm\n---"),
        Some(Dialect::Gfm)
    );
    // Первая подходящая строка решает, даже если её значение не диалект.
    assert_eq!(
        frontmatter_dialect("---\nmarkdown: kramdown\nmarkdown: gfm\n---\n"),
        None
    );
    assert_eq!(
        frontmatter_dialect("---\nmarkdown: \nmarkdown: gfm\n---\n"),
        Some(Dialect::Gfm)
    );
    for fm in [
        "",
        "---\nmarkdown: gfm extra\n---\n",
        "---\nmarkdown: 'gfm'\n---\n",
        "---\nmarkdownx: gfm\n---\n",
        "---\nmarkdown gfm\n---\n",
    ] {
        assert_eq!(frontmatter_dialect(fm), None, "{fm:?}");
    }
}

#[test]
fn truncate_at_char_boundary_cases() {
    assert_eq!(truncate_at_char_boundary("aé", 2), ("a", true));
    assert_eq!(truncate_at_char_boundary("aé", 10), ("aé", false));
    assert_eq!(truncate_at_char_boundary("aé", 0), ("", true));
    assert_eq!(truncate_at_char_boundary("aé", 3), ("aé", false));
    assert_eq!(truncate_at_char_boundary("", 0), ("", false));
    assert_eq!(MAX_BYTES, 10_485_760);
}

/// Детерминированный генератор (xorshift64), чтобы падение воспроизводилось.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn random_string(rng: &mut Rng) -> String {
    const PIECES: [&str; 14] = [
        "-", "---", "\n", "\r", "\r\n", " ", "\t", "a", "é", "😀", "\u{FEFF}", "+++", "markdown",
        ":",
    ];
    let len = rng.next() % 24;
    (0..len)
        .map(|_| PIECES[usize::from(rng.next().to_le_bytes()[0] % 14)])
        .collect()
}

fn fixed_strings() -> Vec<String> {
    [
        "",
        "\r",
        "---",
        "---\n",
        "é",
        "😀",
        "a\u{FEFF}b",
        "\u{FEFF}---\n---\n",
        "---\n😀\n---\nтело",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

fn reference_line_ending(src: &str) -> LineEnding {
    let crlf = src.matches("\r\n").count();
    let lf = src.matches('\n').count() - crlf;
    if crlf > lf {
        LineEnding::CrLf
    } else {
        LineEnding::Lf
    }
}

fn reference_normalize(src: &str) -> String {
    let mut out = String::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            chars.next_if_eq(&'\n');
            out.push('\n');
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn functions_are_total_and_match_reference() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let inputs = fixed_strings()
        .into_iter()
        .chain((0..2000).map(|_| random_string(&mut rng)));
    for src in inputs {
        let (had_bom, rest) = strip_bom(&src);
        assert_eq!(had_bom, src.starts_with('\u{FEFF}'), "{src:?}");
        assert!(src.ends_with(rest), "{src:?}");

        assert_eq!(
            detect_line_ending(&src),
            reference_line_ending(&src),
            "{src:?}"
        );

        let normalized = normalize_newlines(&src);
        assert_eq!(normalized, reference_normalize(&src), "{src:?}");
        assert_eq!(
            matches!(normalized, Cow::Borrowed(_)),
            !src.contains('\r'),
            "{src:?}"
        );

        for input in [src.as_str(), normalized.as_ref()] {
            let (fm, body) = split_frontmatter(input);
            assert_eq!(fm.unwrap_or("").to_owned() + body, input, "{input:?}");
            if let Some(fm) = fm {
                assert!(fm.starts_with("---\n"), "{input:?}");
                assert!(
                    fm.ends_with("\n---\n") || fm.ends_with("\n---"),
                    "{input:?}"
                );
                let _ = frontmatter_dialect(fm);
            }
        }
        let _ = frontmatter_dialect(&src);

        for max in 0..=src.len() + 1 {
            let (cut, truncated) = truncate_at_char_boundary(&src, max);
            assert!(src.starts_with(cut) && cut.len() <= max, "{src:?} {max}");
            assert_eq!(truncated, src.len() > max, "{src:?} {max}");
            let longest = src
                .char_indices()
                .map(|(i, c)| i + c.len_utf8())
                .take_while(|&end| end <= max)
                .last()
                .unwrap_or(0);
            assert_eq!(cut.len(), longest.min(src.len()), "{src:?} {max}");
        }
    }
}
