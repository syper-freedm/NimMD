use syntect::easy::ScopeRangeIterator;
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use syntect::util::LinesWithEndings;

use nimmd_core::{HIGHLIGHT_LIMIT, Span, highlight};

fn utf16_len(code: &str) -> u32 {
    u32::try_from(code.encode_utf16().count()).unwrap_or(u32::MAX)
}

/// Инварианты FR-10: непустые, упорядочены, не пересекаются, максимальны, в пределах кода.
fn check_spans(spans: &[Span], code: &str) {
    let len = utf16_len(code);
    let mut prev: Option<&Span> = None;
    for span in spans {
        assert!(span.start < span.end, "{span:?}");
        assert!(span.end <= len, "{span:?} > {len}");
        assert!(!span.class.is_empty(), "{span:?}");
        let atoms: Vec<&str> = span.class.split(' ').collect();
        for (i, atom) in atoms.iter().enumerate() {
            assert!(!atom.is_empty(), "{span:?}");
            assert!(!atoms[..i].contains(atom), "повтор атома: {span:?}");
        }
        if let Some(prev) = prev {
            assert!(prev.end <= span.start, "{prev:?} {span:?}");
            assert!(
                prev.end < span.start || prev.class != span.class,
                "не максимален: {prev:?} {span:?}"
            );
        }
        prev = Some(span);
    }
}

#[test]
fn data_languages_have_spans() {
    for (lang, code, scope) in [
        ("toml", "a = 1", "source toml"),
        ("yaml", "a: 1", "source yaml"),
        ("json", "{}", "source json"),
    ] {
        let spans = highlight(lang, code);
        assert!(!spans.is_empty(), "{lang}");
        check_spans(&spans, code);
        assert!(spans[0].class.starts_with(scope), "{lang}: {spans:?}");
    }
}

#[test]
fn rust_spans_in_utf16() {
    let spans = highlight("rust", "let x = 1;");
    assert!(!spans.is_empty(), "{spans:?}");
    check_spans(&spans, "let x = 1;");
    let first = spans.iter().find(|s| s.start == 0).unwrap();
    assert!(
        first.class.contains("keyword") || first.class.contains("storage"),
        "{first:?}"
    );
    assert!(first.class.starts_with("source rust"), "{first:?}");
    assert_eq!(first.end, 3, "{spans:?}");

    let code = "let ё = \"ж\";";
    assert_eq!((code.len(), utf16_len(code)), (14, 12));
    let spans = highlight("rust", code);
    assert!(!spans.is_empty(), "{spans:?}");
    check_spans(&spans, code);
    assert!(spans.last().unwrap().end <= 12, "{spans:?}");
    // Строка `"ж"` — UTF-16 смещения 8..11, в байтах было бы 10..14
    let string: Vec<(u32, u32)> = spans
        .iter()
        .filter(|s| s.class.contains("string"))
        .map(|s| (s.start, s.end))
        .collect();
    assert_eq!(string, [(8, 9), (9, 10), (10, 11)], "{spans:?}");
    assert_eq!(spans.last().unwrap().start, 11, "{spans:?}");
    // Символы вне BMP — две единицы UTF-16
    let code = "// \u{1F600}\nx";
    let spans = highlight("rust", code);
    check_spans(&spans, code);
    assert_eq!(spans.last().unwrap().end, utf16_len(code), "{spans:?}");
}

#[test]
fn unknown_and_empty_language() {
    let spans = highlight("nosuchlang", "x");
    assert!(!spans.is_empty(), "{spans:?}");
    check_spans(&spans, "x");
    assert!(spans.iter().all(|s| s.class == "text plain"), "{spans:?}");
    assert_eq!(
        highlight("", "x\ny"),
        vec![Span {
            start: 0,
            end: 3,
            class: "text plain".to_owned()
        }]
    );
    assert_eq!(highlight("", ""), Vec::new());
    assert_eq!(highlight("rust", ""), Vec::new());
    assert_eq!(highlight("nosuchlang", ""), Vec::new());
    // Токен — первое слово до `,`; без грамматики — по первой строке
    assert_eq!(
        highlight("rust,ignore", "fn a() {}"),
        highlight("rust", "fn a() {}")
    );
    let shell = highlight("nosuchlang", "#!/bin/bash\necho 1\n");
    assert!(shell.iter().any(|s| s.class.contains("shell")), "{shell:?}");
    for lang in [",", ",rust", " ", "\0", "ж"] {
        check_spans(&highlight(lang, "a\n"), "a\n");
    }
}

