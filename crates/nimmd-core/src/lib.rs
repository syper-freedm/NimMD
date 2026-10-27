#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod csv;
mod data;
mod detect;
mod document;
mod highlight;
mod parse;
pub mod profile;
mod render;
pub mod text;

pub use data::FileKind;
pub use detect::detect;
pub use document::{AssetMode, RenderOptions, Rendered, Theme, render};
pub use highlight::{CSV_COLOR_LIMIT, HIGHLIGHT_LIMIT, MARKDOWN_HIGHLIGHT_LIMIT, Span, highlight};
pub use render::{MathMode, render_fragment};
