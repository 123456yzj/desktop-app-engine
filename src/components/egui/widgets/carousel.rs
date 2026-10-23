use super::*;
component!(
    &[
        ("items", Items),
        ("current", Integer(0, 10000)),
        ("autoplay", Bool),
        ("autoplaySpeed", Integer(100, 3600000)),
        ("dots", Bool),
        ("arrows", Bool),
        ("infinite", Bool)
    ],
    &["change"]; Some("current"), &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        value,
        ..
    } = c;
    let children = r
        .api
        .get("@childCount")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let count = children.max(contract::items(n, state).len());
    if count == 0 {
        return;
    }
    let mut index = value.as_u64().unwrap_or(0) as usize % count;
    let now = ui.input(|i| i.time);
    let start = memory.started.get_or_insert(now);
    if contract::flag(n, state, "autoplay", false) {
        if now - *start >= contract::number(n, state, "autoplaySpeed", 3000.) / 1000. {
            index = (index + 1) % count;
            *start = now;
        }
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));
    }
    ui.horizontal(|ui| {
        if contract::flag(n, state, "arrows", true) && ui.button("‹").clicked() {
            index = if index == 0 {
                if contract::flag(n, state, "infinite", true) {
                    count - 1
                } else {
                    0
                }
            } else {
                index - 1
            };
        }
        if contract::flag(n, state, "dots", true) {
            for dot in 0..count {
                if ui.selectable_label(dot == index, "●").clicked() {
                    index = dot;
                }
            }
        }
        if contract::flag(n, state, "arrows", true) && ui.button("›").clicked() {
            index = if index + 1 == count {
                if contract::flag(n, state, "infinite", true) {
                    0
                } else {
                    index
                }
            } else {
                index + 1
            };
        }
    });
    if children == 0
        && let Some(item) = contract::items(n, state).get(index)
    {
        ui.heading(item_label(item));
        if let Some(text) = item["description"].as_str() {
            ui.label(text);
        }
    }
    *value = json!(index);
}
