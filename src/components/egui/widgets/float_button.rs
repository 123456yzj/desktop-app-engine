use super::*;
component!(
    &[
        ("icon", Text),
        ("description", Text),
        ("tooltip", Text),
        ("shape", Enum(&["circle", "square"])),
        ("offsetX", Number(0., 10000.)),
        ("offsetY", Number(0., 10000.))
    ],
    &["click"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        collector,
        ..
    } = c;
    let screen = ui.ctx().screen_rect();
    let pos = Pos2::new(
        screen.right() - 64. - contract::number(n, state, "offsetX", 24.) as f32,
        screen.bottom() - 64. - contract::number(n, state, "offsetY", 24.) as f32,
    );
    egui::Area::new(egui::Id::new((&n.id, "float")))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| {
            let label = if r.text.is_empty() {
                contract::string(n, state, "icon", "+")
            } else {
                &r.text
            };
            let response = ui.add_enabled(
                r.enabled,
                egui::Button::new(label)
                    .min_size(Vec2::splat(48.))
                    .corner_radius(
                        if contract::string(n, state, "shape", "circle") == "circle" {
                            24.
                        } else {
                            6.
                        },
                    ),
            );
            if response.clicked() {
                collector.push(ui, "click", None, Value::Null);
            }
            response.on_hover_text(contract::string(n, state, "tooltip", ""));
            let description = contract::string(n, state, "description", "");
            if !description.is_empty() {
                ui.small(description);
            }
        });
}
