use super::*;
component!(shared::card::PROPERTIES, &[], LayoutKind::Column);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    n: &protocol::Node,
    rect: egui::Rect,
    content: egui::Rect,
    state: &crate::binding::State,
) {
    shared::card::paint(ui, n, rect, content, state);
}
