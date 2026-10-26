use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(in crate::components::egui::widgets) enum Mode {
    Tabs,
    Segmented,
    Steps,
    Anchor,
    Breadcrumb,
}

pub(in crate::components::egui::widgets) fn paint(c: WidgetContext<'_, '_>, mode: Mode) {
    let WidgetContext {
        ui,
        n,
        state,
        value,
        collector,
        ..
    } = c;
    let options: Vec<Value> = if mode == Mode::Segmented {
        n.options
            .as_ref()
            .and_then(|b| b.resolve(state).ok())
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|v| {
                if v.is_string() {
                    json!({"key":v,"label":v})
                } else {
                    v.clone()
                }
            })
            .collect()
    } else {
        contract::items(n, state).to_vec()
    };
    let vertical = contract::string(n, state, "direction", "horizontal") == "vertical"
        || contract::flag(n, state, "vertical", false)
        || matches!(
            contract::string(n, state, "tabPlacement", "top"),
            "start" | "end"
        );
    let layout = if vertical {
        egui::Layout::top_down(egui::Align::Min)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    ui.with_layout(layout, |ui| {
        if mode == Mode::Tabs {
            ui.spacing_mut().item_spacing.x = contract::number(n, state, "tabBarGutter", 8.) as f32;
        }
        for (index, item) in options.iter().enumerate() {
            let key = item_key(item);
            let selected = if mode == Mode::Steps {
                value.as_u64().unwrap_or(0) as usize == index
            } else {
                value.as_str() == Some(key)
                    || value.as_str() == Some("")
                        && !item_disabled(item)
                        && !options[..index].iter().any(|i| !item_disabled(i))
            };
            if mode == Mode::Breadcrumb && index > 0 {
                ui.label(contract::string(n, state, "separator", "/"));
            }
            let label = if mode == Mode::Steps {
                format!(
                    "{}  {}",
                    index + contract::number(n, state, "initial", 0.) as usize + 1,
                    item_label(item)
                )
            } else if let Some(icon) = item["icon"].as_str() {
                format!("{icon}  {}", item_label(item))
            } else {
                item_label(item).to_owned()
            };
            let response = shared::navigation_item(
                ui,
                &label,
                selected && mode != Mode::Breadcrumb,
                !item_disabled(item),
                if mode == Mode::Tabs { 64. } else { 0. },
                if mode == Mode::Breadcrumb { 24. } else { 34. },
            );
            if mode == Mode::Tabs
                && selected
                && contract::string(n, state, "tabType", "line") == "line"
            {
                let line = egui::Rect::from_min_size(
                    response.rect.left_bottom() - Vec2::new(-12., 2.),
                    Vec2::new((response.rect.width() - 24.).max(0.), 2.),
                );
                ui.painter()
                    .rect_filled(line, 1., token(ui, Token::Primary));
            }
            if response.clicked() {
                match mode {
                    Mode::Steps => *value = json!(index),
                    Mode::Tabs => {
                        *value = json!(key);
                        collector.push(ui, "tabClick", None, json!(key));
                    }
                    Mode::Segmented => *value = json!(key),
                    _ => collector.push(
                        ui,
                        "click",
                        None,
                        json!({"key":key,"href":item["href"].as_str().unwrap_or("")}),
                    ),
                }
            }
            if let Some(description) = item["description"].as_str() {
                response.on_hover_text(description);
            }
            if mode == Mode::Tabs
                && contract::string(n, state, "tabType", "line") == "editable-card"
                && item["closable"].as_bool().unwrap_or(true)
                && ui.small_button("×").clicked()
            {
                collector.push(ui, "edit", None, json!({"action":"remove","key":key}));
            }
        }
        if mode == Mode::Tabs
            && contract::string(n, state, "tabType", "line") == "editable-card"
            && !contract::flag(n, state, "hideAdd", false)
            && ui.small_button("+").clicked()
        {
            collector.push(ui, "edit", None, json!({"action":"add","key":""}));
        }
    });
}
