use super::*;
component!(
    &[
        ("items", Items),
        ("activeKey", Array),
        ("defaultActiveKey", Array),
        ("accordion", Bool),
        ("bordered", Bool),
        ("ghost", Bool)
    ],
    &["change"]; Some("activeKey"), &[]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        state,
        value,
        ..
    } = c;
    let mut keys = strings(value);
    for item in contract::items(n, state) {
        let key = item_key(item);
        let text = format!(
            "{} {}",
            if keys.iter().any(|k| k == key) {
                "▾"
            } else {
                "▸"
            },
            item_label(item)
        );
        if ui
            .add_enabled(!item_disabled(item), egui::Button::new(text))
            .clicked()
        {
            let was_open = keys.iter().any(|k| k == key);
            if contract::flag(n, state, "accordion", false) {
                keys.clear();
            }
            if !was_open {
                keys.push(key.to_owned());
            } else {
                keys.retain(|k| k != key);
            }
        }
    }
    *value = json!(keys);
}
