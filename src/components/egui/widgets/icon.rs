use super::*;
component!(
    &[
        ("name", Text),
        ("spin", Bool),
        ("rotate", Number(-36000., 36000.))
    ],
    &["click"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        rect,
        state,
        collector,
        ..
    } = c;
    let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    let response = ui.interact(rect, egui::Id::new((&n.id, "icon")), egui::Sense::click());
    let spin = contract::flag(n, state, "spin", false);
    let angle = contract::number(n, state, "rotate", 0.).to_radians() as f32
        + if spin {
            ui.input(|i| i.time) as f32 * 3.
        } else {
            0.
        };
    let name = contract::string(n, state, "name", "info");
    let size = rect.width().min(rect.height()).min(64.) * 0.75;
    let rotation = egui::emath::Rot2::from_angle(angle);
    if matches!(name, "appstore" | "AppstoreOutlined") {
        // Four outlined tiles are independent of installed font glyph coverage.
        for x in [-0.5, 0.08] {
            for y in [-0.5, 0.08] {
                let points = [(x, y), (x + 0.42, y), (x + 0.42, y + 0.42), (x, y + 0.42)]
                    .map(|(x, y)| rect.center() + rotation * Vec2::new(x * size, y * size));
                ui.painter().add(egui::Shape::closed_line(
                    points.to_vec(),
                    Stroke::new((size / 12.).max(1.), color),
                ));
            }
        }
    } else {
        let galley = ui.painter().layout_no_wrap(
            shared::icon(name).into(),
            FontId::proportional(size),
            color,
        );
        let offset = rotation * (galley.size() * 0.5);
        ui.painter().add(
            egui::epaint::TextShape::new(rect.center() - offset, galley, color).with_angle(angle),
        );
    }
    if response.clicked() {
        collector.push(ui, "click", None, Value::Null);
    }
    if spin {
        ui.ctx().request_repaint();
    }
}
