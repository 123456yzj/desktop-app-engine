//! Renderer-independent metadata shared by compilation, layout and event validation.
#[derive(Clone, Copy)]
pub(crate) enum PropertyType {
    Bool,
    Text,
    Number(f64, f64),
    Integer(u64, u64),
    Enum(&'static [&'static str]),
    Array,
    Items,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutKind {
    Column,
    Row,
    Directional,
    Page,
    Grid,
    Masonry,
    Splitter,
}
#[derive(Clone, Copy)]
pub(crate) struct Definition {
    pub properties: &'static [(&'static str, PropertyType)],
    pub events: &'static [&'static str],
    pub layout: LayoutKind,
    pub primary: Option<&'static str>,
    pub secondary: &'static [(&'static str, &'static str)],
}
pub(in crate::components) use super::egui::widgets::definition;
