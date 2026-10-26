use super::*;
component!(&[], &[]);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    node: &protocol::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    error: Option<&str>,
) {
    let rect = rect.shrink(node.layout.padding + node.style.border_width);
    let font = node.style.font_size.unwrap_or(16.);
    if !resolved.label.is_empty() {
        let required = node.rules.iter().any(|r| r.required);
        let offset = if required { 12. } else { 0. };
        if required {
            painter.text(
                rect.min,
                egui::Align2::LEFT_TOP,
                "*",
                FontId::proportional(font),
                token(ui, Token::Danger),
            );
        }
        label_line(
            ui,
            painter,
            egui::Rect::from_min_size(
                rect.min + Vec2::new(offset, 0.),
                Vec2::new(rect.width() - offset, font + 8.),
            ),
            &resolved.label,
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
