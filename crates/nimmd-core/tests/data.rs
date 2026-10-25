use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use nimmd_core::profile::Profile;
use nimmd_core::text::{LineEnding, normalize_newlines};
use nimmd_core::{AssetMode, FileKind, HIGHLIGHT_LIMIT, RenderOptions, Rendered, render};

const MIB: usize = 1024 * 1024;
const DATA_CSS: &str = include_str!("../../../packages/theme/data.css");
const LIMIT_BANNER: &str = "<div class=\"banner\">Подсветка отключена после 2 МБ</div>";

fn of(kind: FileKind) -> RenderOptions {
    RenderOptions {
        kind,
        ..RenderOptions::default()
    }
}

fn data(src: &str, kind: FileKind) -> Rendered {
    render(src, &of(kind))
}

/// Содержимое `<pre class="data">`.
fn pre(body: &str) -> &str {
    let inner = body
        .split_once("<pre class=\"data\">")
        .and_then(|(_, rest)| rest.split_once("</pre>"));
    assert!(inner.is_some(), "нет <pre class=\"data\">: {body}");
    inner.map_or("", |(pre, _)| pre)
}

/// Видимый текст HTML: без тегов, сущности раскрыты.
fn visible(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;
    while let Some((before, after)) = rest.split_once('<') {
        text.push_str(before);
        assert!(after.contains('>'), "незакрытый тег: {html}");
        rest = after.split_once('>').map_or("", |(_, tail)| tail);
    }
    text.push_str(rest);
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
}

/// Значения `class="…"` во фрагменте.
fn classes(html: &str) -> BTreeSet<&str> {
    html.match_indices("class=\"")
        .filter_map(|(at, m)| html.get(at + m.len()..)?.split('"').next())
        .collect()
}

#[test]
fn file_kind_from_extension() {
    let cases = [
        ("YML", Some(FileKind::Yaml)),
        ("markdown", Some(FileKind::Markdown)),
        ("Json", Some(FileKind::Json)),
        ("txt", Some(FileKind::Text)),
        ("exe", None),
        ("", None),
        (".md", None),
    ];
    for (ext, kind) in cases {
        assert_eq!(FileKind::from_extension(ext), kind, "{ext:?}");
    }
    for (ext, kind) in [
        ("md", FileKind::Markdown),
        ("MD", FileKind::Markdown),
        ("mdown", FileKind::Markdown),
        ("mkd", FileKind::Markdown),
        ("toml", FileKind::Toml),
        ("TOML", FileKind::Toml),
        ("yaml", FileKind::Yaml),
        ("yml", FileKind::Yaml),
        ("CSV", FileKind::Csv),
    ] {
        assert_eq!(FileKind::from_extension(ext), Some(kind), "{ext:?}");
    }
    for ext in ["md ", "json.bak", "tsv", "yam", "текст"] {
        assert_eq!(FileKind::from_extension(ext), None, "{ext:?}");
    }
    assert_eq!(FileKind::default(), FileKind::Markdown);
    assert_eq!(RenderOptions::default().kind, FileKind::Markdown);
}

#[test]
fn file_kind_serde_lowercase() {
    assert_eq!(serde_json::to_string(&FileKind::Json).unwrap(), "\"json\"");
    assert_eq!(
        serde_json::from_str::<FileKind>("\"json\"").unwrap(),
        FileKind::Json
    );
    for (kind, name) in [
        (FileKind::Markdown, "markdown"),
        (FileKind::Toml, "toml"),
        (FileKind::Yaml, "yaml"),
        (FileKind::Csv, "csv"),
        (FileKind::Text, "text"),
    ] {
        let json = format!("\"{name}\"");
        assert_eq!(serde_json::to_string(&kind).unwrap(), json);
        assert_eq!(serde_json::from_str::<FileKind>(&json).unwrap(), kind);
    }
    assert!(serde_json::from_str::<FileKind>("\"Json\"").is_err());
}

