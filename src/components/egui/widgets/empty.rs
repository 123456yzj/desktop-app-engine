use super::*;
component!(&[("description", Text)], &[]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui, n, r, state, ..
    } = c;
    ui.vertical_centered(|ui| {
        ui.add_space(12.);
        ui.label(
            egui::RichText::new("□")
                .size(32.)
                .color(token(ui, Token::TextMuted)),
        );
        ui.weak(contract::string(
            n,
            state,
            "description",
            if r.text.is_empty() {
                "暂无数据"
            } else {
                &r.text
            },
        ));
    });
}
