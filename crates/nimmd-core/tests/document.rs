use std::fs;
use std::path::Path;

use nimmd_core::profile::{Dialect, Features, Profile};
use nimmd_core::text::{LineEnding, MAX_NESTING, normalize_newlines, strip_bom};
use nimmd_core::{AssetMode, MathMode, RenderOptions, Rendered, Theme, detect, render};

const MIB: usize = 1024 * 1024;
const NIMMD_CSS: &str = include_str!("../../../packages/theme/nimmd.css");
const CONTRACT: &str = include_str!("../../../packages/theme/CONTRACT.md");

fn opts() -> RenderOptions {
    RenderOptions::default()
}

fn with_math(math: MathMode) -> RenderOptions {
    RenderOptions { math, ..opts() }
}

fn with_assets(assets: AssetMode) -> RenderOptions {
    RenderOptions { assets, ..opts() }
}

fn link(base_url: &str) -> AssetMode {
    AssetMode::Link {
        base_url: base_url.to_owned(),
    }
}

/// Имена тегов (`<имя` и `</имя`) в HTML.
fn tag_names(html: &str) -> Vec<&str> {
    html.split('<')
        .skip(1)
        .map(|rest| {
            let rest = rest.strip_prefix('/').unwrap_or(rest);
            let end = rest
                .find(|c: char| !c.is_ascii_alphanumeric())
                .unwrap_or(rest.len());
            &rest[..end]
        })
        .collect()
}

#[test]
fn empty_and_blank_input() {
    for src in ["", "  \n\n "] {
        let out = render(src, &opts());
        assert!(out.html.starts_with("<!doctype html>"), "{}", out.html);
        assert!(
            out.html.contains("<main class=\"nimmd\"></main>"),
            "{}",
            out.html
        );
        assert_eq!(out.body, "<main class=\"nimmd\"></main>", "{src:?}");
        assert_eq!(out.title, None);
        assert!(!out.truncated && !out.had_bom);
        assert_eq!(out.frontmatter, None);
        assert!(!out.html.contains("<title>"), "{}", out.html);
        assert!(out.html.ends_with("</body></html>"), "{}", out.html);
    }
}

#[test]
fn bom_crlf_frontmatter_and_title() {
    let out = render("\u{FEFF}---\r\ntitle: X\r\n---\r\n# Заголовок\r\n", &opts());
    assert_eq!(out.frontmatter.as_deref(), Some("---\ntitle: X\n---\n"));
    assert!(out.had_bom);
    assert_eq!(out.line_ending, LineEnding::CrLf);
    let pre = out
        .body
        .split_once("<pre class=\"frontmatter\">")
        .unwrap_or_else(|| panic!("{}", out.body))
        .1
        .split_once("</pre>")
        .unwrap()
        .0;
    assert!(pre.contains("title"), "{pre}");
    assert!(pre.contains("class=\""), "{pre}");
    // Разделители `---` в подсветку не входят
    assert!(!pre.contains("---"), "{pre}");
    assert_eq!(out.title.as_deref(), Some("Заголовок"));
    assert!(
        out.html.contains("<title>Заголовок</title>"),
        "{}",
        out.html
    );
    assert!(!out.html.contains('\r'), "{}", out.html);
    // Порядок в `<main>`: frontmatter, затем фрагмент
    let fm = out.body.find("class=\"frontmatter\"").unwrap();
    let h1 = out.body.find("<h1").unwrap();
    assert!(fm < h1, "{}", out.body);
}

#[test]
fn frontmatter_text_is_escaped() {
    let out = render("---\na: <script>x</script> & y\n---\ntext\n", &opts());
    assert!(!out.body.contains("<script"), "{}", out.body);
    assert!(out.body.contains("&lt;"), "{}", out.body);
    assert!(out.body.contains("&amp;"), "{}", out.body);
}

