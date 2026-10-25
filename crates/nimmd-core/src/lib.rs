//! Ядро NimMD
#![forbid(unsafe_code)]

mod detect;
mod parse;
pub mod profile;
pub mod text;

pub use detect::detect;
