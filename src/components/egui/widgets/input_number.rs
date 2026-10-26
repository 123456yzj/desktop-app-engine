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
fn display_number(node: &protocol::Node, value: f64) -> String {
    if let Some(precision) = node.precision {
        format!("{value:.precision$}")
    } else {
        value.to_string()
    }
}
pub(in crate::components::egui) fn control(
    ui: &mut egui::Ui,
    node: &protocol::Node,
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
    let hovered = ui.rect_contains_pointer(rect);
    let colors = if !resolved.enabled {
        node.style.disabled.as_ref()
    } else if hovered {
        node.style.hover.as_ref()
    } else {
        None
    };
    let background = colors
        .and_then(|s| s.background)
        .or(node.style.background)
        .map(|c| token(ui, c))
        .unwrap_or(ui.visuals().extreme_bg_color);
    ui.painter()
        .rect_filled(rect, node.style.radius, background);
    let response = ui.put(
        text_rect,
        egui::TextEdit::singleline(&mut edit.text)
            .id(egui::Id::new((&node.id, "number")))
            .font(style_font(&node.style, ui.ctx()))
            .frame(false)
            .margin(egui::Margin::symmetric(12, 8))
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
    let border = if response.has_focus() {
        token(ui, Token::Primary)
    } else {
        colors
            .and_then(|s| s.border_color)
            .or(node.style.border_color)
            .map(|c| token(ui, c))
            .unwrap_or_else(|| token(ui, Token::Border))
    };
    ui.painter().rect_stroke(
        rect,
        node.style.radius,
        Stroke::new(node.style.border_width, border),
        StrokeKind::Inside,
    );
    if controls {
        let separator = token(ui, node.style.border_color.unwrap_or(Token::Border));
        ui.painter().line_segment(
            [upper.left_top(), lower.left_bottom()],
            Stroke::new(1_f32, separator),
        );
        ui.painter().line_segment(
            [upper.left_bottom(), upper.right_bottom()],
            Stroke::new(1_f32, separator),
        );
    }
    ui.spacing_mut().interact_size.y = 0.;
    ui.spacing_mut().button_padding.y = 0.;
    let up = ui
        .add_enabled_ui(controls && !readonly && current < max, |ui| {
            ui.put(
                upper,
                egui::Button::new(egui::RichText::new("+").size(12.)).frame(false),
            )
        })
        .inner
        .clicked();
    let down = ui
        .add_enabled_ui(controls && !readonly && current > min, |ui| {
            ui.put(
                lower,
                egui::Button::new(egui::RichText::new("−").size(12.)).frame(false),
            )
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
