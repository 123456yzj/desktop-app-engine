extern crate self as desktop_app_engine;

pub mod application;
pub mod binding;
pub mod chart;
pub mod command;
pub mod component;
pub mod credentials;
pub mod description;
pub mod engine;
pub mod gui;
pub mod gui_logs;
pub mod gui_v02;
pub mod http;
pub mod interaction;
pub mod layout;
pub mod loader;
pub mod logging;
pub mod plugin;
pub mod scope;
pub mod table;
pub mod v02;
pub mod validation;

pub const EXAMPLE: &str = include_str!("../examples/hello.json");
pub const MAX_INPUT_CHARS: usize = validation::MAX_STRING_BYTES / 4;