#[test]
fn title_from_first_level_one_heading() {
    let cases = [
        (
            "## не тот\n\n# Первый *em* `code`\n\n# Второй\n",
            Some("Первый em code"),
        ),
        ("Setext\n===\n", Some("Setext")),
        ("## Только второй уровень\n", None),
        ("текст\n", None),
        ("#\n", None),
    ];
    for (src, title) in cases {
        let out = render(src, &opts());
        assert_eq!(out.title.as_deref(), title, "{src:?}");
        assert_eq!(out.html.contains("<title>"), title.is_some(), "{src:?}");
    }
    // `\<` — текст, а не HTML-тег
    let out = render("# a \\<b\\> & \"c\"\n", &opts());
    assert_eq!(out.title.as_deref(), Some("a <b> & \"c\""));
    let head = out.html.split_once("</head>").unwrap().0;
    assert!(
        head.contains("<title>a &lt;b&gt; &amp; &quot;c&quot;</title>"),
        "{head}"
    );
}

#[test]
fn theme_attribute_and_single_style() {
    for (theme, attr) in [
        (Theme::Dark, "dark"),
        (Theme::Light, "light"),
        (Theme::Auto, "auto"),
    ] {
        let out = render("x", &RenderOptions { theme, ..opts() });
        assert!(
            out.html.starts_with(&format!(
                "<!doctype html><html data-theme=\"{attr}\"><head>"
            )),
            "{}",
            out.html
        );
        assert_eq!(out.html.matches("<style").count(), 1, "{attr}");
        let style = out
            .html
            .split_once("<style>")
            .unwrap()
            .1
            .split_once("</style>")
            .unwrap()
            .0;
        assert!(style.contains("prefers-color-scheme: dark"), "{attr}");
        assert!(out.html.contains("<meta charset=\"utf-8\">"));
        assert!(
            out.html.contains(
                "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
            )
        );
    }
    assert_eq!(render("x", &opts()).html, {
        let auto = RenderOptions {
            theme: Theme::Auto,
            ..opts()
        };
        render("x", &auto).html
    });
}

#[test]
fn body_is_substring_and_profile_independent() {
    let src = "[[Note]] и $a+b$\n\n- [x] done\n";
    let obsidian = Profile {
        dialect: Dialect::Obsidian,
        features: Features {
            wikilinks: true,
            math: true,
            ..Features::default()
        },
    };
    let mut bodies = Vec::new();
    for profile in [Profile::default(), obsidian] {
        let out = render(
            src,
            &RenderOptions {
                profile: Some(profile),
                ..opts()
            },
        );
        assert!(out.html.contains(&out.body), "{}", out.html);
        assert!(
            out.body.starts_with("<main class=\"nimmd\">"),
            "{}",
            out.body
        );
        assert!(out.body.ends_with("</main>"), "{}", out.body);
        assert_eq!(out.html.matches("<main").count(), 1, "{}", out.html);
        assert_eq!(out.profile, profile);
        bodies.push(out.body);
    }
    assert_eq!(bodies[0], bodies[1]);
}

#[test]
fn profile_matches_detect_on_fixtures() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;
    for group in fs::read_dir(&root).unwrap() {
        for entry in fs::read_dir(group.unwrap().path()).unwrap() {
            let md = entry.unwrap().path();
            if md.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            let name = md.strip_prefix(&root).unwrap().display().to_string();
            let src = fs::read_to_string(&md).unwrap();
            let expected = detect(&normalize_newlines(strip_bom(&src).1));
            assert_eq!(render(&src, &opts()).profile, expected, "{name}");
            // Тот же текст с BOM и CRLF
            let crlf = format!("\u{FEFF}{}", src.replace('\n', "\r\n"));
            assert_eq!(render(&crlf, &opts()).profile, expected, "{name} (crlf)");
            checked += 1;
        }
    }
    assert!(checked > 0, "в {} нет фикстур", root.display());
}

#[test]
fn profile_matches_detect_on_random_input() {
    let mut rng = Rng(0x0BAD_5EED_1234_5678);
    for _ in 0..1_500 {
        let src = random_src(&mut rng);
        let out = render(&src, &opts());
        assert_eq!(
            out.profile,
            detect(&normalize_newlines(strip_bom(&src).1)),
            "{src:?}"
        );
    }
}

