use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("open", Bool),
    ("trigger", Enum(&["hover", "click", "contextMenu"])),
    ("description", Text),
    ("okText", Text),
    ("cancelText", Text),
    ("showCancel", Bool),
];

pub(in crate::components::egui::widgets) fn paint(
    c: WidgetContext<'_, '_>,
    default_trigger: &str,
    confirm: bool,
) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        memory,
        collector,
        ..
    } = c;
    let response = if r
        .api
        .get("@childCount")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
    {
        ui.add_sized(
            rect.size(),
            egui::Button::new(if r.text.is_empty() { &r.title } else { &r.text }),
        )
    } else {
        ui.interact(
            rect,
            egui::Id::new((&n.id, "trigger")),
            egui::Sense::click(),
        )
    };
    let id = egui::Id::new((&n.id, "popover"));
    let trigger = contract::string(n, state, "trigger", default_trigger);
    let controlled = matches!(n.api.get("open"), Some(Binding::State { .. }));
    let desired = local(n, state, memory, "open", json!(false)) == true;
    sync(ui, id, controlled, desired, memory.was_open);
    if requested(&response, trigger) && !ui.memory(|m| m.is_popup_open(id)) {
        if controlled {
            emit(ui, n, memory, collector, "openChange", json!(true));
        } else {
            ui.memory_mut(|m| m.open_popup(id));
        }
    }
    let popup = egui::popup::popup_above_or_below_widget(
        ui,
        id,
        &response,
        egui::AboveOrBelow::Below,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.strong(&r.title);
            ui.label(&r.text);
            ui.label(contract::string(n, state, "description", ""));
            if confirm {
                ui.horizontal(|ui| {
                    if contract::flag(n, state, "showCancel", true)
                        && ui
                            .button(contract::string(n, state, "cancelText", "取消"))
                            .clicked()
                    {
                        collector.push(ui, "cancel", None, Value::Null);
                        ui.memory_mut(|m| m.close_popup());
                    }
                    if ui
                        .button(contract::string(n, state, "okText", "确定"))
                        .clicked()
                    {
                        collector.push(ui, "confirm", None, Value::Null);
                        ui.memory_mut(|m| m.close_popup());
                    }
                });
            }
            ui.min_rect()
        },
    );
    close_on_leave(ui, &response, popup, trigger);
    finish(ui, n, state, memory, collector, id, controlled, desired);
}

pub(in crate::components::egui::widgets) fn sync(
    ui: &egui::Ui,
    id: egui::Id,
    controlled: bool,
    desired: bool,
    was_open: bool,
) {
    if controlled {
        if desired {
            ui.memory_mut(|m| m.open_popup(id));
        } else if ui.memory(|m| m.is_popup_open(id)) {
            ui.memory_mut(|m| m.close_popup());
        }
    } else if !was_open && desired {
        ui.memory_mut(|m| m.open_popup(id));
    }
}

pub(in crate::components::egui::widgets) fn requested(
    response: &egui::Response,
    trigger: &str,
) -> bool {
    match trigger {
        "hover" => response.hovered(),
        "contextMenu" => response.secondary_clicked(),
        _ => response.clicked(),
    }
}

pub(in crate::components::egui::widgets) fn close_on_leave(
    ui: &egui::Ui,
    response: &egui::Response,
    popup: Option<egui::Rect>,
    trigger: &str,
) {
    if trigger == "hover"
        && let Some(popup) = popup
        && ui
            .input(|i| i.pointer.hover_pos())
            .is_some_and(|pos| !response.rect.contains(pos) && !popup.expand(8.).contains(pos))
    {
        ui.memory_mut(|m| m.close_popup());
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::components::egui::widgets) fn finish(
    ui: &egui::Ui,
    n: &v02::Node,
    _state: &crate::binding::State,
    memory: &mut Memory,
    collector: &mut Collector<'_>,
    id: egui::Id,
    controlled: bool,
    desired: bool,
) {
    let shown = ui.memory(|m| m.is_popup_open(id));
    if shown != memory.was_open && (!controlled || !shown && desired) {
        emit(ui, n, memory, collector, "openChange", json!(shown));
    }
    memory.was_open = shown;
}
