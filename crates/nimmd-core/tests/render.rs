use std::borrow::Cow;
use std::time::{Duration, Instant};

use nimmd_core::detect;
use nimmd_core::profile::Dialect;
use nimmd_core::render_fragment;
use nimmd_core::text::{MAX_NESTING, limit_nesting, split_frontmatter};

/// Значения атрибута `attr="…"` во всём фрагменте.
fn attr_values<'a>(html: &'a str, attr: &str) -> Vec<&'a str> {
    let needle = format!("{attr}=\"");
    html.match_indices(&needle)
        .filter_map(|(at, _)| html.get(at + needle.len()..)?.split('"').next())
        .collect()
}

#[test]
fn table_alignment_and_tasks() {
    let html = render_fragment("| a | b |\n|---|:-:|\n| 1 | 2 |\n\n- [x] done\n- [ ] todo\n");
    assert!(html.contains("<th align=\"center\">"), "{html}");
    assert_eq!(
        html.matches("<input type=\"checkbox\"").count(),
        2,
        "{html}"
    );
    assert_eq!(html.matches("checked").count(), 1, "{html}");
    assert_eq!(html.matches("disabled").count(), 2, "{html}");
    assert!(html.contains("task-list-item"), "{html}");
    insta::assert_snapshot!(html);
}

#[test]
fn code_highlighted_with_classes() {
    let html = render_fragment("```rust\nfn main() {}\n```\n");
    assert!(html.contains("<pre"), "{html}");
    let code = html.split_once("<code").unwrap().1;
    assert!(code.contains("class=\""), "{html}");
    assert!(!html.contains("style="), "{html}");
    insta::assert_snapshot!(html);
}

#[test]
fn code_without_or_with_unknown_language() {
    let plain = render_fragment("```\n<b>x</b>\n```\n");
    let unknown = render_fragment("```nosuchlang\nx\n```\n");
    for html in [&plain, &unknown] {
        assert!(html.contains("<pre") && html.contains("<code"), "{html}");
    }
    assert!(plain.contains("&lt;b&gt;"), "{plain}");
    assert!(!plain.contains("<b>"), "{plain}");
    assert!(unknown.contains('x'), "{unknown}");
}

#[test]
fn five_alert_types_ignore_case() {
    for (marker, kind) in [
        ("NOTE", "note"),
        ("tip", "tip"),
        ("IMPORTANT", "important"),
        ("warning", "warning"),
        ("CAUTION", "caution"),
    ] {
        let html = render_fragment(&format!("> [!{marker}]\n> текст\n"));
        assert!(
            html.contains(&format!("markdown-alert markdown-alert-{kind}")),
            "{html}"
        );
        assert!(html.contains("markdown-alert-title"), "{html}");
        assert!(html.contains("текст"), "{html}");
    }
}

#[test]
fn obsidian_callout_is_plain_blockquote() {
    let html = render_fragment("> [!abstract] заголовок\n");
    assert!(html.contains("<blockquote>"), "{html}");
    assert!(html.contains("[!abstract] заголовок"), "{html}");
    assert!(!html.contains("markdown-alert"), "{html}");
}

#[test]
fn math_and_prices() {
    let display = render_fragment("$$E=mc^2$$\n");
    assert!(display.contains("data-math-style=\"display\""), "{display}");
    let inline = render_fragment("$a+b$\n");
    assert!(inline.contains("data-math-style=\"inline\""), "{inline}");
    let prices = render_fragment("цена $5 и $10\n");
    assert!(!prices.contains("data-math-style"), "{prices}");
    assert!(prices.contains("$5") && prices.contains("$10"), "{prices}");
}

