use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use nimmd_core::detect;
use nimmd_core::profile::{Dialect, Features, Profile};
use nimmd_core::text::{normalize_newlines, split_frontmatter};

fn profile(dialect: Dialect, features: Features) -> Profile {
    Profile { dialect, features }
}

fn none() -> Features {
    Features::default()
}

#[test]
fn plain_commonmark() {
    assert_eq!(
        detect("# A\n\nабзац с *em*\n\n- x\n- y\n"),
        Profile::default()
    );
}

#[test]
fn gfm_table_task_strike() {
    assert_eq!(
        detect("| a | b |\n|---|---|\n\n- [x] done\n\n~~old~~\n"),
        profile(
            Dialect::Gfm,
            Features {
                tables: true,
                tasklists: true,
                strikethrough: true,
                ..none()
            }
        )
    );
}

#[test]
fn footnote_is_gfm_not_multimarkdown() {
    assert_eq!(
        detect("текст[^1]\n\n[^1]: текст\n"),
        profile(
            Dialect::Gfm,
            Features {
                footnotes: true,
                ..none()
            }
        )
    );
}

#[test]
fn obsidian_unambiguous_signs() {
    assert_eq!(
        detect("[[Note]]\n%%c%%\n#tag\n"),
        profile(
            Dialect::Obsidian,
            Features {
                wikilinks: true,
                ..none()
            }
        )
    );
    assert_eq!(detect("%%c%%"), profile(Dialect::Obsidian, none()));
    assert_eq!(
        detect("![[img.png]]\n"),
        profile(
            Dialect::Obsidian,
            Features {
                wikilinks: true,
                ..none()
            }
        )
    );
}

#[test]
fn obsidian_comment_needs_content() {
    assert_eq!(detect("%%%%\n"), Profile::default());
    assert_eq!(detect("%%c\nd%%\n"), Profile::default());
}

#[test]
fn obsidian_weak_signs() {
    let callouts = Features {
        callouts: true,
        ..none()
    };
    assert_eq!(
        detect("> [!note] x\n\n#tag текст\n"),
        profile(Dialect::Obsidian, callouts)
    );
    assert_eq!(
        detect("> [!NOTE] a\n\n> [!warning] b\n"),
        profile(Dialect::CommonMark, callouts)
    );
    assert_eq!(detect("#tag\n\n#идея\n"), Profile::default());
}

#[test]
fn issue_reference_and_heading_are_not_tags() {
    let callouts = Features {
        callouts: true,
        ..none()
    };
    for src in [
        "#123 исправлено\n> [!NOTE] x\n",
        "# Заголовок\n> [!NOTE] x\n",
        "##tag\n> [!NOTE] x\n",
        " #tag\n> [!NOTE] x\n",
    ] {
        assert_eq!(
            detect(src),
            profile(Dialect::CommonMark, callouts),
            "{src:?}"
        );
    }
}

#[test]
fn callout_kind_is_validated() {
    for src in [
        "> [!] x\n#tag\n",
        "> [!a b] x\n#tag\n",
        "> [!note x\n#tag\n",
    ] {
        assert_eq!(detect(src), Profile::default(), "{src:?}");
    }
    assert_eq!(
        detect(">[!tip-1] x\n#tag\n"),
        profile(
            Dialect::Obsidian,
            Features {
                callouts: true,
                ..none()
            }
        )
    );
}

#[test]
fn multimarkdown_signs() {
    let mmd = profile(Dialect::MultiMarkdown, none());
    assert_eq!(detect("Title: X\nAuthor: Y\n\n# T {#t}\n"), mmd);
    assert_eq!(detect("## Раздел {#sec}\n"), mmd);
    assert_eq!(
        detect("| a | b |\n|---|---|\n| c ||\n"),
        profile(
            Dialect::MultiMarkdown,
            Features {
                tables: true,
                ..none()
            }
        )
    );
    assert_eq!(detect("Note: X\n\n"), Profile::default());
    assert_eq!(detect("Title: X\n\nабзац\n"), mmd);
}

#[test]
fn multimarkdown_metadata_edges() {
    let mmd = profile(Dialect::MultiMarkdown, none());
    assert_eq!(detect("Title: X"), mmd);
    assert_eq!(detect("note: a\nAUTHOR: b\n"), mmd);
    assert_eq!(detect("Doc title_2-x: a\nauthor: b\n\nтекст\n"), mmd);
    assert_eq!(
        detect("---\na: b\n---\nTitle: X\n\nабзац\n"),
        profile(
            Dialect::MultiMarkdown,
            Features {
                frontmatter: true,
                ..none()
            }
        )
    );
    for src in [
        "абзац\nTitle: X\n",
        "\nTitle: X\n",
        "Note: a\nпросто строка\nTitle: X\n",
        "Заголовок: a\nTitle: X\n",
        "Note: a\n\nTitle: X\n",
        "Titles: X\n",
    ] {
        assert_eq!(detect(src), Profile::default(), "{src:?}");
    }
}