#[test]
fn toml_yaml_json_highlighted_in_data_wrapper() {
    let cases = [
        ("[server]\nport = 8080\n", FileKind::Toml, "toml"),
        ("a: &x 1\nb: *x\n", FileKind::Yaml, "yaml"),
        ("{\"a\": [1, true, null]}\n", FileKind::Json, "json"),
    ];
    for (src, kind, name) in cases {
        let out = data(src, kind);
        let main = format!("<main class=\"nimmd nimmd-data\" data-kind=\"{name}\">");
        assert!(out.body.starts_with(&main), "{}", out.body);
        assert!(out.body.ends_with("</pre></main>"), "{}", out.body);
        assert!(out.html.contains(&out.body), "{name}");
        let pre = pre(&out.body);
        assert!(pre.contains("<span class=\""), "{pre}");
        assert!(
            classes(pre).iter().any(|c| c.contains(name)),
            "{name}: {pre}"
        );
        assert_eq!(visible(pre), src, "{name}");
        assert_eq!(out.html.matches("<style").count(), 1, "{name}");
        let style = out
            .html
            .split_once("<style>")
            .unwrap()
            .1
            .split_once("</style>")
            .unwrap()
            .0;
        assert!(style.contains(".c0 {"), "{name}");
        assert!(style.contains(DATA_CSS), "{name}");
        assert!(!out.highlight_limited);
        assert!(!out.body.contains("class=\"banner"), "{}", out.body);
    }
    let toml = data("[server]\nport = 8080\n", FileKind::Toml);
    assert!(
        pre(&toml.body).contains("<span class=\"source toml"),
        "{}",
        toml.body
    );
}

#[test]
fn markdown_has_no_data_css() {
    let out = render("# x\n", &RenderOptions::default());
    assert!(
        out.body.starts_with("<main class=\"nimmd\">"),
        "{}",
        out.body
    );
    assert!(!out.html.contains("nimmd-data"), "{}", out.html);
    assert!(!out.html.contains("--csv-0"), "{}", out.html);
    assert!(!out.highlight_limited);
}

#[test]
fn json_file_classes_match_markdown_block() {
    let src = "{\"a\": [1, true, null]}\n";
    let file = data(src, FileKind::Json);
    let md = render(
        &format!("```json\n{src}```\n\n```toml\na = 1\n```\n"),
        &RenderOptions::default(),
    );
    let blocks: Vec<&str> = md
        .body
        .split("<pre class=\"syntax-highlighting\">")
        .skip(1)
        .map(|b| {
            let code = b.split_once("</code>").unwrap().0;
            code.split_once('>').unwrap().1
        })
        .collect();
    assert_eq!(blocks.len(), 2, "{}", md.body);
    assert_eq!(classes(pre(&file.body)), classes(blocks[0]), "{}", md.body);
    assert_eq!(visible(blocks[0]), src);
    assert!(
        blocks[1].contains("<span class=\"source toml"),
        "{}",
        md.body
    );
}

#[test]
fn text_escaped_without_highlight() {
    let long = "ш".repeat(250);
    let src = format!("<b>не тег</b>\tтаб\n{long}\n");
    let out = data(&src, FileKind::Text);
    assert!(
        out.body
            .starts_with("<main class=\"nimmd nimmd-data\" data-kind=\"text\">"),
        "{}",
        out.body
    );
    assert!(
        out.body.contains("&lt;b&gt;не тег&lt;/b&gt;\tтаб"),
        "{}",
        out.body
    );
    assert!(!out.body.contains("<b>"), "{}", out.body);
    assert!(!out.body.contains("<span"), "{}", out.body);
    assert_eq!(visible(pre(&out.body)), src);
    assert!(!out.highlight_limited);
    let rule = DATA_CSS
        .split_once("[data-kind=\"text\"]")
        .unwrap_or_else(|| panic!("{DATA_CSS}"))
        .1
        .split_once('}')
        .unwrap()
        .0;
    assert!(rule.contains("white-space: pre-wrap"), "{rule}");
}

