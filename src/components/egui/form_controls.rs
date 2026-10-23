//! Shared form-control frame: styling, value comparison and submit collection.
use super::*;
use serde_json::Value;
pub(super) use widgets::input_number::NumberEdit;
pub(super) fn control(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
    edit: &mut NumberEdit,
) -> (bool, bool) {
    control_theme(ui, &node.style, rect.height().min(34.));
    ui.style_mut().override_font_id = Some(style_font(&node.style, ui.ctx()));
    let before = value.clone();
    let mut submit = false;
    ui.add_enabled_ui(resolved.enabled, |ui| {
        ui.push_id((&node.id, "control"), |ui| match node.kind {
            Kind::Checkbox => widgets::checkbox::control(ui, node, resolved, rect, value),
            Kind::Switch => widgets::switch::control(ui, node, resolved, rect, value),
            Kind::Radio => widgets::radio::control(ui, node, resolved, rect, value),
            Kind::InputNumber => {
                submit = widgets::input_number::control(ui, node, resolved, rect, value, edit)
            }
            Kind::Slider => widgets::slider::control(ui, node, resolved, rect, value),
            _ => {}
        });
    });
    (
        if matches!(node.kind, Kind::InputNumber | Kind::Slider) && !value.is_array() {
            before.as_f64() != value.as_f64()
        } else {
            before != *value
        },
        submit,
    )
}
