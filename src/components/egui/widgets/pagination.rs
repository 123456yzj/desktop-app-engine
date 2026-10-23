use super::*;

component!(
    &[
        ("current", Integer(1, 1000000000)),
        ("defaultCurrent", Integer(1, 1000000000)),
        ("pageSize", Integer(1, 1000000)),
        ("defaultPageSize", Integer(1, 1000000)),
        ("total", Integer(0, 1000000000)),
        ("pageSizeOptions", Array),
        ("hideOnSinglePage", Bool),
        ("showLessItems", Bool),
        ("showQuickJumper", Bool),
        ("showSizeChanger", Bool),
        ("showTotal", Bool),
        ("simple", Bool),
        ("align", Enum(&["start", "center", "end"]))
    ],
    &["change", "showSizeChange"]; Some("current"), &[("showSizeChange", "pageSize")]
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
    let total = contract::number(n, state, "total", 0.) as u64;
    let mut size = local(
        n,
        state,
        memory,
        "pageSize",
        json!(contract::number(n, state, "defaultPageSize", 10.)),
    )
    .as_u64()
    .unwrap_or(10)
    .max(1);
    let mut page = value.as_u64().unwrap_or(1);
    let old_size = size;
    let max_page = total.div_ceil(size).max(1);
    if contract::flag(n, state, "hideOnSinglePage", false) && max_page == 1 {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(page > 1, egui::Button::new("‹")).clicked() {
            page = page.saturating_sub(1).max(1);
        }
        if contract::flag(n, state, "simple", false) {
            ui.add(egui::DragValue::new(&mut page).range(1..=max_page));
            ui.label(format!("/ {max_page}"));
        } else {
            let radius = if contract::flag(n, state, "showLessItems", false) {
                1
            } else {
                2
            };
            let mut pages: Vec<u64> = (page.saturating_sub(radius).max(1)
                ..=page.saturating_add(radius).min(max_page))
                .collect();
            pages.extend([1, max_page]);
            pages.sort();
            pages.dedup();
            let mut previous = 0;
            for number in pages {
                if number > previous + 1 {
                    ui.label("…");
                }
                if ui
                    .selectable_label(page == number, number.to_string())
                    .clicked()
                {
                    page = number;
                }
                previous = number;
            }
        }
        if ui
            .add_enabled(page < max_page, egui::Button::new("›"))
            .clicked()
        {
            page = (page + 1).min(max_page);
        }
        if contract::flag(n, state, "showSizeChanger", total > 50) {
            let options = contract::property(n, state, "pageSizeOptions")
                .cloned()
                .unwrap_or(json!([10, 20, 50, 100]));
            egui::ComboBox::from_id_salt("pageSize")
                .selected_text(format!("{size} / 页"))
                .show_ui(ui, |ui| {
                    for option in options
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_u64)
                        .filter(|&x| x > 0)
                    {
                        ui.selectable_value(&mut size, option, option.to_string());
                    }
                });
        }
        if contract::flag(n, state, "showQuickJumper", false) {
            ui.label("跳至");
            ui.add(egui::DragValue::new(&mut page).range(1..=max_page));
        }
        if contract::flag(n, state, "showTotal", false) {
            ui.label(format!("共 {total} 条"));
        }
    });
    if old_size != size {
        page = page.min(total.div_ceil(size).max(1));
        emit(ui, n, memory, collector, "showSizeChange", json!(size));
    }
    *value = json!(page.clamp(1, total.div_ceil(size).max(1)));
}
