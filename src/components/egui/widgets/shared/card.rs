use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("loading", Bool),
    ("bordered", Bool),
    ("hoverable", Bool),
    ("extra", Text),
];
pub(in crate::components::egui::widgets) fn paint(
    ui: &mut egui::Ui,
    n: &v02::Node,
    rect: egui::Rect,
    content: egui::Rect,
    state: &crate::binding::State,
) {
    let painter = ui.painter().clone();
    let radius = n.style.radius.max(6.);
    painter.rect_filled(
        rect,
        radius,
        token(ui, n.style.background.unwrap_or(Token::SurfaceRaised)),
    );
    let hovered = contract::flag(n, state, "hoverable", false)
        && ui
            .interact(
                rect,
                egui::Id::new((&n.id, "cardHover")),
                egui::Sense::hover(),
            )
            .hovered();
    if contract::flag(n, state, "bordered", true) {
        let width = if n.style.border_color.is_some() {
            n.style.border_width
        } else {
            1_f32
        };
        if width > 0. {
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(
                    width,
                    token(
                        ui,
                        if hovered {
                            Token::Accent
                        } else {
                            n.style.border_color.unwrap_or(Token::Border)
                        },
                    ),
                ),
                StrokeKind::Inside,
            );
        }
    }
    let extra = contract::string(n, state, "extra", "");
    if !extra.is_empty() {
        painter.text(
            rect.right_top() - Vec2::new(n.layout.padding, -n.layout.padding),
            egui::Align2::RIGHT_TOP,
            extra,
            style_font(&n.style, ui.ctx()),
            token(ui, Token::TextMuted),
        );
    }
    if contract::flag(n, state, "loading", false) {
        ui.put(content, egui::Spinner::new());
    }
}