#[test]
fn mermaid_closed_unclosed_and_any_case() {
    for src in [
        "```mermaid\ngraph TD; A-->B\n```\n",
        "```mermaid\ngraph TD; A-->B\n",
        "```Mermaid\ngraph TD; A-->B\n```\n",
        "```MERMAID theme\ngraph TD; A-->B\n```\n",
    ] {
        let html = render_fragment(src);
        let pre = html
            .split_once("<pre class=\"mermaid\">")
            .unwrap_or_else(|| panic!("{html}"))
            .1;
        let pre = pre.split_once("</pre>").unwrap().0;
        assert!(pre.contains("graph TD; A--&gt;B"), "{html}");
        assert!(!pre.contains("<span") && !pre.contains("class="), "{html}");
    }
    // Слово с префиксом mermaid — не диаграмма
    assert!(!render_fragment("```mermaidx\ny\n```\n").contains("class=\"mermaid\""));
}

#[test]
fn raw_html_is_escaped() {
    for src in [
        "<script>alert(1)</script>\n",
        "a <img src=x onerror=alert(1)> b\n",
    ] {
        let html = render_fragment(src);
        assert!(!html.contains("<script"), "{html}");
        assert!(!html.contains("<img src=x"), "{html}");
        assert!(!html.contains("raw HTML omitted"), "{html}");
        assert!(html.contains("&lt;"), "{html}");
    }
}

#[test]
fn dangerous_urls_dropped_safe_images_kept() {
    let html = render_fragment(
        "[a](javascript:alert(1)) [[javascript:alert(1)]] ![i](data:text/html,x) \
         [v](vbscript:x) [f](file:///etc/passwd) <javascript:alert(1)> \
         [A](JaVaScRiPt:alert(1)) [s]( javascript:x)\n",
    );
    let urls: Vec<&str> = attr_values(&html, "href")
        .into_iter()
        .chain(attr_values(&html, "src"))
        .collect();
    for url in &urls {
        let lower = url.to_ascii_lowercase();
        for scheme in ["javascript:", "vbscript:", "file:", "data:text/html"] {
            assert!(!lower.contains(scheme), "{url} в {html}");
        }
    }
    let png = render_fragment("![p](data:image/png;base64,AAAA)\n");
    assert!(png.contains("src=\"data:image/png;base64,AAAA\""), "{png}");
    let safe = render_fragment("[ok](https://example.com/a?b=1)\n");
    assert!(
        safe.contains("href=\"https://example.com/a?b=1\""),
        "{safe}"
    );
}

#[test]
fn wikilink_and_footnotes() {
    let link = render_fragment("[[Note|текст]]\n");
    assert!(link.contains("<a href=\"Note\""), "{link}");
    assert!(link.contains(">текст</a>"), "{link}");
    let notes = render_fragment("текст[^1]\n\n[^1]: сноска\n");
    assert!(notes.contains("href=\"#fn-1\""), "{notes}");
    assert!(notes.contains("class=\"footnotes\""), "{notes}");
    insta::assert_snapshot!(notes);
}

#[test]
fn leading_rule_block_kept() {
    let src = "---\nmarkdown: gfm\n---\n---\nтекст\n---\n";
    let (_, body) = split_frontmatter(src);
    assert_eq!(body, "---\nтекст\n---\n");
    let html = render_fragment(body);
    assert!(html.contains("текст") && html.contains("<hr"), "{html}");
}

#[test]
fn detect_parses_body_after_frontmatter() {
    let profile = detect("---\nmarkdown: gfm\n---\n---\nтекст\n---\n");
    assert_eq!(profile.dialect, Dialect::Gfm);
    assert!(profile.features.frontmatter);
    // Формула и диаграмма внутри frontmatter — не признаки тела
    let inside = detect("---\ntitle: $a+b$\n---\nтекст\n");
    assert!(!inside.features.math, "{inside:?}");
    let inside = detect("---\nx: |\n  ```mermaid\n  a\n  ```\n---\nтекст\n");
    assert!(!inside.features.mermaid, "{inside:?}");
}

