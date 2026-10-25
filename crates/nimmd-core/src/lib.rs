//! Ядро NimMD
#![forbid(unsafe_code)]

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
pub use highlight::{HIGHLIGHT_LIMIT, Span, highlight};
pub use render::{MathMode, render_fragment};
