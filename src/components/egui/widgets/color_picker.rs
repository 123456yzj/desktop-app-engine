use super::*;
component!(
    &[
        ("disabledAlpha", Bool),
        ("showText", Bool),
        ("allowClear", Bool),
        ("presets", Array)
    ],
    &["change", "openChange", "clear"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        state,
        value,
        collector,
        ..
    } = c;
    let text = value.as_str().unwrap_or("#1677ff");
    let mut rgba = parse_color(text);
    let old = rgba;
    ui.horizontal(|ui| {
        if contract::flag(n, state, "disabledAlpha", false) {
            let mut rgb = [rgba[0], rgba[1], rgba[2]];
            ui.color_edit_button_srgb(&mut rgb);
            rgba = [rgb[0], rgb[1], rgb[2], 255];
        } else {
            ui.color_edit_button_srgba_unmultiplied(&mut rgba);
        }
        if contract::flag(n, state, "showText", false) {
            ui.monospace(text);
        }
    });
    if old != rgba {
        *value = json!(format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            rgba[0], rgba[1], rgba[2], rgba[3]
        ));
    }
    if let Some(presets) = contract::property(n, state, "presets").and_then(Value::as_array) {
        ui.horizontal_wrapped(|ui| {
            for color in presets.iter().filter_map(Value::as_str) {
                let rgba = parse_color(color);
                if ui
                    .add(
                        egui::Button::new("  ").fill(Color32::from_rgba_unmultiplied(
                            rgba[0], rgba[1], rgba[2], rgba[3],
                        )),
                    )
                    .clicked()
                {
                    *value = json!(color);
                }
            }
        });
    }
    if contract::flag(n, state, "allowClear", false) && ui.small_button("清除").clicked() {
        *value = json!("");
        collector.push(ui, "clear", None, Value::Null);
    }
}
fn parse_color(text: &str) -> [u8; 4] {
    match serde_json::from_value::<Token>(json!(text)) {
        Ok(Token::Hex(rgba)) => rgba,
        _ => [22, 119, 255, 255],
    }
}
