//! Ядро NimMD
#![forbid(unsafe_code)]

mod detect;
mod parse;
pub mod profile;
mod render;
pub mod text;

pub use detect::detect;
pub use render::render_fragment;