#[test]
fn heading_id_edges() {
    for src in [
        "# T {#}\n",
        "# T {#a b}\n",
        "####### T {#t}\n",
        "    # T {#t}\n",
        "T {#t}\n",
    ] {
        assert_eq!(detect(src), Profile::default(), "{src:?}");
    }
    assert_eq!(
        detect("###### T {#t}  \n"),
        profile(Dialect::MultiMarkdown, none())
    );
}

#[test]
fn bare_autolink() {
    assert_eq!(
        detect("см. https://example.com\n"),
        profile(Dialect::Gfm, none())
    );
    assert_eq!(detect("http://e.com\n"), profile(Dialect::Gfm, none()));
    assert_eq!(
        detect("[x](https://e.com)\n\n<https://e.com>\n"),
        Profile::default()
    );
    assert_eq!(detect("httpx и http:/e\n"), Profile::default());
}

#[test]
fn task_markers() {
    let tasks = profile(
        Dialect::Gfm,
        Features {
            tasklists: true,
            ..none()
        },
    );
    for src in [
        "* [ ] a\n",
        "+ [X] a\n",
        "1. [ ] a\n",
        "2) [x] a\n",
        "  - [ ] a\n",
        "- [ ]\n",
    ] {
        assert_eq!(detect(src), tasks, "{src:?}");
    }
    for src in ["-[ ] a\n", "- [y] a\n", "a. [ ] a\n", "[ ] a\n"] {
        assert_eq!(detect(src), Profile::default(), "{src:?}");
    }
}

#[test]
fn table_needs_header_row_with_pipe() {
    assert_eq!(detect("a\n|---|---|\n"), Profile::default());
    assert!(detect("a | b\n--|--\n").features.tables);
    assert_eq!(detect("| a |\n\n|---|\n"), Profile::default());
}

#[test]
fn footnote_edges() {
    for src in ["[^ 1]\n", "[^]\n", "[^1 \n"] {
        assert_eq!(detect(src), Profile::default(), "{src:?}");
    }
    assert!(detect("[^[^x]\n").features.footnotes);
}

#[test]
fn signs_inside_code_fences_are_ignored() {
    let inner = "[[x]]\n- [ ] a\n~~s~~\n";
    let backticks = format!("```text\n{inner}```\n");
    let tildes = format!("~~~\n{inner}~~~\n");
    for src in [backticks.clone(), tildes.clone(), backticks + &tildes] {
        assert_eq!(detect(&src), Profile::default(), "{src:?}");
    }
}

#[test]
fn fence_closing_rules() {
    for src in [
        "````\n```\n[[x]]\n````\n",
        "```\n~~~\n[[x]]\n```\n",
        "~~~~\n~~~ \n[[x]]\n",
        "```\n[[x]]\n",
        "   ```\n[[x]]\n   ```\n",
        "```\n[[x]]\n```` \n",
        "```\ncode\n``` x\n[[x]]\n```\n",
    ] {
        assert_eq!(detect(src), Profile::default(), "{src:?}");
    }
    let wiki = profile(
        Dialect::Obsidian,
        Features {
            wikilinks: true,
            ..none()
        },
    );
    for src in [
        "```\ncode\n```\n[[x]]\n",
        "``` a`b\n[[x]]\n",
        "``\n[[x]]\n",
        "    ```\n\n[[x]]\n",
    ] {
        assert_eq!(detect(src), wiki, "{src:?}");
    }
}

#[test]
fn table_header_inside_fence_does_not_count() {
    assert_eq!(detect("```\n| a |\n```\n|---|\n"), Profile::default());
}

#[test]
fn math_from_ast() {
    let math = profile(
        Dialect::CommonMark,
        Features {
            math: true,
            ..none()
        },
    );
    assert_eq!(detect("$$E=mc^2$$ и $a+b$\n"), math);
    assert_eq!(detect("$a+b$\n"), math);
    assert_eq!(detect("$$\nE=mc^2\n$$\n"), math);
    assert_eq!(detect("цена $5 и $10\n"), Profile::default());
    assert_eq!(detect("```\n$a$\n```\n"), Profile::default());
    assert_eq!(detect("`$a$`\n"), Profile::default());
}

#[test]
fn mermaid_from_ast() {
    let mermaid = profile(
        Dialect::CommonMark,
        Features {
            mermaid: true,
            ..none()
        },
    );
    assert_eq!(detect("```mermaid\ngraph TD"), mermaid);
    assert_eq!(detect("```mermaid\ngraph TD\n"), mermaid);
    assert_eq!(detect("```Mermaid\ngraph TD\n```\n"), mermaid);
    assert_eq!(detect("~~~ MERMAID extra\ngraph\n~~~\n"), mermaid);
    assert_eq!(detect("```rust\nmermaid\n```\n"), Profile::default());
    assert_eq!(detect("```mermaidjs\ngraph\n```\n"), Profile::default());
    assert_eq!(detect("    mermaid\n"), Profile::default());
    assert_eq!(detect("mermaid\n"), Profile::default());
}