#[test]
fn data_has_no_profile_frontmatter_or_scripts() {
    let src = "---\nmarkdown: obsidian\nmath: $$x$$\n";
    let assets = [
        AssetMode::Link {
            base_url: "assets".to_owned(),
        },
        AssetMode::Inline {
            katex_css: "K_CSS".to_owned(),
            katex_js: "K_JS".to_owned(),
            mermaid_js: "M_JS".to_owned(),
        },
    ];
    let srcs = [
        (src, FileKind::Yaml),
        (
            "# Заголовок\n$$x$$\n```mermaid\ngraph TD\n```\n",
            FileKind::Text,
        ),
        ("a = \"$$x$$\"\n", FileKind::Toml),
        ("\"# t\", $x$\n", FileKind::Csv),
    ];
    for assets in assets {
        for (src, kind) in srcs {
            let opts = RenderOptions {
                assets: assets.clone(),
                profile: None,
                ..of(kind)
            };
            let out = render(src, &opts);
            assert_eq!(out.profile, Profile::default(), "{kind:?}");
            assert_eq!(out.frontmatter, None, "{kind:?}");
            assert_eq!(out.title, None, "{kind:?}");
            assert!(!out.html.contains("<script"), "{}", out.html);
            assert!(!out.html.contains("<link"), "{}", out.html);
            assert!(!out.html.contains("<title"), "{}", out.html);
            assert!(!out.html.contains("K_JS"), "{}", out.html);
            assert!(!out.body.contains("frontmatter"), "{}", out.body);
            assert_eq!(visible(pre(&out.body)), src, "{kind:?}");
        }
    }
}

#[test]
fn highlight_limit_splits_head_and_tail() {
    let line = "{\"k\": 1}\n";
    let src = line.repeat(HIGHLIGHT_LIMIT / line.len() + 1000);
    let out = data(&src, FileKind::Json);
    assert!(out.highlight_limited);
    assert!(out.body.contains(LIMIT_BANNER));
    assert!(
        out.body.contains(&format!("</pre>{LIMIT_BANNER}</main>")),
        "баннер сразу после </pre>"
    );
    assert!(out.body.contains("\n{&quot;k&quot;: 1}\n</pre>"));
    let shown = pre(&out.body);
    assert_eq!(visible(shown), src);
    // Хвост после последнего тега — экранированный текст без подсветки
    let tail = shown.rsplit_once('>').unwrap().1;
    assert!(tail.len() >= 1000 * 15, "хвост: {} байт", tail.len());
    // Голова кончается на `\n` в пределах лимита: хвост начинается с целой строки
    let head_len = src.len() - visible(tail).len();
    assert!(head_len <= HIGHLIGHT_LIMIT, "{head_len}");
    assert_eq!(head_len % line.len(), 0, "{head_len}");
    assert!(head_len > HIGHLIGHT_LIMIT - line.len(), "{head_len}");

    let csv_line = "a,b\n";
    let csv = csv_line.repeat(HIGHLIGHT_LIMIT / csv_line.len() + 1000);
    let out = data(&csv, FileKind::Csv);
    assert!(out.highlight_limited);
    assert!(out.body.contains(LIMIT_BANNER));
    assert!(out.body.contains("\na,b\n</pre>"));
    assert!(out.body.contains("<span class=\"c1\">b</span>"));
    assert_eq!(visible(pre(&out.body)), csv);

    let text = data(&src, FileKind::Text);
    assert!(!text.highlight_limited);
    assert!(!text.body.contains("Подсветка отключена"));
    assert_eq!(visible(pre(&text.body)), src);

    let small = data("{}\n", FileKind::Json);
    assert!(!small.highlight_limited);
    assert!(
        !small.body.contains("Подсветка отключена"),
        "{}",
        small.body
    );
}

#[test]
fn highlight_limit_exact_boundary() {
    // Ровно лимит — подсвечен целиком; на байт больше — уже с хвостом
    let exact = format!("{}\n", "1".repeat(HIGHLIGHT_LIMIT - 1));
    assert_eq!(exact.len(), HIGHLIGHT_LIMIT);
    assert!(!data(&exact, FileKind::Json).highlight_limited);
    let over = format!("{exact}2");
    let out = data(&over, FileKind::Json);
    assert!(out.highlight_limited);
    assert!(
        out.body.contains("</span>2</pre>"),
        "хвост — один символ без подсветки"
    );
    assert_eq!(visible(pre(&out.body)), over);
}