#[test]
fn explicit_profile_wins_over_detector() {
    let gfm = Profile {
        dialect: Dialect::Gfm,
        features: Features::default(),
    };
    let out = render(
        "$$x$$\n",
        &RenderOptions {
            profile: Some(gfm),
            assets: link("assets"),
            ..opts()
        },
    );
    assert_eq!(out.profile, gfm);
    // Ресурсы — по переданному профилю, а не по найденной формуле
    assert!(!out.html.contains("katex"), "{}", out.html);
}

const IMAGES: &str = "![a](https://x/y.png)\n\n![b](HTTP://x/z.png)\n\n![c](//x/w.png)\n\n\
                      ![d][r]\n\n[r]: https://x/r.png\n\n![f](https:x/y.png)\n\n\
                      ![g](HTTP:/x/y.png)\n\n![e](local.png)\n\n[l](https://x)\n";

#[test]
fn remote_images_replaced_unless_allowed() {
    let blocked = render(IMAGES, &opts()).body;
    assert_eq!(blocked.matches("<img").count(), 1, "{blocked}");
    assert!(blocked.contains("<img src=\"local.png\""), "{blocked}");
    assert_eq!(
        blocked.matches("<span class=\"remote-image\">").count(),
        6,
        "{blocked}"
    );
    for url in [
        "https://x/y.png",
        "HTTP://x/z.png",
        "//x/w.png",
        "https://x/r.png",
        "https:x/y.png",
        "HTTP:/x/y.png",
    ] {
        assert!(
            blocked.contains(&format!("<span class=\"remote-image\">{url}</span>")),
            "{url}: {blocked}"
        );
    }
    assert!(blocked.contains("<a href=\"https://x\""), "{blocked}");
    // Альт-текст удалённой картинки не выводится отдельно
    assert!(!blocked.contains(">a<"), "{blocked}");

    let allowed = render(
        IMAGES,
        &RenderOptions {
            remote_images: true,
            ..opts()
        },
    )
    .body;
    assert_eq!(allowed.matches("<img").count(), 7, "{allowed}");
    assert!(!allowed.contains("remote-image"), "{allowed}");
    assert!(allowed.contains("<a href=\"https://x\""), "{allowed}");
}

#[test]
fn remote_image_edge_urls() {
    for (src, remote) in [
        ("![x](hTtPs://h/p.png)", true),
        ("![x](<  https://h/p.png>)", true),
        ("![x](/local/p.png)", false),
        ("![x](./p.png)", false),
        ("![x](httpx://h/p.png)", false),
        ("![x](data:image/png;base64,AAAA)", false),
        ("[![x](https://h/i.png)](https://h)", true),
    ] {
        let body = render(src, &opts()).body;
        assert_eq!(body.contains("remote-image"), remote, "{src}: {body}");
        if remote {
            assert!(!body.contains("<img"), "{src}: {body}");
        }
    }
    // Адрес выводится экранированным текстом
    let body = render("![x](https://h/a?b=1&c=<d>)", &opts()).body;
    assert!(!body.contains("<d>"), "{body}");
    assert!(body.contains("remote-image"), "{body}");
}

const FORMULAS: &str = "$x^2$\n\n$$\\frac{a}{b}$$\n";
const DANGEROUS: [&str; 4] = [
    "$\\text{<script>}$",
    "$a<b$",
    "$a&b$",
    "$\\mathrm{<img src=x onerror=y>}$",
];

#[test]
fn math_katex_mode_keeps_spans() {
    let body = render(FORMULAS, &with_math(MathMode::Katex)).body;
    assert!(body.contains("data-math-style=\"inline\""), "{body}");
    assert!(body.contains("data-math-style=\"display\""), "{body}");
    assert!(!body.contains("<math"), "{body}");
}

#[test]
fn math_mathml_mode() {
    let body = render(FORMULAS, &with_math(MathMode::MathML)).body;
    assert_eq!(body.matches("<math").count(), 2, "{body}");
    assert!(!body.contains("data-math-style"), "{body}");
    insta::assert_snapshot!(body);
}