#[test]
fn frontmatter_overrides_dialect_only() {
    assert_eq!(
        detect("---\nmarkdown: obsidian\n---\nабзац\n"),
        profile(
            Dialect::Obsidian,
            Features {
                frontmatter: true,
                ..none()
            }
        )
    );
    assert_eq!(
        detect("---\nmarkdown: gfm\n---\n[[x]]\n"),
        profile(
            Dialect::Gfm,
            Features {
                frontmatter: true,
                wikilinks: true,
                ..none()
            }
        )
    );
    assert_eq!(
        detect("---\nmarkdown: commonmark\n---\n| a |\n|---|\n"),
        profile(
            Dialect::CommonMark,
            Features {
                frontmatter: true,
                tables: true,
                ..none()
            }
        )
    );
}

#[test]
fn frontmatter_content_is_not_scanned() {
    assert_eq!(
        detect("---\ntags: [[x]]\nmath: $a$\nTitle: X\n---\nабзац\n"),
        profile(
            Dialect::CommonMark,
            Features {
                frontmatter: true,
                ..none()
            }
        )
    );
    // Неизвестное значение `markdown:` не переопределяет детектор.
    assert_eq!(
        detect("---\nmarkdown: kramdown\n---\n[[x]]\n").dialect,
        Dialect::Obsidian
    );
}

#[test]
fn dialect_priority() {
    let all = "[[x]]\n\n# T {#t}\n\n| a |\n|---|\n";
    assert_eq!(detect(all).dialect, Dialect::Obsidian);
    assert_eq!(
        detect("# T {#t}\n\n| a |\n|---|\n").dialect,
        Dialect::MultiMarkdown
    );
}

#[test]
fn edge_inputs_are_total() {
    assert_eq!(detect(""), Profile::default());
    assert_eq!(detect("---\nmarkdown: gfm\n"), Profile::default());
    for src in [
        "|-|-|", "%%%%%", "%%", "\n", "---", "[[", "]]", "[^", "{#", "```", "~~~", "> [!", "#",
        "$", "$$", "é😀",
    ] {
        let _ = detect(src);
    }
    let _ = detect(&(">".repeat(10_000) + "x"));
}

#[test]
fn deep_nesting_is_fast() {
    let limit = Duration::from_secs(1);
    for src in [">".repeat(60_000) + "x", "- ".repeat(60_000) + "x"] {
        let start = Instant::now();
        let _ = detect(&src);
        assert!(start.elapsed() < limit, "{:?}", start.elapsed());
    }
    assert_eq!(detect("> > > текст"), Profile::default());
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

const PIECES: [&str; 28] = [
    "[[", "]]", "%%", "~~", "```", "~~~", "> ", "[!", "#", "|", "-", ":", "$", "[^", "]",
    "http://", "(", "<", "\n", " ", "\t", "x", "é", "{#", "}", "Title: ", "---\n", "1. ",
];

#[test]
fn detect_is_total_on_random_input() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..3000 {
        let len = rng.next() % 40;
        let src: String = (0..len)
            .map(|_| PIECES[usize::try_from(rng.next() % 28).unwrap()])
            .collect();
        let got = detect(&src);
        assert_eq!(
            got.features.frontmatter,
            split_frontmatter(&src).0.is_some(),
            "{src:?}"
        );
        // Признаки внутри незакрытого блока кода не видны: весь текст — код.
        let fenced = format!("{}\n{src}", "~".repeat(128));
        assert_eq!(detect(&fenced), Profile::default(), "{fenced:?}");
    }
}

/// Фикстуры `tests/fixtures/<группа>/<имя>.md` с ожиданием в парном `<имя>.profile.json`.
#[test]
fn fixtures_match_profiles() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;
    for group in fs::read_dir(&root).unwrap() {
        for entry in fs::read_dir(group.unwrap().path()).unwrap() {
            let md = entry.unwrap().path();
            if md.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            let name = md.strip_prefix(&root).unwrap().display().to_string();
            let expected = md.with_extension("profile.json");
            let expected = fs::read_to_string(&expected)
                .unwrap_or_else(|e| panic!("{name}: нет {}: {e}", expected.display()));
            let expected: Profile = serde_json::from_str(&expected)
                .unwrap_or_else(|e| panic!("{name}: неверный .profile.json: {e}"));
            // Checkout с autocrlf на Windows даёт `\r\n`, а `detect` ждёт `\n`.
            let src = fs::read_to_string(&md).unwrap();
            assert_eq!(detect(&normalize_newlines(&src)), expected, "{name}");
            checked += 1;
        }
    }
    assert!(checked > 0, "в {} нет фикстур", root.display());
}