#[test]
fn long_line_without_newline_cut_at_char_boundary() {
    // `"` + `ж`…: предел 2 МБ попадает в середину двухбайтового символа
    let src = format!("\"{}\"", "ж".repeat(HIGHLIGHT_LIMIT / 2 + 100));
    assert!(!src.is_char_boundary(HIGHLIGHT_LIMIT));
    let out = data(&src, FileKind::Json);
    assert!(out.highlight_limited);
    assert!(out.body.contains(LIMIT_BANNER));
    assert_eq!(visible(pre(&out.body)), src);
    let csv = data(&src, FileKind::Csv);
    assert!(csv.highlight_limited);
    assert_eq!(visible(pre(&csv.body)), src);
}

#[test]
fn not_utf8_bom_crlf_and_truncation() {
    let opts = RenderOptions {
        not_utf8: true,
        ..of(FileKind::Toml)
    };
    let out = render("a = \"\u{FFFD}\"\n", &opts);
    assert!(
        out.body.starts_with(
            "<main class=\"nimmd nimmd-data\" data-kind=\"toml\"><div class=\"banner not-utf8\">"
        ),
        "{}",
        out.body
    );
    assert!(out.body.find("not-utf8").unwrap() < out.body.find("<pre").unwrap());

    let out = data("\u{FEFF}a = 1\r\nb = 2\r\n", FileKind::Toml);
    assert!(out.had_bom);
    assert_eq!(out.line_ending, LineEnding::CrLf);
    assert!(!out.html.contains('\r'), "{}", out.body);
    assert!(!out.body.contains('\u{FEFF}'), "{}", out.body);
    assert_eq!(visible(pre(&out.body)), "a = 1\nb = 2\n");

    let line = "key = \"значение\"\n";
    let src = line.repeat(3 * MIB / line.len());
    let opts = RenderOptions {
        max_bytes: MIB,
        ..of(FileKind::Toml)
    };
    let out = render(&src, &opts);
    assert!(out.truncated);
    assert!(!out.highlight_limited);
    assert!(
        out.body.ends_with(
            "</pre><div class=\"banner truncated\">Файл обрезан: показаны первые 1 МБ</div></main>"
        ),
        "{}",
        out.body.get(out.body.len() - 200..).unwrap_or_default()
    );
    let shown = visible(pre(&out.body));
    assert!(
        shown.len() <= MIB && shown.len() > MIB - 4,
        "{}",
        shown.len()
    );
    assert!(src.starts_with(&shown));
}

#[test]
fn csv_colored_by_column() {
    let out = data("a,b,c,d,e,f,g,h,i\n", FileKind::Csv);
    assert!(
        out.body
            .starts_with("<main class=\"nimmd nimmd-data\" data-kind=\"csv\">"),
        "{}",
        out.body
    );
    let pre = pre(&out.body);
    for frag in [
        "<span class=\"c0\">a</span>",
        "<span class=\"c7\">h</span>",
        "<span class=\"c0\">i</span>",
        "<span class=\"d\">,</span>",
    ] {
        assert!(pre.contains(frag), "{frag}: {pre}");
    }
    assert_eq!(
        pre,
        "<span class=\"c0\">a</span><span class=\"d\">,</span><span class=\"c1\">b</span>\
         <span class=\"d\">,</span><span class=\"c2\">c</span><span class=\"d\">,</span>\
         <span class=\"c3\">d</span><span class=\"d\">,</span><span class=\"c4\">e</span>\
         <span class=\"d\">,</span><span class=\"c5\">f</span><span class=\"d\">,</span>\
         <span class=\"c6\">g</span><span class=\"d\">,</span><span class=\"c7\">h</span>\
         <span class=\"d\">,</span><span class=\"c0\">i</span>\n"
    );
}

