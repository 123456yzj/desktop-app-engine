use super::*;
component!(&[], &[]);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    error: Option<&str>,
) {
    let rect = rect.shrink(node.layout.padding + node.style.border_width);
    let font = node.style.font_size.unwrap_or(16.);
    if !resolved.label.is_empty() {
        let label = format!(
            "{}{}",
            if node.rules.iter().any(|r| r.required) {
                "* "
            } else {
                ""
            },
            resolved.label
        );
        label_line(
            ui,
            painter,
            egui::Rect::from_min_size(rect.min, Vec2::new(rect.width(), font + 8.)),
            &label,
            font,
            token(ui, node.style.color.unwrap_or(Token::TextPrimary)),
        );
    }
    if let Some(error) = error {
        label_line(
            ui,
            painter,
            egui::Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - font - 8.), rect.max),
            error,
            font,
            token(ui, Token::Danger),
        );
    }
}
