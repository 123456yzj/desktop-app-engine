use super::*;
use chrono::Timelike;
component!(shared::date::PROPERTIES, &["change", "openChange", "clear"]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        value,
        collector,
        ..
    } = c;
    let source = value.as_str().unwrap_or("");
    let display = if source.is_empty() {
        if r.placeholder.is_empty() {
            "选择日期".to_owned()
        } else {
            r.placeholder.clone()
        }
    } else {
        shared::date::format(
            source,
            contract::string(
                n,
                state,
                "format",
                if contract::flag(n, state, "showTime", false) {
                    "YYYY-MM-DD HH:mm:ss"
                } else {
                    "YYYY-MM-DD"
                },
            ),
        )
    };
    let response = ui.button(format!("{display} ▾"));
    let id = egui::Id::new((&n.id, "datePopup"));
    if response.clicked() {
        ui.memory_mut(|m| m.toggle_popup(id));
    }
    egui::popup::popup_above_or_below_widget(
        ui,
        id,
        &response,
        egui::AboveOrBelow::Below,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            let before = value.clone();
            shared::date::calendar(ui, n, state, memory, value);
            if contract::flag(n, state, "showTime", false) {
                if value.as_str().is_some_and(|s| s.len() == 10) {
                    *value = json!(format!("{} 00:00:00", value.as_str().unwrap()));
                }
                let date = value
                    .as_str()
                    .unwrap_or("")
                    .get(..10)
                    .unwrap_or("")
                    .to_owned();
                let time = value
                    .as_str()
                    .and_then(|s| s.get(11..))
                    .unwrap_or("00:00:00");
                let mut time =
                    chrono::NaiveTime::parse_from_str(time, "%H:%M:%S").unwrap_or_default();
                let (mut h, mut m, mut s) = (time.hour(), time.minute(), time.second());
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut h).range(0..=23));
                    ui.add(egui::DragValue::new(&mut m).range(0..=59));
                    ui.add(egui::DragValue::new(&mut s).range(0..=59));
                });
                if !date.is_empty() {
                    time = chrono::NaiveTime::from_hms_opt(h, m, s).unwrap();
                    *value = json!(format!("{date} {}", time.format("%H:%M:%S")));
                }
                if ui.button("确定").clicked() {
                    ui.memory_mut(|m| m.close_popup());
                }
            } else if *value != before {
                ui.memory_mut(|m| m.close_popup());
            }
        },
    );
    let open = ui.memory(|m| m.is_popup_open(id));
    if open != memory.was_open {
        emit(ui, n, memory, collector, "openChange", json!(open));
        memory.was_open = open;
    }
    if contract::flag(n, state, "allowClear", true)
        && value != ""
        && ui.small_button("清除").clicked()
    {
        *value = json!("");
        collector.push(ui, "clear", None, Value::Null);
    }
}
