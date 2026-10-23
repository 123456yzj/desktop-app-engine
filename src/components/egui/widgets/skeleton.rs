use super::*;
component!(
    &[
        ("active", Bool),
        ("rows", Integer(1, 100)),
        ("avatar", Bool),
        ("round", Bool)
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui, n, rect, state, ..
    } = c;
    let active = contract::flag(n, state, "active", false);
    let time = ui.input(|i| i.time) as f32;
    let tone =
        token(ui, Token::Border).gamma_multiply(if active { 0.5 + 0.25 * time.sin() } else { 0.6 });
    let rows = contract::number(n, state, "rows", 3.) as usize;
    let avatar = contract::flag(n, state, "avatar", false);
    if avatar {
        ui.painter()
            .circle_filled(rect.left_top() + Vec2::splat(20.), 20., tone);
    }
    let inset = if avatar { 52. } else { 0. };
    for index in 0..rows {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                rect.min + Vec2::new(inset, index as f32 * 28.),
                Vec2::new(
                    (rect.width() - inset).max(0.) * if index + 1 == rows { 0.6 } else { 1. },
                    16.,
                ),
            ),
            if contract::flag(n, state, "round", false) {
                8.
            } else {
                3.
            },
            tone,
        );
    }
    if active {
        ui.ctx().request_repaint();
    }
}