#[test]
fn csv_cases_keep_text() {
    let cases: [(&str, &[&str], &[&str]); 7] = [
        ("a,b,c,d,e,f,g,h,i\n", &["<span class=\"c8"], &[]),
        (
            "a,\"x\ny\",c\n",
            &[],
            &[
                "<span class=\"c1\">&quot;x\ny&quot;</span>",
                "<span class=\"c2\">c</span>",
            ],
        ),
        (
            "a,\"q\"\"q\",b\n",
            &[],
            &["<span class=\"c1\">&quot;q&quot;&quot;q&quot;</span>"],
        ),
        (
            "a,b\n1,2,3,4,5\nx\n",
            &[],
            &[
                "<span class=\"c4\">5</span>",
                "\n<span class=\"c0\">x</span>\n",
            ],
        ),
        (
            "a,\"b\nc,d\n",
            &[],
            &["<span class=\"c1\">&quot;b\nc,d\n</span>"],
        ),
        (
            "a,,b\n",
            &["class=\"c1\""],
            &["<span class=\"d\">,</span><span class=\"d\">,</span><span class=\"c2\">b</span>"],
        ),
        ("<x>,&\n", &["<x>"], &["&lt;x&gt;", "&amp;"]),
    ];
    for (src, absent, present) in cases {
        let out = data(src, FileKind::Csv);
        let pre = pre(&out.body);
        for frag in present {
            assert!(pre.contains(frag), "{src:?}: {frag} в {pre}");
        }
        for frag in absent {
            assert!(!pre.contains(frag), "{src:?}: {frag} в {pre}");
        }
        assert_eq!(visible(pre), src, "{src:?}");
        assert!(!out.highlight_limited);
    }
}

#[test]
fn csv_edge_inputs_keep_text() {
    for src in [
        "",
        "\n",
        "\n\n",
        ",",
        ",,\n",
        "\"",
        "\"\"",
        "\"\"\"",
        "a\"b,c\n",
        "\"a\"b,c\n",
        "a,b",
        "ж;ё;\u{1F600}\n1;2;3\n",
        "a|b\n1|2\n",
        "a\tb\n",
    ] {
        let out = data(src, FileKind::Csv);
        assert_eq!(visible(pre(&out.body)), src, "{src:?}");
    }
    let out = data("a,b\r\n1,2\r\n", FileKind::Csv);
    assert_eq!(out.line_ending, LineEnding::CrLf);
    assert_eq!(visible(pre(&out.body)), "a,b\n1,2\n");
}

#[test]
fn csv_fixtures_keep_text() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/data");
    let mut seen = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        let out = data(&src, FileKind::Csv);
        let pre = pre(&out.body);
        assert_eq!(visible(pre), normalize_newlines(&src), "{}", path.display());
        assert!(pre.contains("<span class=\"c1\">"), "{}", path.display());
        let name = path.file_name().unwrap().to_str().unwrap();
        match name {
            "semicolon.csv" => {
                assert!(pre.contains("<span class=\"d\">;</span>"), "{pre}");
                assert!(!pre.contains("<span class=\"d\">,</span>"), "{pre}");
                assert!(pre.contains("<span class=\"c1\">1,5</span>"), "{pre}");
            }
            "tabs.csv" => assert!(pre.contains("<span class=\"d\">\t</span>"), "{pre}"),
            "quoted.csv" => assert!(
                pre.contains("<span class=\"c1\">&quot;Smith, John&quot;</span>"),
                "{pre}"
            ),
            _ => {}
        }
        seen += 1;
    }
    assert!(seen >= 4, "фикстур: {seen}");
}

/// Детерминированный генератор (xorshift) для случайных входов.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn text(&mut self, alphabet: &[&str], len: usize) -> String {
        (0..len)
            .map(|_| {
                let i = usize::try_from(self.next() % alphabet.len() as u64).unwrap_or_default();
                alphabet[i]
            })
            .collect()
    }
}

#[test]
fn random_data_keeps_text() {
    let alphabet = [
        "a",
        "1",
        ",",
        ";",
        "\t",
        "|",
        "\"",
        "\"\"",
        "\n",
        "\r\n",
        "\r",
        "<",
        ">",
        "&",
        "'",
        "ж",
        "\u{1F600}",
        " ",
        "{",
        "}",
        "[",
        "=",
        "#",
        ":",
    ];
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..300 {
        let len = usize::try_from(rng.next() % 60).unwrap();
        let src = rng.text(&alphabet, len);
        let expected = normalize_newlines(&src);
        for kind in [
            FileKind::Csv,
            FileKind::Text,
            FileKind::Json,
            FileKind::Toml,
            FileKind::Yaml,
        ] {
            let out = data(&src, kind);
            assert_eq!(visible(pre(&out.body)), expected, "{kind:?}: {src:?}");
            assert!(!out.html.contains("<script"), "{kind:?}: {src:?}");
            assert_eq!(out.profile, Profile::default());
        }
    }
}