#[test]
fn math_source_mode() {
    let body = render(FORMULAS, &with_math(MathMode::Source)).body;
    assert!(body.contains("<code>x^2</code>"), "{body}");
    assert!(
        body.contains("<pre><code>\\frac{a}{b}</code></pre>"),
        "{body}"
    );
    assert!(!body.contains("data-math-style"), "{body}");
    assert!(!body.contains("<math"), "{body}");
}

#[test]
fn dangerous_formulas_are_escaped_in_every_mode() {
    let html_tags = ["main", "p", "span", "code", "pre"];
    for mode in [MathMode::Katex, MathMode::MathML, MathMode::Source] {
        for src in DANGEROUS {
            let body = render(src, &with_math(mode)).body;
            for (at, _) in body.match_indices('<') {
                let next = body[at + 1..].chars().next();
                assert!(
                    next.is_some_and(|c| c.is_ascii_alphabetic() || c == '/'),
                    "{mode:?} {src}: {body}"
                );
            }
            for name in tag_names(&body) {
                // Теги MathML начинаются с `m`
                assert!(
                    html_tags.contains(&name) || name.starts_with('m'),
                    "{mode:?} {src}: тег {name}: {body}"
                );
                assert!(!["script", "img", "b"].contains(&name), "{mode:?} {src}");
            }
            for (at, _) in body.match_indices('&') {
                let rest = &body[at + 1..];
                let entity = rest.split_once(';').map_or("", |(e, _)| e);
                assert!(
                    !entity.is_empty()
                        && entity.len() <= 10
                        && (entity.chars().all(|c| c.is_ascii_alphanumeric())
                            || entity.strip_prefix('#').is_some_and(|n| {
                                n.chars().all(|c| c.is_ascii_hexdigit() || c == 'x')
                            })),
                    "{mode:?} {src}: & не сущность: {body}"
                );
            }
        }
        if mode != MathMode::MathML {
            assert!(
                render("$a<b$", &with_math(mode)).body.contains("a&lt;b"),
                "{mode:?}"
            );
            assert!(
                render("$a&b$", &with_math(mode)).body.contains("a&amp;b"),
                "{mode:?}"
            );
        }
    }
}

#[test]
fn deep_formulas_fall_back_to_source_in_mathml() {
    let deep = |n: usize| format!("${}x{}$", "{".repeat(n), "}".repeat(n));
    // 10 000 — предел длины, MAX_NESTING + 1 — предел вложенности при короткой формуле
    for n in [10_000, MAX_NESTING + 1] {
        let src = deep(n);
        let mathml = render(&src, &with_math(MathMode::MathML)).body;
        let source = render(&src, &with_math(MathMode::Source)).body;
        assert!(source.contains("<code>{"), "{n}: {source}");
        assert_eq!(mathml, source, "{n}");
    }
    // На пределе вложенности формула ещё идёт в MathML
    let ok = render(&deep(MAX_NESTING / 2), &with_math(MathMode::MathML)).body;
    assert!(ok.contains("<math"), "{ok}");
}

#[test]
fn unsupported_formula_in_mathml_shows_source() {
    for tex in ["$\\nosuchcommand{x}$", "$\\middle x$", "$\\frac{a$"] {
        let body = render(tex, &with_math(MathMode::MathML)).body;
        assert!(!body.contains("PARSE ERROR"), "{tex}: {body}");
    }
    let body = render("$\\nosuchcommand{x}$", &with_math(MathMode::MathML)).body;
    assert!(body.contains("<code>\\nosuchcommand{x}</code>"), "{body}");
}

const MATH_SRC: &str = "$$x$$\n";
const MERMAID_SRC: &str = "```mermaid\nA-->B\n```\n";

#[test]
fn asset_mode_none_has_no_scripts() {
    for src in [MATH_SRC, "plain", MERMAID_SRC] {
        let html = render(src, &opts()).html;
        assert!(!html.contains("<script"), "{src}: {html}");
        assert!(!html.contains("<link"), "{src}: {html}");
    }
}

