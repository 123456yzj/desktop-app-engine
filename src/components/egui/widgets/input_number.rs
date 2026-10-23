use super::*;
component!(
    &[
        ("readOnly", Bool),
        ("controls", Bool),
        ("prefix", Text),
        ("suffix", Text)
    ],
    &["change", "submit"]
);
#[derive(Default)]
pub(in crate::components::egui) struct NumberEdit {
    pub(in crate::components::egui) text: String,
    value: Option<f64>,
}
fn display_number(node: &v02::Node, value: f64) -> String {
    if let Some(precision) = node.precision {
        format!("{value:.precision$}")
    } else {
        value.to_string()
    }
}
pub(in crate::components::egui) fn control(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
    edit: &mut NumberEdit,
) -> bool {
    let number = value.as_f64().unwrap();
    if edit.value != Some(number) {
        edit.text = display_number(node, number);
        edit.value = Some(number);
    }
    let readonly = resolved.api.get("readOnly") == Some(&json!(true));
    let controls = resolved.api.get("controls") != Some(&json!(false));
    let prefix = resolved
        .api
        .get("prefix")
        .and_then(Value::as_str)
        .unwrap_or("");
    let suffix = resolved
        .api
        .get("suffix")
        .and_then(Value::as_str)
        .unwrap_or("");
    let prefix_width = prefix.chars().count() as f32 * 10.;
    let suffix_width = suffix.chars().count() as f32 * 10.;
    for (text, x) in [(prefix, rect.left()), (suffix, rect.right() - suffix_width)] {
        ui.painter().text(
            Pos2::new(x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            style_font(&node.style, ui.ctx()),
            token(ui, Token::TextPrimary),
        );
    }
    let rect = egui::Rect::from_min_max(
        rect.min + Vec2::new(prefix_width, 0.),
        rect.max - Vec2::new(suffix_width, 0.),
    );
    let buttons = if controls {
        24_f32.min(rect.width() / 3.)
    } else {
        0.
    };
    let text_rect = egui::Rect::from_min_max(rect.min, rect.max - Vec2::new(buttons, 0.));
    let response = ui.put(
        text_rect,
        egui::TextEdit::singleline(&mut edit.text)
            .id(egui::Id::new((&node.id, "number")))
            .font(style_font(&node.style, ui.ctx()))
            .interactive(!readonly)
            .char_limit(64),
    );
    let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if response.changed()
        && let Ok(number) = edit.text.trim().parse::<f64>()
        && number.is_finite()
    {
        let number = node.normalize_number(number);
        *value = json!(number);
        edit.value = Some(number);
    }
    let upper = egui::Rect::from_min_max(
        Pos2::new(text_rect.right(), rect.top()),
        Pos2::new(rect.right(), rect.center().y),
    );
    let lower = egui::Rect::from_min_max(upper.left_bottom(), rect.max);
    let (min, max) = node.number_range();
    let current = value.as_f64().unwrap();
    ui.spacing_mut().interact_size.y = 0.;
    ui.spacing_mut().button_padding.y = 0.;
    let up = ui
        .add_enabled_ui(controls && !readonly && current < max, |ui| {
            ui.put(upper, egui::Button::new("+"))
        })
        .inner
        .clicked();
    let down = ui
        .add_enabled_ui(controls && !readonly && current > min, |ui| {
            ui.put(lower, egui::Button::new("−"))
        })
        .inner
        .clicked();
    let key = if response.has_focus() && !readonly {
        ui.input(|i| {
            i32::from(i.key_pressed(egui::Key::ArrowUp))
                - i32::from(i.key_pressed(egui::Key::ArrowDown))
        })
    } else {
        0
    };
    let delta = i32::from(up) - i32::from(down) + key;
    if delta != 0 {
        *value = json!(node.normalize_number(current + f64::from(delta) * node.step.unwrap_or(1.)));
        edit.value = value.as_f64();
        edit.text = display_number(node, edit.value.unwrap());
    }
    if response.lost_focus() {
        edit.text = display_number(node, value.as_f64().unwrap());
    }
    enter
}
