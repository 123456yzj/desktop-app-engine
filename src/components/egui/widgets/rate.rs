use super::*;
component!(
    &[
        ("count", Integer(1, 100)),
        ("allowHalf", Bool),
        ("allowClear", Bool),
        ("character", Text),
        ("tooltips", Array)
    ],
    &["change", "hoverChange"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        state,
        value,
        collector,
        ..
    } = c;
    let mut rating = value.as_f64().unwrap_or(0.);
    let count = contract::number(n, state, "count", 5.) as usize;
    ui.horizontal(|ui| {
        for index in 0..count {
            let response = ui.add(
                egui::Button::new(
                    egui::RichText::new(contract::string(n, state, "character", "★"))
                        .size(24.)
                        .color(if rating > index as f64 {
                            token(ui, Token::Warning)
                        } else {
                            token(ui, Token::Border)
                        }),
                )
                .frame(false),
            );
            if response.clicked() {
                let half = contract::flag(n, state, "allowHalf", false)
                    && response
                        .interact_pointer_pos()
                        .is_some_and(|p| p.x < response.rect.center().x);
                let next = index as f64 + if half { 0.5 } else { 1. };
                rating = if contract::flag(n, state, "allowClear", true) && next == rating {
                    0.
                } else {
                    next
                };
            }
            if response.hovered() {
                collector.push(ui, "hoverChange", None, json!(index + 1));
            }
            if let Some(tip) = contract::property(n, state, "tooltips")
                .and_then(|tips| tips.get(index))
                .and_then(Value::as_str)
            {
                response.on_hover_text(tip);
            }
        }
    });
    *value = json!(rating);
}