#[test]
fn asset_mode_link_follows_profile() {
    let math = render(MATH_SRC, &with_assets(link("assets"))).html;
    let plain = render("plain", &with_assets(link("assets"))).html;
    let mermaid = render(MERMAID_SRC, &with_assets(link("assets"))).html;

    assert!(math.contains("<link rel=\"stylesheet\" href=\"assets/katex.min.css\">"));
    assert!(math.contains("<script defer src=\"assets/katex.min.js\"></script>"));
    assert!(!math.contains("mermaid.min.js"), "{math}");
    assert!(!math.contains("securityLevel"), "{math}");
    assert!(math.contains("trust: false"), "{math}");
    assert!(math.contains("DOMContentLoaded"), "{math}");

    assert!(mermaid.contains("<script src=\"assets/mermaid.min.js\"></script>"));
    assert!(!mermaid.contains("katex.min"), "{mermaid}");
    assert!(!mermaid.contains("trust: false"), "{mermaid}");
    assert!(mermaid.contains("securityLevel: 'strict'"), "{mermaid}");
    assert!(mermaid.contains("DOMContentLoaded"), "{mermaid}");

    assert!(!plain.contains("<script"), "{plain}");
    assert!(!plain.contains("<link"), "{plain}");

    // Ресурсы — в `<head>`, а не в `<main>`
    for html in [&math, &mermaid] {
        let head = html.split_once("</head>").unwrap().0;
        assert!(head.contains("<script"), "{html}");
        let body = html.split_once("</head>").unwrap().1;
        assert!(!body.contains("<script"), "{html}");
    }
}

#[test]
fn katex_assets_only_in_katex_mode() {
    for mode in [MathMode::MathML, MathMode::Source] {
        let html = render(
            MATH_SRC,
            &RenderOptions {
                math: mode,
                assets: link("assets"),
                ..opts()
            },
        )
        .html;
        assert!(!html.contains("katex"), "{mode:?}: {html}");
        assert!(!html.contains("<script"), "{mode:?}: {html}");
    }
}

#[test]
fn inline_assets_escape_closing_tags() {
    let inline = AssetMode::Inline {
        katex_css: "c</style>s".to_owned(),
        katex_js: "a</script>b".to_owned(),
        mermaid_js: "m</script>j".to_owned(),
    };
    let both = format!("{MATH_SRC}\n{MERMAID_SRC}");
    let html = render(&both, &with_assets(inline.clone())).html;
    assert!(html.contains("a<\\/script>b"), "{html}");
    assert!(!html.contains("a</script>b"), "{html}");
    assert!(html.contains("c<\\/style>s"), "{html}");
    assert!(html.contains("m<\\/script>j"), "{html}");
    assert!(html.contains("trust: false"), "{html}");
    assert!(html.contains("securityLevel: 'strict'"), "{html}");
    // Второй `<style>` — только у KaTeX при `Inline`
    assert_eq!(html.matches("<style").count(), 2, "{html}");
    assert!(!html.contains("<link"), "{html}");

    let plain = render("plain", &with_assets(inline)).html;
    assert!(!plain.contains("<script"), "{plain}");
    assert_eq!(plain.matches("<style").count(), 1, "{plain}");
}

#[test]
fn link_base_url_is_escaped() {
    let html = render(MATH_SRC, &with_assets(link("a\"><x"))).html;
    assert!(!html.contains("a\"><x"), "{html}");
    assert!(html.contains("a&quot;&gt;&lt;x/katex.min.js"), "{html}");
}

#[test]
fn truncated_banner_after_fragment() {
    let src = "a\n".repeat(MIB);
    let out = render(
        &src,
        &RenderOptions {
            max_bytes: MIB,
            ..opts()
        },
    );
    assert!(out.truncated);
    assert!(
        out.body.ends_with(
            "<div class=\"banner truncated\">Файл обрезан: показаны первые 1 МБ</div></main>"
        ),
        "{}",
        &out.body[out.body.len() - 200..]
    );
    assert!(out.html.contains(&out.body));

    let exact = render(
        &"a".repeat(MIB),
        &RenderOptions {
            max_bytes: MIB,
            ..opts()
        },
    );
    assert!(!exact.truncated);
    assert!(!exact.body.contains("banner"));

    let small = render(
        "abcdef",
        &RenderOptions {
            max_bytes: 3,
            ..opts()
        },
    );
    assert!(small.truncated);
    assert!(
        small.body.ends_with(
            "<p>abc</p>\n<div class=\"banner truncated\">Файл обрезан: показаны первые 3 байт</div></main>"
        ),
        "{}",
        small.body
    );
}

