use super::*;

component!(
    &[
        ("items", Items),
        ("trigger", Enum(&["click", "hover", "contextMenu"])),
        (
            "placement",
            Enum(&["bottomLeft", "bottomRight", "topLeft", "topRight"])
        ),
        ("open", Bool)
    ],
    &["click", "openChange"]; None, &[("openChange", "open")]
);

pub(super) fn paint(c: WidgetContext<'_, '_>) {
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
    let response = ui.add_sized(
        rect.size(),
        egui::Button::new(if r.text.is_empty() {
            "菜单 ▾"
        } else {
            &r.text
        }),
    );
    let id = egui::Id::new((&n.id, "popup"));
    let trigger = contract::string(n, state, "trigger", "click");
    let controlled = matches!(n.api.get("open"), Some(Binding::State { .. }));
    let desired = local(n, state, memory, "open", json!(false)) == true;
    shared::popup::sync(ui, id, controlled, desired, memory.was_open);
    if shared::popup::requested(&response, trigger) && !ui.memory(|m| m.is_popup_open(id)) {
        if controlled {
            emit(ui, n, memory, collector, "openChange", json!(true));
        } else {
            ui.memory_mut(|m| m.open_popup(id));
        }
    }
    let mut selected = vec![];
    let mut open = vec![];
    let direction = if contract::string(n, state, "placement", "bottomLeft").starts_with("top") {
        egui::AboveOrBelow::Above
    } else {
        egui::AboveOrBelow::Below
    };
    let style = ui.style().clone();
    let popup = egui::popup::popup_above_or_below_widget(
        ui,
        id,
        &response,
        direction,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_style(style);
            shared::menu::paint(
                ui,
                n,
                state,
                contract::items(n, state),
                &mut selected,
                &mut open,
                collector,
                0,
            );
            if !selected.is_empty() {
                ui.memory_mut(|m| m.close_popup());
            }
            ui.min_rect()
        },
    );
    shared::popup::close_on_leave(ui, &response, popup, trigger);
    shared::popup::finish(ui, n, state, memory, collector, id, controlled, desired);
}
