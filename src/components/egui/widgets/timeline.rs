use super::*;
component!(
    &[
        ("items", Items),
        ("reverse", Bool),
        ("pending", Text),
        ("mode", Enum(&["left", "right", "alternate"]))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext { ui, n, state, .. } = c;
    let items = contract::items(n, state);
    let mut indices: Vec<_> = (0..items.len()).collect();
    if contract::flag(n, state, "reverse", false) {
        indices.reverse();
    }
    for index in indices {
        let item = &items[index];
        let mode = contract::string(n, state, "mode", "left");
        let right = mode == "right" || mode == "alternate" && index % 2 == 1;
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), 24.),
            if right {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            },
            |ui| {
                ui.colored_label(token(ui, Token::Accent), "●");
                ui.strong(item_label(item));
                if let Some(description) = item["description"].as_str() {
                    ui.label(description);
                }
            },
        );
        if index + 1 < items.len() {
            ui.weak("│");
        }
    }
    let pending = contract::string(n, state, "pending", "");
    if !pending.is_empty() {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(pending);
        });
    }
}
