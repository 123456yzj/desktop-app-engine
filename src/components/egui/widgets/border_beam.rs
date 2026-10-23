use super::*;
component!(
    &[
        ("duration", Number(0.1, 3600.)),
        ("borderWidth", Number(1., 32.))
    ],
    &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui, n, rect, state, ..
    } = c;
    let time = ui.input(|i| i.time) as f32 / contract::number(n, state, "duration", 6.) as f32;
    let perimeter = 2. * (rect.width() + rect.height());
    let point = |distance: f32| {
        let d = distance.rem_euclid(perimeter.max(1.));
        if d < rect.width() {
            rect.left_top() + Vec2::new(d, 0.)
        } else if d < rect.width() + rect.height() {
            rect.right_top() + Vec2::new(0., d - rect.width())
        } else if d < 2. * rect.width() + rect.height() {
            rect.right_bottom() - Vec2::new(d - rect.width() - rect.height(), 0.)
        } else {
            rect.left_bottom() - Vec2::new(0., d - 2. * rect.width() - rect.height())
        }
    };
    for i in 0..32 {
        let start = (time.fract() + i as f32 / 128.) * perimeter;
        ui.painter().line_segment(
            [point(start), point(start + perimeter / 128.)],
            Stroke::new(
                contract::number(n, state, "borderWidth", 2.) as f32,
                token(ui, Token::Accent).gamma_multiply(i as f32 / 32.),
            ),
        );
    }
    ui.ctx().request_repaint();
}
