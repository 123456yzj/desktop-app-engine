use super::*;
component!(
    &[("prefix", Text), ("suffix", Text), ("loading", Bool)],
    &[]
);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    n: &v02::Node,
    r: &Resolved,
    rect: egui::Rect,
    state: &crate::binding::State,
) {
    if contract::flag(n, state, "loading", false) {
        ui.put(rect, egui::Spinner::new());
        return;
    }
    let font = n.style.font_size.unwrap_or(16.);
    let painter = ui.painter().clone();
    label(
        ui,
        &painter,
        rect,
        &r.label,
        font,
        token(ui, Token::TextMuted),
    );
    label(
        ui,
        &painter,
        egui::Rect::from_min_max(rect.min + Vec2::new(0., font + 12.), rect.max),
        &r.text,
        font * 1.875,
        token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
    );
}
