//! Native rendering adapters. Runtime state remains owned by the engine.
mod egui;
pub(crate) mod legacy;
pub(crate) mod logs;
pub use egui::DashboardApp;
pub use legacy::{DesktopApp, install_chinese_font};
