use super::*;

component!(
    &[
        ("items", Items),
        ("mode", Enum(&["vertical", "horizontal", "inline"])),
        ("multiple", Bool),
        ("selectable", Bool),
        ("selectedKeys", Array),
        ("defaultSelectedKeys", Array),
        ("openKeys", Array),
        ("defaultOpenKeys", Array),
        ("inlineCollapsed", Bool),
        ("inlineIndent", Number(0., 128.)),
        ("theme", Enum(&["light", "dark"]))
    ],
    &["change", "click", "select", "deselect", "openChange"]; Some("selectedKeys"), &[("openChange", "openKeys")]
);

pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        rect,
        state,
        memory,
        value,
        collector,
        ..
    } = c;
    if contract::string(n, state, "theme", "light") == "dark" {
        ui.set_style(egui::Style {
            visuals: egui::Visuals::dark(),
            ..ui.style().as_ref().clone()
        });
    }
    let mut selected = strings(value);
    let mut open = strings(&local(
        n,
        state,
        memory,
        "openKeys",
        contract::property(n, state, "defaultOpenKeys")
            .cloned()
            .unwrap_or(json!([])),
    ));
    let old_open = open.clone();
    let items = contract::items(n, state);
    if contract::string(n, state, "mode", "vertical") == "horizontal" {
        ui.horizontal_wrapped(|ui| {
            shared::menu::paint(ui, n, state, items, &mut selected, &mut open, collector, 0)
        });
    } else {
        egui::ScrollArea::vertical()
            .max_height(rect.height())
            .show(ui, |ui| {
                shared::menu::paint(ui, n, state, items, &mut selected, &mut open, collector, 0)
            });
    }
    *value = json!(selected);
    if old_open != open {
        emit(ui, n, memory, collector, "openChange", json!(open));
    }
}
