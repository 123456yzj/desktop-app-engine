use super::*;
use chrono::Timelike;
component!(shared::date::PROPERTIES, &["change", "openChange", "clear"]);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        rect,
        state,
        value,
        collector,
        ..
    } = c;
    let time = value
        .as_str()
        .and_then(|s| chrono::NaiveTime::parse_from_str(s, "%H:%M:%S").ok())
        .unwrap_or_default();
    let (mut h, mut m, mut s) = (time.hour(), time.minute(), time.second());
    let old = (h, m, s);
    let twelve = contract::flag(n, state, "use12Hours", false);
    let mut pm = h >= 12;
    if twelve {
        h = if h % 12 == 0 { 12 } else { h % 12 };
    }
    let mut now = false;
    let mut clear = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = rect.height().min(40.);
        ui.add(
            egui::DragValue::new(&mut h)
                .range(if twelve { 1..=12 } else { 0..=23 })
                .custom_formatter(|n, _| format!("{:02}", n as u32))
                .speed(contract::number(n, state, "hourStep", 1.)),
        );
        ui.label(":");
        ui.add(
            egui::DragValue::new(&mut m)
                .range(0..=59)
                .custom_formatter(|n, _| format!("{:02}", n as u32))
                .speed(contract::number(n, state, "minuteStep", 1.)),
        );
        ui.label(":");
        ui.add(
            egui::DragValue::new(&mut s)
                .range(0..=59)
                .custom_formatter(|n, _| format!("{:02}", n as u32))
                .speed(contract::number(n, state, "secondStep", 1.)),
        );
        if twelve {
            egui::ComboBox::from_id_salt("period")
                .selected_text(if pm { "PM" } else { "AM" })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut pm, false, "AM");
                    ui.selectable_value(&mut pm, true, "PM");
                });
        }
        now = contract::flag(n, state, "showNow", true) && ui.button("此刻").clicked();
        clear = contract::flag(n, state, "allowClear", true)
            && value != ""
            && ui.button(contract::tr(n, state, "清除", "Clear")).clicked();
    });
    if twelve {
        h = h % 12 + if pm { 12 } else { 0 };
    }
    if old != (h, m, s) {
        *value = json!(format!("{h:02}:{m:02}:{s:02}"));
    }
    if now {
        *value = json!(chrono::Local::now().format("%H:%M:%S").to_string());
    }
    if clear {
        *value = json!("");
        collector.push(ui, "clear", None, Value::Null);
    }
    if n.api.contains_key("format") {
        ui.weak(shared::date::format(
            value.as_str().unwrap_or(""),
            contract::string(n, state, "format", "HH:mm:ss"),
        ));
    }
}