#[test]
fn large_code_highlights_only_head() {
    let code = "x = 1\n".repeat(3 * 1024 * 1024 / 6);
    let spans = highlight("nosuchlang", &code);
    assert!(!spans.is_empty(), "{spans:?}");
    check_spans(&spans, &code);
    let last = spans.last().unwrap().end;
    assert!(last <= u32::try_from(HIGHLIGHT_LIMIT).unwrap(), "{last}");
    // Голова — до последнего `\n` в пределах лимита
    assert_eq!(last, u32::try_from(HIGHLIGHT_LIMIT / 6 * 6).unwrap());

    // Без `\n`: голова до границы символа; лимит попадает в середину `ж`
    let line = format!("\"{}", "ж".repeat(HIGHLIGHT_LIMIT));
    assert!(!line.is_char_boundary(HIGHLIGHT_LIMIT));
    let spans = highlight("nosuchlang", &line);
    let last = spans.last().unwrap().end;
    assert_eq!(last, utf16_len(&line[..HIGHLIGHT_LIMIT - 1]), "{last}");

    let exact = "a".repeat(HIGHLIGHT_LIMIT);
    let spans = highlight("nosuchlang", &exact);
    assert_eq!(spans.last().unwrap().end, utf16_len(&exact));
}

/// Эталон FR-10 по стандартному набору syntect (дамп ядра — он же плюс TOML).
fn reference(set: &SyntaxSet, lang: &str, code: &str) -> Option<Vec<Span>> {
    let syntax = set
        .find_syntax_by_token(lang)
        .or_else(|| set.find_syntax_by_first_line(code))
        .or_else(|| set.find_syntax_by_name("Plain Text"))?;
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut out: Vec<Span> = Vec::new();
    let mut offset = 0;
    for line in LinesWithEndings::from(code) {
        let ops = state.parse_line(line, set).ok()?;
        for (range, op) in ScopeRangeIterator::new(&ops, line) {
            stack.apply(op).ok()?;
            let text = line.get(range)?;
            let start = offset;
            offset += utf16_len(text);
            if !text.is_empty() && !stack.is_empty() {
                let mut atoms: Vec<String> = Vec::new();
                for scope in stack.as_slice() {
                    for atom in scope.build_string().split('.') {
                        if !atoms.iter().any(|a| a == atom) {
                            atoms.push(atom.to_owned());
                        }
                    }
                }
                let class = atoms.join(" ");
                match out.last_mut() {
                    Some(last) if last.end == start && last.class == class => last.end = offset,
                    _ => out.push(Span {
                        start,
                        end: offset,
                        class,
                    }),
                }
            }
        }
    }
    Some(out)
}

#[test]
fn random_code_matches_reference() {
    let set = SyntaxSet::load_defaults_newlines();
    let alphabet = [
        "fn",
        "let",
        " ",
        "x",
        "1",
        "\"",
        "'",
        "{",
        "}",
        "(",
        ")",
        "[",
        "]",
        ":",
        ",",
        ";",
        "#",
        "//",
        "/*",
        "*/",
        "\n",
        "ж",
        "\u{1F600}",
        "<",
        ">",
        "=",
        "-",
        "\\",
        "true",
        "null",
    ];
    let langs = [
        "rust",
        "json",
        "yaml",
        "python",
        "js",
        "html",
        "sh",
        "nosuchlang",
    ];
    let mut seed: u64 = 0xA076_1D64_78BD_642F;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        usize::try_from(seed % 1_000_003).unwrap()
    };
    for _ in 0..400 {
        let lang = langs[next() % langs.len()];
        let len = next() % 50;
        let code: String = (0..len)
            .map(|_| alphabet[next() % alphabet.len()])
            .collect();
        let spans = highlight(lang, &code);
        check_spans(&spans, &code);
        assert_eq!(
            Some(spans),
            reference(&set, lang, &code),
            "{lang}: {code:?}"
        );
    }
}

#[test]
fn span_serializes_to_json() {
    let span = Span {
        start: 1,
        end: 2,
        class: "source rust".to_owned(),
    };
    assert_eq!(
        serde_json::to_string(&span).unwrap(),
        r#"{"start":1,"end":2,"class":"source rust"}"#
    );
}