#[test]
fn not_utf8_banner_first_in_main() {
    let out = render(
        "---\na: 1\n---\nx",
        &RenderOptions {
            not_utf8: true,
            ..opts()
        },
    );
    assert!(
        out.body.starts_with(
            "<main class=\"nimmd\"><div class=\"banner not-utf8\">Файл не в UTF-8: \
             нераспознанные байты заменены знаком \u{FFFD}</div><pre class=\"frontmatter\">"
        ),
        "{}",
        out.body
    );
    assert!(!render("x", &opts()).body.contains("not-utf8"));
}

/// Классы списка `## <заголовок>` в `CONTRACT.md`; нет раздела — пусто.
fn contract_list(heading: &str) -> Vec<&'static str> {
    let section = CONTRACT
        .split_once(&format!("## {heading}\n"))
        .map_or("", |(_, rest)| rest);
    let section = section.split("\n## ").next().unwrap_or("");
    section
        .lines()
        .filter_map(|line| line.strip_prefix("- `")?.strip_suffix('`'))
        .collect()
}

/// Классовые селекторы `.имя` CSS вне комментариев, строк и значений свойств.
fn css_classes(css: &str) -> Vec<&str> {
    let mut classes = Vec::new();
    let mut rest = css;
    while let Some(at) = rest.find("/*") {
        let end = rest[at..].find("*/").map_or(rest.len(), |e| at + e + 2);
        classes.extend(selector_classes(&rest[..at]));
        rest = &rest[end..];
    }
    classes.extend(selector_classes(rest));
    classes
}

fn selector_classes(css: &str) -> Vec<&str> {
    let mut classes = Vec::new();
    // Значения свойств лежат между `{`/`;` и `;`/`}`: селекторы — только до `{`
    for chunk in css.split('}') {
        let mut parts = chunk.rsplitn(2, '{');
        let _declarations = parts.next();
        let Some(prelude) = parts.next() else {
            continue;
        };
        let selector = prelude.rsplit('{').next().unwrap_or(prelude);
        for (at, _) in selector.match_indices('.') {
            let name = &selector[at + 1..];
            let end = name
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                .unwrap_or(name.len());
            let name = &name[..end];
            if name.starts_with(|c: char| c.is_ascii_alphabetic()) {
                classes.push(name);
            }
        }
    }
    classes
}

/// Значения `class="…"` в HTML, по словам.
fn html_classes(html: &str) -> Vec<&str> {
    html.split("class=\"")
        .skip(1)
        .filter_map(|rest| rest.split_once('"'))
        .flat_map(|(value, _)| value.split_whitespace())
        .collect()
}

const REFERENCE: &str = "---\ntitle: эталон\n---\n# Эталон\n\n\
                         > [!NOTE]\n> n\n\n> [!TIP]\n> t\n\n> [!IMPORTANT]\n> i\n\n\
                         > [!WARNING]\n> w\n\n> [!CAUTION]\n> c\n\n\
                         - [x] задача\n- [ ] ещё\n\nсноска[^1]\n\n[^1]: текст\n\n\
                         ```mermaid\nA-->B\n```\n";

#[test]
fn class_contract() {
    let shared = contract_list("Общие с редактором");
    let core = contract_list("Только ядра");
    assert!(shared.len() >= 14 && core.len() >= 8, "{shared:?} {core:?}");

    let out = render(REFERENCE, &opts());
    let classes = html_classes(&out.html);
    for class in &shared {
        assert!(
            classes.contains(class),
            "{class} нет в выводе: {}",
            out.body
        );
    }
    for class in ["footnote-backref", "nimmd"] {
        assert!(classes.contains(&class), "{class}: {}", out.body);
    }

    let selectors = css_classes(NIMMD_CSS);
    assert!(selectors.contains(&"nimmd"), "{selectors:?}");
    for class in &selectors {
        assert!(
            shared.contains(class) || core.contains(class),
            "селектор .{class} не из CONTRACT.md"
        );
    }
}

