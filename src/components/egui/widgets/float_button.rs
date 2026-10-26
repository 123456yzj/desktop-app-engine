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
fn position(screen: egui::Rect, n: &protocol::Node, state: &crate::binding::State) -> Pos2 {
    Pos2::new(
        screen.right() - 64. - contract::number(n, state, "offsetX", 24.) as f32,
        screen.bottom() - 64. - contract::number(n, state, "offsetY", 24.) as f32,
    )
}

fn content(
    ui: &mut egui::Ui,
    n: &protocol::Node,
    r: &Resolved,
    state: &crate::binding::State,
) -> egui::Response {
    ui.style_mut().spacing.button_padding = Vec2::splat(10.);
    let label = if r.text.is_empty() {
        contract::string(n, state, "icon", "+")
    } else {
        &r.text
    };
    let response = ui.add_enabled(
        r.enabled,
        egui::Button::new(
            egui::RichText::new(label)
                .size(20.)
                .color(token(ui, n.style.color.unwrap_or(Token::OnPrimary))),
        )
        .fill(token(ui, n.style.background.unwrap_or(Token::Primary)))
        .stroke(Stroke::NONE)
        .min_size(Vec2::splat(48.))
        .corner_radius(
            if contract::string(n, state, "shape", "circle") == "circle" {
                24.
            } else {
                6.
            },
        ),
    );
    let response = response.on_hover_text(contract::string(n, state, "tooltip", ""));
    let description = contract::string(n, state, "description", "");
    if !description.is_empty() {
        ui.small(description);
    }
    response
}

pub(super) fn reserved_rect(
    ui: &egui::Ui,
    n: &protocol::Node,
    r: &Resolved,
    state: &crate::binding::State,
) -> egui::Rect {
    let available = ui
        .ctx()
        .style()
        .spacing
        .default_area_size
        .min(ui.ctx().screen_rect().size());
    let mut measure = floating_measure_ui(ui, n, available);
    content(&mut measure, n, r, state);
    let rect = egui::Rect::from_min_size(
        position(ui.ctx().screen_rect(), n, state),
        measure.min_size(),
    );
    let screen = ui.ctx().screen_rect();
    let max = (screen.max - rect.size()).max(screen.min);
    egui::Rect::from_min_size(rect.min.clamp(screen.min, max), rect.size())
}

pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        collector,
        ..
    } = c;
    let rect = reserved_rect(ui, n, r, state);
    let style = ui.style().clone();
    let width = ui
        .ctx()
        .style()
        .spacing
        .default_area_size
        .x
        .min(ui.ctx().screen_rect().width());
    let shown = egui::Area::new(egui::Id::new((&n.id, "float")))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .default_size(rect.size())
        .constrain(false)
        .show(ui.ctx(), |ui| {
            ui.set_style(style);
            ui.set_max_width(width);
            if content(ui, n, r, state).clicked() {
                collector.push(ui, "click", None, Value::Null);
            }
        });
    memory.body = Some(shown.response.rect);
}
