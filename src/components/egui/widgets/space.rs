use super::*;
component!(&[("separator", Text)], &[], LayoutKind::Directional);
pub(in crate::components::egui) fn paint(
    ui: &egui::Ui,
    n: &protocol::Node,
    r: &Resolved,
    children: &[usize],
    resolved: &[Resolved],
    boxes: &crate::layout::LayoutResult,
) {
    let Some(separator) = r.api.get("separator").and_then(Value::as_str) else {
        return;
    };
    let visible: Vec<_> = children
        .iter()
        .copied()
        .filter(|&i| resolved[i].visible)
        .collect();
    for pair in visible.windows(2) {
        let first = er(boxes.boxes[pair[0]].rect);
        let second = er(boxes.boxes[pair[1]].rect);
        let pos = if n.vertical {
            Pos2::new(first.center().x, (first.bottom() + second.top()) / 2.)
        } else {
            Pos2::new((first.right() + second.left()) / 2., first.center().y)
        };
        ui.painter().text(
            pos,
            egui::Align2::CENTER_CENTER,
            separator,
            style_font(&n.style, ui.ctx()),
            token(ui, n.style.color.unwrap_or(Token::TextPrimary)),
        );
    }
}
