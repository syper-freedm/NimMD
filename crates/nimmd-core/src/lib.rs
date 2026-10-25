//! Ядро NimMD
#![forbid(unsafe_code)]

mod detect;
mod document;
mod parse;
pub mod profile;
mod render;
pub mod text;

pub use detect::detect;
pub use document::{AssetMode, MathMode, RenderOptions, Rendered, Theme, render};
pub use render::render_fragment;
