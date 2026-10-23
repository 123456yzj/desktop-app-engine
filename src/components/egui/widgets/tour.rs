use super::*;
component!(
    &[
        ("steps", Items),
        ("current", Integer(0, 10000)),
        ("open", Bool),
        ("nextText", Text),
        ("prevText", Text),
        ("finishText", Text)
    ],
    &["change", "openChange", "close", "finish"]; Some("current"), &[("openChange", "open")]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        state,
        memory,
        value,
        collector,
        ..
    } = c;
    if !local(n, state, memory, "open", json!(false))
        .as_bool()
        .unwrap_or(false)
    {
        return;
    }
    let steps = contract::items(n, state);
    let mut current = value.as_u64().unwrap_or(0) as usize;
    let Some(step) = steps.get(current) else {
        return;
    };
    egui::Window::new(item_label(step))
        .id(egui::Id::new((&n.id, "tour")))
        .order(egui::Order::Foreground)
        .collapsible(false)
        .show(ui.ctx(), |ui| {
            ui.label(step["description"].as_str().unwrap_or(""));
            ui.label(format!("{} / {}", current + 1, steps.len()));
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        current > 0,
                        egui::Button::new(contract::string(n, state, "prevText", "上一步")),
                    )
                    .clicked()
                {
                    current = current.saturating_sub(1);
                }
                if current + 1 < steps.len() {
                    if ui
                        .button(contract::string(n, state, "nextText", "下一步"))
                        .clicked()
                    {
                        current += 1;
                    }
                } else if ui
                    .button(contract::string(n, state, "finishText", "完成"))
                    .clicked()
                {
                    emit(ui, n, memory, collector, "openChange", json!(false));
                    collector.push(ui, "finish", None, Value::Null);
                }
                if ui.button("关闭").clicked() {
                    emit(ui, n, memory, collector, "openChange", json!(false));
                    collector.push(ui, "close", None, Value::Null);
                }
            });
        });
    *value = json!(current);
}
