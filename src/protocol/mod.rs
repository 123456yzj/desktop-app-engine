//! Authored application contracts. Parsing does not execute application effects.
pub mod binding;
mod common;
pub use common::{App, Window};
pub mod v02;
pub(crate) mod validation;
