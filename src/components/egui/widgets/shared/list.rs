use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("items", Items),
    ("bordered", Bool),
    ("split", Bool),
    ("itemHeight", Number(16., 1024.)),
    ("loading", Bool),
];
pub(in crate::components::egui::widgets) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui, n, rect, state, ..
    } = c;
    if contract::flag(n, state, "loading", false) {
        ui.spinner();
        return;
    }
    let items = contract::items(n, state);
    if contract::flag(n, state, "bordered", false) {
        ui.painter().rect_stroke(
            rect,
            4.,
            Stroke::new(1_f32, token(ui, Token::Border)),
            StrokeKind::Inside,
        );
    }
    egui::ScrollArea::vertical()
        .max_height(rect.height())
        .show_rows(
            ui,
            contract::number(n, state, "itemHeight", 48.) as f32,
            items.len(),
            |ui, range| {
                for index in range {
                    let item = &items[index];
                    ui.strong(item_label(item));
                    if let Some(description) = item["description"].as_str() {
                        ui.label(description);
                    }
                    if contract::flag(n, state, "split", true) {
                        ui.separator();
                    }
                }
            },
        );
}
