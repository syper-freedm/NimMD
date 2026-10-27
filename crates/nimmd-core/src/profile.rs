//! Профиль рендеринга: диалект Markdown и включённые расширения.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Диалект Markdown, задающий базовый набор расширений.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    /// `CommonMark` без расширений.
    #[default]
    CommonMark,
    /// GitHub Flavored Markdown.
    Gfm,
    /// `MultiMarkdown`.
    MultiMarkdown,
    /// Obsidian.
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
    /// YAML между строками `---` в начале.
    pub frontmatter: bool,
    /// Формулы `$…$` и `$$…$$`: подключается KaTeX.
    pub math: bool,
    /// Блоки ```` ```mermaid ````: подключается Mermaid.
    pub mermaid: bool,
    /// Сноски `[^id]`.
    pub footnotes: bool,
    /// Таблицы GFM.
    pub tables: bool,
    /// Списки задач `- [ ]`.
    pub tasklists: bool,
    /// Зачёркивание `~~…~~`.
    pub strikethrough: bool,
    /// Вики-ссылки `[[…]]`.
    pub wikilinks: bool,
    /// Алерты `> [!тип]`.
    pub callouts: bool,
}

/// Диалект и расширения, по которым рендерится документ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Диалект.
    pub dialect: Dialect,
    /// Расширения.
    pub features: Features,
}
