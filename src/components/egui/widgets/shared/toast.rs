use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("open", Bool),
    ("duration", Number(0., 3600.)),
    ("description", Text),
    (
        "status",
        Enum(&["success", "info", "warning", "error", "loading"]),
    ),
    ("closable", Bool),
    (
        "placement",
        Enum(&[
            "topLeft",
            "topRight",
            "bottomLeft",
            "bottomRight",
            "top",
            "bottom",
        ]),
    ),
];

pub(in crate::components::egui::widgets) fn paint(
    c: WidgetContext<'_, '_>,
    placement: &str,
    closable: bool,
) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        collector,
        ..
    } = c;
    if !local(n, state, memory, "open", json!(true))
        .as_bool()
        .unwrap_or(true)
    {
        memory.started = None;
        memory.dismissed = false;
        return;
    }
    let now = ui.input(|i| i.time);
    let start = *memory.started.get_or_insert(now);
    let duration = contract::number(n, state, "duration", 3.);
    if memory.dismissed {
        return;
    }
    let mut close = duration > 0. && now - start >= duration;
    if !close {
        let anchor = match contract::string(n, state, "placement", placement) {
            "topLeft" => egui::Align2::LEFT_TOP,
            "bottomLeft" => egui::Align2::LEFT_BOTTOM,
            "bottomRight" => egui::Align2::RIGHT_BOTTOM,
            "bottom" => egui::Align2::CENTER_BOTTOM,
            "top" => egui::Align2::CENTER_TOP,
            _ => egui::Align2::RIGHT_TOP,
        };
        egui::Area::new(egui::Id::new((&n.id, "toast")))
            .order(egui::Order::Foreground)
            .anchor(anchor, Vec2::new(16., 16.))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(12.)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                token(
                                    ui,
                                    shared::status_color(contract::string(
                                        n, state, "status", "info",
                                    )),
                                ),
                                &r.text,
                            );
                            if contract::flag(n, state, "closable", closable)
                                && ui.small_button("×").clicked()
                            {
                                close = true;
                            }
                        });
                        if !r.title.is_empty() {
                            ui.strong(&r.title);
                        }
                        let description = contract::string(n, state, "description", "");
                        if !description.is_empty() {
                            ui.label(description);
                        }
                    });
            });
        if duration > 0. {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    if close {
        memory.dismissed = true;
        emit(ui, n, memory, collector, "openChange", json!(false));
        collector.push(ui, "close", None, Value::Null);
    }
}
