use super::*;
component!(
    &[
        ("description", Text),
        ("status", Enum(&["success", "info", "warning", "error"])),
        ("closable", Bool),
        ("showIcon", Bool),
        ("banner", Bool)
    ],
    &["close"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        collector,
        ..
    } = c;
    if memory.dismissed {
        return;
    }
    let banner = contract::flag(n, state, "banner", false);
    let status = contract::string(n, state, "status", if banner { "warning" } else { "info" });
    egui::Frame::new()
        .fill(token(ui, shared::status_color(status)).gamma_multiply(0.15))
        .inner_margin(8.)
        .corner_radius(if banner { 0. } else { 6. })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if contract::flag(n, state, "showIcon", banner) {
                    ui.label(shared::icon(status));
                }
                ui.label(&r.text);
                if contract::flag(n, state, "closable", false) && ui.small_button("×").clicked() {
                    memory.dismissed = true;
                    collector.push(ui, "close", None, Value::Null);
                }
            });
            let description = contract::string(n, state, "description", "");
            if !description.is_empty() {
                ui.label(description);
            }
        });
}
