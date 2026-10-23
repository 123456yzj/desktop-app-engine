use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("multiple", Bool),
    ("allowClear", Bool),
    ("showSearch", Bool),
    ("changeOnSelect", Bool),
    ("treeDefaultExpandAll", Bool),
    ("treeCheckable", Bool),
];
pub(in crate::components::egui::widgets) fn paint(c: WidgetContext<'_, '_>, cascader: bool) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        memory,
        value,
        collector,
        ..
    } = c;
    let options = options(n, state);
    let multiple = contract::flag(n, state, "multiple", false)
        || contract::flag(n, state, "treeCheckable", false);
    let mut selected = if cascader && multiple {
        value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| p.as_array().and_then(|a| a.last()).and_then(Value::as_str))
            .map(str::to_owned)
            .collect()
    } else if cascader || multiple {
        strings(value)
    } else {
        vec![value.as_str().unwrap_or("").to_owned()]
    };
    let display = if selected.is_empty() {
        r.placeholder.clone()
    } else {
        selected.join(" / ")
    };
    let response = egui::ComboBox::from_id_salt("treeSelect")
        .width((rect.width() - 8.).max(1.))
        .selected_text(display)
        .show_ui(ui, |ui| {
            if contract::flag(n, state, "showSearch", false) {
                ui.text_edit_singleline(&mut memory.search);
            }
            tree_options(
                ui,
                options,
                n,
                state,
                &memory.search,
                &mut selected,
                &mut vec![],
                multiple,
                0,
            );
        });
    let open = response.inner.is_some();
    if open != memory.was_open {
        emit(ui, n, memory, collector, "openChange", json!(open));
        memory.was_open = open;
    }
    if contract::flag(n, state, "allowClear", true)
        && !selected.is_empty()
        && ui.small_button("清除").clicked()
    {
        selected.clear();
        collector.push(ui, "clear", None, Value::Null);
    }
    *value = if cascader && multiple {
        json!(
            selected
                .iter()
                .filter_map(|key| option_path(options, key, &[]))
                .collect::<Vec<_>>()
        )
    } else if cascader || multiple {
        json!(selected)
    } else {
        json!(selected.first().cloned().unwrap_or_default())
    };
}
#[allow(clippy::too_many_arguments)]
fn tree_options(
    ui: &mut egui::Ui,
    options: &[Value],
    n: &v02::Node,
    state: &crate::binding::State,
    search: &str,
    selected: &mut Vec<String>,
    path: &mut Vec<String>,
    multiple: bool,
    depth: usize,
) {
    if depth > 32 {
        return;
    }
    for option in options {
        let (key, label, disabled) = option_parts(option);
        path.push(key.into());
        if let Some(children) = option["children"].as_array().filter(|a| !a.is_empty()) {
            ui.add_enabled_ui(!disabled, |ui| {
                ui.collapsing(label, |ui| {
                    if contract::flag(n, state, "changeOnSelect", false)
                        && ui
                            .selectable_label(selected.contains(&key.to_owned()), label)
                            .clicked()
                    {
                        *selected = if n.kind == Kind::Cascader {
                            path.clone()
                        } else {
                            vec![key.into()]
                        };
                    }
                    tree_options(
                        ui,
                        children,
                        n,
                        state,
                        search,
                        selected,
                        path,
                        multiple,
                        depth + 1,
                    );
                });
            });
        } else if label.to_lowercase().contains(&search.to_lowercase())
            && ui
                .add_enabled(
                    !disabled,
                    egui::SelectableLabel::new(selected.iter().any(|s| s == key), label),
                )
                .clicked()
        {
            if n.kind == Kind::Cascader && !multiple {
                *selected = path.clone();
            } else if multiple {
                toggle(selected, key);
            } else {
                *selected = vec![key.into()];
            }
            if !multiple {
                ui.close_menu();
            }
        }
        path.pop();
    }
}
fn option_path(options: &[Value], key: &str, parent: &[String]) -> Option<Vec<String>> {
    for option in options {
        let mut path = parent.to_vec();
        path.push(item_key(option).into());
        if item_key(option) == key {
            return Some(path);
        }
        if let Some(children) = option["children"].as_array()
            && let Some(path) = option_path(children, key, &path)
        {
            return Some(path);
        }
    }
    None
}
