use nimmd_core::profile::{Dialect, Features, Profile, UnknownDialect};

const ALL: [(Dialect, &str); 4] = [
    (Dialect::CommonMark, "commonmark"),
    (Dialect::Gfm, "gfm"),
    (Dialect::MultiMarkdown, "multimarkdown"),
    (Dialect::Obsidian, "obsidian"),
];

#[test]
fn default_profile_is_commonmark_without_features() {
    let p = Profile::default();
    assert_eq!(p.dialect, Dialect::CommonMark);
    let Features {
        frontmatter,
        math,
        mermaid,
        footnotes,
        tables,
        tasklists,
        strikethrough,
        wikilinks,
        callouts,
    } = p.features;
    assert!(
        ![
            frontmatter,
            math,
            mermaid,
            footnotes,
            tables,
            tasklists,
            strikethrough,
            wikilinks,
            callouts
        ]
        .contains(&true)
    );
    assert_eq!(p.features, Features::default());
}

#[test]
fn dialect_json_names_round_trip() {
    for (dialect, name) in ALL {
        let json = serde_json::to_string(&dialect).unwrap();
        assert_eq!(json, format!("\"{name}\""));
        assert_eq!(serde_json::from_str::<Dialect>(&json).unwrap(), dialect);
    }
}

#[test]
fn profile_json_round_trip() {
    let profile = Profile::default();
    let json = serde_json::to_string(&profile).unwrap();
    assert_eq!(serde_json::from_str::<Profile>(&json).unwrap(), profile);

    let custom = Profile {
        dialect: Dialect::Obsidian,
        features: Features {
            frontmatter: true,
            callouts: true,
            ..Features::default()
        },
    };
    let json = serde_json::to_string(&custom).unwrap();
    assert_eq!(serde_json::from_str::<Profile>(&json).unwrap(), custom);
}

#[test]
fn profile_rejects_unknown_fields() {
    let json = r#"{"dialect":"gfm","features":{"frontmatter":false,"math":false,"mermaid":false,"footnotes":false,"tables":false,"tasklists":false,"strikethrough":false,"wikilinks":false,"callouts":false},"extra":1}"#;
    assert!(serde_json::from_str::<Profile>(json).is_err());
    let features = r#"{"frontmatter":false,"math":false,"mermaid":false,"footnotes":false,"tables":false,"tasklists":false,"strikethrough":false,"wikilinks":false,"callouts":false,"extra":true}"#;
    assert!(serde_json::from_str::<Features>(features).is_err());
}

#[test]
fn dialect_from_str_is_ascii_case_insensitive() {
    assert_eq!("GFM".parse::<Dialect>(), Ok(Dialect::Gfm));
    assert_eq!(
        "MultiMarkdown".parse::<Dialect>(),
        Ok(Dialect::MultiMarkdown)
    );
    for (dialect, name) in ALL {
        assert_eq!(name.parse::<Dialect>(), Ok(dialect));
        assert_eq!(name.to_ascii_uppercase().parse::<Dialect>(), Ok(dialect));
    }
}

#[test]
fn dialect_from_str_rejects_unknown() {
    for s in ["kramdown", "", " gfm", "gfm ", "gfm\n", "g fm"] {
        assert_eq!(s.parse::<Dialect>(), Err(UnknownDialect), "{s:?}");
    }
}

#[test]
fn unknown_dialect_is_an_error() {
    assert_ne!(UnknownDialect.to_string(), "");
    let err: &dyn std::error::Error = &UnknownDialect;
    assert!(err.source().is_none());
}