#[test]
fn deep_nesting_is_fast() {
    let ladder: String = (0..800)
        .map(|k| "\t".repeat(k) + &"- ".repeat(64) + "x\n")
        .collect();
    // Отступ продолжения не наращивает глубину: ни одна строка не шире 4 * MAX_NESTING колонок
    for line in limit_nesting(&ladder).lines() {
        let width = line
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .fold(
                0,
                |col, b| if b == b'\t' { col / 4 * 4 + 4 } else { col + 1 },
            );
        assert!(width <= 4 * MAX_NESTING, "{width}");
    }
    // AC-10: 1 с; «лестница» в debug на Apple Silicon занимает 0,85–0,95 с, поэтому её порог
    // поднят до 5 с — запас 5× для медленных раннеров CI (QA-004, ARCH-002)
    for (src, limit) in [
        (">".repeat(60_000) + "x", Duration::from_secs(1)),
        ("1. ".repeat(100) + "x", Duration::from_secs(1)),
        (ladder, Duration::from_secs(5)),
    ] {
        let start = Instant::now();
        let html = render_fragment(&src);
        let elapsed = start.elapsed();
        assert!(elapsed < limit, "{elapsed:?}");
        assert!(html.contains('x'));
    }
}

#[test]
fn ordered_marker_over_limit_escapes_punctuation() {
    let src = "1. ".repeat(100) + "x";
    let limited = limit_nesting(&src);
    assert_eq!(
        limited,
        "1. ".repeat(MAX_NESTING) + "1\\. " + &"1. ".repeat(100 - MAX_NESTING - 1) + "x"
    );
    assert!(!render_fragment(&src).contains('\\'));
    let paren = "1) ".repeat(MAX_NESTING + 1) + "x";
    assert_eq!(limit_nesting(&paren), "1) ".repeat(MAX_NESTING) + "1\\) x");
    assert!(!render_fragment(&paren).contains('\\'));
}

#[test]
fn leading_indent_is_capped() {
    let over = " ".repeat(300) + "- x";
    assert_eq!(limit_nesting(&over), " ".repeat(256) + "- x");
    let at_limit = " ".repeat(256) + "- x";
    assert!(matches!(limit_nesting(&at_limit), Cow::Borrowed(_)));
    let tabs = "\t".repeat(65) + "- x";
    assert_eq!(limit_nesting(&tabs), "\t".repeat(64) + "- x");
    // Табуляция после пробелов дотягивает до кратной 4 колонки
    let mixed = "  \t".repeat(70) + "- x\nb\n";
    assert_eq!(limit_nesting(&mixed), "  \t".repeat(64) + "- x\nb\n");
    let blank = " ".repeat(300) + "\n";
    assert_eq!(limit_nesting(&blank), " ".repeat(256) + "\n");
}

#[test]
fn disabled_and_enabled_extensions() {
    let dl = render_fragment("T\n: d\n");
    assert!(!dl.contains("<dl"), "{dl}");
    let heading = render_fragment("# H {#id}\n");
    assert!(!heading.contains("id="), "{heading}");
    assert!(heading.contains("{#id}"), "{heading}");
    let quote = render_fragment(">>>\nx\n>>>\n");
    assert_eq!(quote.matches("<blockquote>").count(), 1, "{quote}");
    assert!(quote.contains("<p>x</p>"), "{quote}");
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

const PIECES: [&str; 26] = [
    "```",
    "```mermaid\n",
    "~~~",
    "> ",
    "[!NOTE]",
    "- [ ] ",
    "1. ",
    "|",
    ":-:",
    "$",
    "$$",
    "[[",
    "]]",
    "[^1]",
    "<script>",
    "<img src=x>",
    "](javascript:x)",
    "\t",
    "    ",
    "\n",
    "\r\n",
    "---\n",
    "é",
    "😀",
    "{#id}",
    "\\",
];

#[test]
fn render_is_total_on_random_input() {
    let mut rng = Rng(0x00C0_FFEE_D00D_F00D);
    for _ in 0..2_000 {
        let len = rng.next() % 40;
        let src: String = (0..len)
            .map(|_| PIECES[usize::try_from(rng.next() % PIECES.len() as u64).unwrap()])
            .collect();
        let html = render_fragment(&src);
        assert!(!html.contains("<script"), "{src:?}");
        assert!(!html.contains("<img src=x"), "{src:?}");
    }
}
