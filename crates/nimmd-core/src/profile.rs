//! Профиль рендеринга: диалект Markdown и включённые расширения.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Диалект Markdown, задающий базовый набор расширений.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    #[default]
    CommonMark,
    Gfm,
    MultiMarkdown,
    Obsidian,
}

/// Строка не является именем известного диалекта.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownDialect;

impl fmt::Display for UnknownDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown markdown dialect")
    }
}

impl std::error::Error for UnknownDialect {}

impl FromStr for Dialect {
    type Err = UnknownDialect;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        [
            ("commonmark", Self::CommonMark),
            ("gfm", Self::Gfm),
            ("multimarkdown", Self::MultiMarkdown),
            ("obsidian", Self::Obsidian),
        ]
        .into_iter()
        .find(|(name, _)| s.eq_ignore_ascii_case(name))
        .map(|(_, dialect)| dialect)
        .ok_or(UnknownDialect)
    }
}

/// Включённые расширения синтаксиса.
#[allow(clippy::struct_excessive_bools)] // набор независимых флагов задан форматом профиля
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Features {
    pub frontmatter: bool,
    pub math: bool,
    pub mermaid: bool,
    pub footnotes: bool,
    pub tables: bool,
    pub tasklists: bool,
    pub strikethrough: bool,
    pub wikilinks: bool,
    pub callouts: bool,
}

/// Диалект и расширения, по которым рендерится документ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub dialect: Dialect,
    pub features: Features,
}