#[test]
fn contract_core_classes_appear_in_output() {
    let out = render(
        IMAGES,
        &RenderOptions {
            not_utf8: true,
            max_bytes: 40,
            ..opts()
        },
    );
    let classes = html_classes(&out.body);
    for class in ["nimmd", "banner", "truncated", "not-utf8", "remote-image"] {
        assert!(classes.contains(&class), "{class}: {}", out.body);
    }
}

#[test]
fn css_selector_parser_sees_classes() {
    let classes = css_classes("/* .no */ a.x, .y > .z-w { color: #fff; margin: .5em; } .q{}");
    assert_eq!(classes, ["x", "y", "z-w", "q"]);
}

#[test]
fn rendered_serializes_lowercase() {
    let out: Rendered = render("\u{FEFF}# T\r\n", &opts());
    let json = serde_json::to_value(&out).unwrap();
    assert_eq!(json["line_ending"], "crlf");
    assert_eq!(json["title"], "T");
    assert_eq!(json["had_bom"], true);
    assert_eq!(json["profile"]["dialect"], "commonmark");
    assert_eq!(
        serde_json::to_value(Theme::Dark).unwrap(),
        serde_json::json!("dark")
    );
    assert_eq!(
        serde_json::to_value(MathMode::MathML).unwrap(),
        serde_json::json!("mathml")
    );
    let theme: Theme = serde_json::from_str("\"light\"").unwrap();
    assert_eq!(theme, Theme::Light);
}

#[test]
fn defaults_match_design() {
    let d = RenderOptions::default();
    assert_eq!(d.theme, Theme::Auto);
    assert_eq!(d.assets, AssetMode::None);
    assert_eq!(d.math, MathMode::Katex);
    assert!(!d.remote_images && !d.not_utf8);
    assert_eq!(d.max_bytes, nimmd_core::text::MAX_BYTES);
    assert_eq!(d.profile, None);
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

const PIECES: [&str; 30] = [
    "```",
    "```mermaid\n",
    "> ",
    "[!NOTE]",
    "- [ ] ",
    "|",
    "$",
    "$$",
    "\\frac{",
    "{",
    "}",
    "^",
    "[[",
    "]]",
    "[^1]",
    "<script>",
    "![i](https://x/",
    "![i](//x)",
    ")",
    "# ",
    "===\n",
    "\n",
    "\r\n",
    "---\n",
    "\u{FEFF}",
    "é",
    "😀",
    "&",
    "<",
    "\\",
];

fn random_src(rng: &mut Rng) -> String {
    let len = rng.next() % 40;
    (0..len)
        .map(|_| PIECES[usize::try_from(rng.next() % PIECES.len() as u64).unwrap_or(0)])
        .collect()
}

#[test]
fn render_is_total_on_random_input() {
    let mut rng = Rng(0x00DE_C0DE_F00D_BEEF);
    let modes = [MathMode::Katex, MathMode::MathML, MathMode::Source];
    for i in 0..2_000 {
        let src = random_src(&mut rng);
        let opts = RenderOptions {
            math: modes[i % 3],
            max_bytes: usize::try_from(rng.next() % 64).unwrap(),
            not_utf8: i % 2 == 0,
            ..opts()
        };
        let out = render(&src, &opts);
        assert!(out.html.contains(&out.body), "{src:?}");
        assert!(out.body.starts_with("<main class=\"nimmd\">"), "{src:?}");
        assert!(out.body.ends_with("</main>"), "{src:?}");
        assert!(!out.html.contains("<script"), "{src:?}");
        assert!(!out.body.contains("<img src=\"https:"), "{src:?}");
        assert!(!out.body.contains("<img src=\"//"), "{src:?}");
    }
}
