use super::*;
component!(
    &[
        ("treeData", Items),
        ("selectedKeys", Array),
        ("checkedKeys", Array),
        ("expandedKeys", Array),
        ("defaultExpandedKeys", Array),
        ("defaultExpandAll", Bool),
        ("multiple", Bool),
        ("checkable", Bool),
        ("checkStrictly", Bool),
        ("showLine", Bool),
        ("selectable", Bool)
    ],
    &["change", "select", "check", "expand"]; Some("selectedKeys"), &[("check", "checkedKeys"), ("expand", "expandedKeys")]
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
    let mut selected = strings(value);
    let mut checked = strings(&local(n, state, memory, "checkedKeys", json!([])));
    let mut expanded = strings(&local(
        n,
        state,
        memory,
        "expandedKeys",
        contract::property(n, state, "defaultExpandedKeys")
            .cloned()
            .unwrap_or(json!([])),
    ));
    if memory.started.is_none() {
        memory.started = Some(0.);
        if !n.api.contains_key("expandedKeys")
            && contract::flag(n, state, "defaultExpandAll", false)
        {
            collect_keys(contract::items(n, state), &mut expanded, false);
            memory
                .properties
                .insert("expandedKeys".into(), json!(expanded));
        }
    }
    let old_checked = checked.clone();
    let old_expanded = expanded.clone();
    egui::ScrollArea::vertical()
        .max_height(rect.height())
        .show(ui, |ui| {
            tree(
                ui,
                n,
                state,
                contract::items(n, state),
                &mut selected,
                &mut checked,
                &mut expanded,
                collector,
                0,
            )
        });
    if old_checked != checked && !contract::flag(n, state, "checkStrictly", false) {
        reconcile_checks(contract::items(n, state), &mut checked);
    }
    *value = json!(selected);
    if old_checked != checked {
        emit(ui, n, memory, collector, "check", json!(checked));
    }
    if old_expanded != expanded {
        emit(ui, n, memory, collector, "expand", json!(expanded));
    }
}
fn reconcile_checks(items: &[Value], checked: &mut Vec<String>) {
    for item in items {
        if item_disabled(item) {
            continue;
        }
        if let Some(children) = item["children"].as_array().filter(|c| !c.is_empty()) {
            reconcile_checks(children, checked);
            let all = children
                .iter()
                .filter(|c| !item_disabled(c))
                .all(|c| checked.iter().any(|k| k == item_key(c)));
            checked.retain(|k| k != item_key(item));
            if all {
                checked.push(item_key(item).into());
            }
        }
    }
}
fn collect_keys(items: &[Value], keys: &mut Vec<String>, skip_disabled: bool) {
    for item in items {
        if skip_disabled && item_disabled(item) {
            continue;
        }
        let key = item_key(item).to_owned();
        if !keys.contains(&key) {
            keys.push(key);
        }
        if let Some(children) = item["children"].as_array() {
            collect_keys(children, keys, skip_disabled);
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn tree(
    ui: &mut egui::Ui,
    n: &v02::Node,
    state: &crate::binding::State,
    items: &[Value],
    selected: &mut Vec<String>,
    checked: &mut Vec<String>,
    expanded: &mut Vec<String>,
    collector: &mut Collector<'_>,
    depth: usize,
) {
    if depth > 32 {
        return;
    }
    for item in items {
        let key = item_key(item);
        ui.add_enabled_ui(!item_disabled(item), |ui| {
            ui.horizontal(|ui| {
                if item["children"].as_array().is_some_and(|a| !a.is_empty())
                    && ui
                        .small_button(if expanded.iter().any(|k| k == key) {
                            "▾"
                        } else {
                            "▸"
                        })
                        .clicked()
                {
                    toggle(expanded, key);
                }
                if contract::flag(n, state, "checkable", false) {
                    let mut on = checked.iter().any(|k| k == key);
                    if ui.checkbox(&mut on, "").changed() {
                        let mut keys = vec![key.into()];
                        if !contract::flag(n, state, "checkStrictly", false)
                            && let Some(children) = item["children"].as_array()
                        {
                            collect_keys(children, &mut keys, true);
                        }
                        for key in keys {
                            checked.retain(|x| x != &key);
                            if on {
                                checked.push(key);
                            }
                        }
                    }
                }
                if ui
                    .selectable_label(selected.iter().any(|k| k == key), item_label(item))
                    .clicked()
                    && contract::flag(n, state, "selectable", true)
                {
                    if contract::flag(n, state, "multiple", false) {
                        toggle(selected, key);
                    } else {
                        *selected = vec![key.into()];
                    }
                    collector.push(ui, "select", None, json!(selected));
                }
            });
            if expanded.iter().any(|k| k == key)
                && let Some(children) = item["children"].as_array()
            {
                ui.indent(key, |ui| {
                    tree(
                        ui,
                        n,
                        state,
                        children,
                        selected,
                        checked,
                        expanded,
                        collector,
                        depth + 1,
                    )
                });
            }
        });
    }
}
