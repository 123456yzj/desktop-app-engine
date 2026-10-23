use super::*;
component!(
    &[
        ("showSearch", Bool),
        ("titles", Array),
        ("oneWay", Bool),
        ("showSelectAll", Bool)
    ],
    &["change", "search", "selectChange"]
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
    let options = options(n, state);
    let mut target = strings(value);
    if contract::flag(n, state, "showSearch", false)
        && ui.text_edit_singleline(&mut memory.search).changed()
    {
        collector.push(ui, "search", None, json!(memory.search));
    }
    ui.columns(3, |cols| {
        let titles = contract::property(n, state, "titles");
        cols[0].label(
            titles
                .and_then(|t| t.get(0))
                .and_then(Value::as_str)
                .unwrap_or("可选项"),
        );
        cols[2].label(
            titles
                .and_then(|t| t.get(1))
                .and_then(Value::as_str)
                .unwrap_or("已选项"),
        );
        for (column, right) in [(0, false), (2, true)] {
            egui::ScrollArea::vertical()
                .id_salt(column)
                .max_height(rect.height() - 50.)
                .show(&mut cols[column], |ui| {
                    for option in options {
                        let (key, label, disabled) = option_parts(option);
                        if target.iter().any(|s| s == key) != right
                            || !label.to_lowercase().contains(&memory.search.to_lowercase())
                        {
                            continue;
                        }
                        let mut checked = memory.checks.contains(key);
                        if ui
                            .add_enabled(!disabled, egui::Checkbox::new(&mut checked, label))
                            .changed()
                        {
                            if checked {
                                memory.checks.insert(key.into());
                            } else {
                                memory.checks.remove(key);
                            }
                            collector.push(
                                ui,
                                "selectChange",
                                None,
                                json!(memory.checks.iter().collect::<Vec<_>>()),
                            );
                        }
                    }
                });
        }
        if cols[1].button("→").clicked() {
            for option in options {
                let (key, _, disabled) = option_parts(option);
                if !disabled && memory.checks.contains(key) && !target.iter().any(|s| s == key) {
                    target.push(key.into());
                }
            }
            memory.checks.clear();
        }
        if !contract::flag(n, state, "oneWay", false) && cols[1].button("←").clicked() {
            target.retain(|key| !memory.checks.contains(key));
            memory.checks.clear();
        }
    });
    *value = json!(target);
}
