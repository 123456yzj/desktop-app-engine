pub mod application;
pub mod compiler;
mod components;
pub use components::layout;
pub mod loader;
pub mod packaging;
pub mod protocol;
pub mod renderer;
pub mod runtime;
pub mod services;

// Internal short names do not form part of the public library API.
pub(crate) use components::{chart, table, tree as component};
pub(crate) use protocol::{binding, validation};
pub(crate) use renderer::logs as gui_logs;
pub(crate) use runtime as interaction;
pub(crate) use services::{command, credentials, http, logging, plugin};

pub const MAX_INPUT_CHARS: usize = validation::MAX_STRING_BYTES / 4;
