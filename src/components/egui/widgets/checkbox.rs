use super::*;
component!(&[("options", Array)], &["change"]);
pub(in crate::components::egui) fn control(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
) {
    if let Some(options) = resolved.api.get("options").and_then(Value::as_array) {
        let mut selected = value.as_array().cloned().unwrap_or_default();
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((&node.id, "group"))
                .max_rect(rect),
        );
        child.horizontal_wrapped(|ui| {
            for option in options {
                let key = option
                    .as_str()
                    .or_else(|| option["value"].as_str())
                    .unwrap_or("");
                let label = option
                    .as_str()
                    .or_else(|| option["label"].as_str())
                    .unwrap_or(key);
                let mut on = selected.iter().any(|v| v.as_str() == Some(key));
                if ui
                    .add_enabled(
                        option["disabled"] != true,
                        egui::Checkbox::new(&mut on, label),
                    )
                    .changed()
                {
                    selected.retain(|v| v.as_str() != Some(key));
                    if on {
                        selected.push(json!(key));
                    }
                }
            }
        });
        *value = json!(selected);
        return;
    }
    let mut checked = value.as_bool().unwrap_or(false);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((&node.id, "checkbox"))
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.spacing_mut().icon_width = 18.;
    child.spacing_mut().icon_spacing = 8.;
    let response = child
        .add(egui::Checkbox::new(&mut checked, &resolved.text).indeterminate(node.indeterminate));
    if response.clicked() {
        response.request_focus();
    }
    if response.changed() {
        *value = json!(checked);
    }
}
