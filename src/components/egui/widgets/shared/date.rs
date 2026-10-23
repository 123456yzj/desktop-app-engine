use super::*;
use chrono::{Datelike, NaiveDate};
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("format", Text),
    ("minDate", Text),
    ("maxDate", Text),
    ("allowClear", Bool),
    ("showTime", Bool),
    ("showNow", Bool),
    ("showToday", Bool),
    ("use12Hours", Bool),
    ("hourStep", Integer(1, 23)),
    ("minuteStep", Integer(1, 59)),
    ("secondStep", Integer(1, 59)),
    (
        "picker",
        Enum(&["date", "week", "month", "quarter", "year"]),
    ),
    ("fullscreen", Bool),
];
pub(in crate::components::egui::widgets) fn calendar(
    ui: &mut egui::Ui,
    n: &v02::Node,
    state: &crate::binding::State,
    memory: &mut Memory,
    value: &mut Value,
) {
    let date = value
        .as_str()
        .and_then(|s| NaiveDate::parse_from_str(s.get(..10).unwrap_or(s), "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Local::now().date_naive());
    let mut year = memory
        .properties
        .get("year")
        .and_then(Value::as_i64)
        .unwrap_or(i64::from(date.year())) as i32;
    let mut month = memory
        .properties
        .get("month")
        .and_then(Value::as_u64)
        .unwrap_or(u64::from(date.month())) as u32;
    ui.horizontal(|ui| {
        if ui.small_button("‹").clicked() {
            if month == 1 {
                month = 12;
                year -= 1;
            } else {
                month -= 1;
            }
        }
        ui.add(egui::DragValue::new(&mut year).range(1..=9998));
        egui::ComboBox::from_id_salt("month")
            .selected_text(format!("{month} 月"))
            .show_ui(ui, |ui| {
                for m in 1..=12 {
                    ui.selectable_value(&mut month, m, format!("{m} 月"));
                }
            });
        if ui.small_button("›").clicked() {
            if month == 12 {
                month = 1;
                year += 1;
            } else {
                month += 1;
            }
        }
    });
    year = year.clamp(1, 9998);
    memory.properties.insert("year".into(), json!(year));
    memory.properties.insert("month".into(), json!(month));
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let offset = first.weekday().num_days_from_monday() as i64;
    let min = contract::property(n, state, "minDate")
        .and_then(Value::as_str)
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let max = contract::property(n, state, "maxDate")
        .and_then(Value::as_str)
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let picker = contract::string(n, state, "picker", "date");
    if matches!(picker, "month" | "quarter" | "year") {
        egui::Grid::new("periods").num_columns(3).show(ui, |ui| {
            let count = if picker == "quarter" { 4 } else { 12 };
            for index in 0..count {
                let (y, m, label) = match picker {
                    "year" => (
                        year / 12 * 12 + index,
                        1,
                        (year / 12 * 12 + index).to_string(),
                    ),
                    "quarter" => (year, index * 3 + 1, format!("Q{}", index + 1)),
                    _ => (year, index + 1, format!("{} 月", index + 1)),
                };
                let Some(day) = NaiveDate::from_ymd_opt(y, m as u32, 1) else {
                    continue;
                };
                if ui
                    .add_enabled(
                        min.is_none_or(|min| day >= min) && max.is_none_or(|max| day <= max),
                        egui::Button::new(label),
                    )
                    .clicked()
                {
                    *value = json!(day.format("%Y-%m-%d").to_string());
                }
                if index % 3 == 2 {
                    ui.end_row();
                }
            }
        });
        return;
    }
    egui::Grid::new("calendar")
        .num_columns(7)
        .spacing(Vec2::new(4., 4.))
        .show(ui, |ui| {
            for day in ["一", "二", "三", "四", "五", "六", "日"] {
                ui.label(day);
            }
            ui.end_row();
            for index in 0..42 {
                let day = first + chrono::Duration::days(i64::from(index) - offset);
                let enabled = day.month() == month
                    && min.is_none_or(|min| day >= min)
                    && max.is_none_or(|max| day <= max);
                if ui
                    .add_enabled(
                        enabled,
                        egui::SelectableLabel::new(day == date, day.day().to_string()),
                    )
                    .clicked()
                {
                    let day = if picker == "week" {
                        day - chrono::Duration::days(i64::from(
                            day.weekday().num_days_from_monday(),
                        ))
                    } else {
                        day
                    };
                    if min.is_none_or(|min| day >= min) && max.is_none_or(|max| day <= max) {
                        *value = json!(day.format("%Y-%m-%d").to_string());
                    }
                }
                if index % 7 == 6 {
                    ui.end_row();
                }
            }
        });
    if contract::flag(n, state, "showToday", true) && ui.button("今天").clicked() {
        let today = chrono::Local::now().date_naive();
        if min.is_none_or(|min| today >= min) && max.is_none_or(|max| today <= max) {
            *value = json!(today.format("%Y-%m-%d").to_string());
            memory.properties.remove("year");
            memory.properties.remove("month");
        }
    }
}
pub(in crate::components::egui::widgets) fn format(value: &str, format: &str) -> String {
    let format = format
        .replace("YYYY", "%Y")
        .replace("MM", "%m")
        .replace("DD", "%d")
        .replace("HH", "%H")
        .replace("hh", "%I")
        .replace("mm", "%M")
        .replace("ss", "%S")
        .replace('A', "%p");
    let items = chrono::format::StrftimeItems::new(&format).collect::<Vec<_>>();
    if items
        .iter()
        .any(|i| matches!(i, chrono::format::Item::Error))
    {
        return value.to_owned();
    }
    let date = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
        .or_else(|| {
            chrono::NaiveTime::parse_from_str(value, "%H:%M:%S")
                .ok()
                .map(|t| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap().and_time(t))
        });
    let Some(date) = date else {
        return value.to_owned();
    };
    let mut output = String::new();
    if std::fmt::write(
        &mut output,
        format_args!("{}", date.format_with_items(items.iter())),
    )
    .is_ok()
    {
        output
    } else {
        value.to_owned()
    }
}
