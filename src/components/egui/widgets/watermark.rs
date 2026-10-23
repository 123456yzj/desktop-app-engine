use super::*;
component!(
    &[
        ("content", Text),
        ("gap", Number(16., 1000.)),
        ("rotate", Number(-360., 360.))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        ..
    } = c;
    let gap = contract::number(n, state, "gap", 120.) as f32;
    let content = contract::string(n, state, "content", &r.text);
    let color = token(ui, Token::TextMuted).gamma_multiply(0.25);
    let galley = ui
        .painter()
        .layout_no_wrap(content.into(), FontId::proportional(14.), color);
    let angle = (contract::number(n, state, "rotate", -22.) as f32).to_radians();
    let mut y = rect.top();
    while y < rect.bottom() {
        let mut x = rect.left();
        while x < rect.right() {
            ui.painter().add(
                egui::epaint::TextShape::new(Pos2::new(x, y), galley.clone(), color)
                    .with_angle(angle),
            );
            x += gap;
        }
        y += gap;
    }
}
