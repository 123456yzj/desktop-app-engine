use super::*;
component!(
    &[
        ("allowClear", Bool),
        ("password", Bool),
        ("multiline", Bool),
        ("rows", Integer(1, 100)),
        ("maxLength", Integer(1, 1048576)),
        ("readOnly", Bool),
        ("prefix", Text),
        ("suffix", Text),
        ("showCount", Bool),
        ("status", Enum(&["default", "error", "warning"]))
    ],
    &["change", "submit", "focus", "blur", "clear"]
);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    n: &protocol::Node,
    r: &Resolved,
    content: egui::Rect,
    state: &crate::binding::State,
    text: &mut String,
    collector: &mut Collector<'_>,
) -> bool {
    control_theme(ui, &n.style, content.height());
    let painter = ui.painter().clone();
    let font = n.style.font_size.unwrap_or(16.);
    let color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    let clear = contract::flag(n, state, "allowClear", false);
    let readonly = contract::flag(n, state, "readOnly", false);
    let prefix = contract::string(n, state, "prefix", "");
    let suffix = contract::string(n, state, "suffix", "");
    let prefix_width = prefix.chars().count() as f32 * font * 0.65;
    let suffix_width = suffix.chars().count() as f32 * font * 0.65;
    let input_rect = egui::Rect::from_min_max(
        content.min + Vec2::new(prefix_width, 0.),
        Pos2::new(
            (content.right() - suffix_width - if clear { 28. } else { 0. })
                .max(content.left() + prefix_width + 1.),
            content.bottom(),
        ),
    );
    // TextEdit paints its background from extreme_bg_color, which cannot follow the state visuals.
    let hovered = r.enabled && ui.rect_contains_pointer(input_rect);
    let pressed = hovered && ui.input(|i| i.pointer.primary_down());
    let paint_state = if !r.enabled {
        n.style.disabled.as_ref()
    } else if pressed {
        n.style.pressed.as_ref()
    } else if hovered {
        n.style.hover.as_ref()
    } else {
        None
    };
    let edit_bg = paint_state
        .and_then(|s| s.background)
        .or(n.style.background)
        .map(|c| token(ui, c));
    let edit_fg = paint_state
        .and_then(|s| s.color)
        .or(n.style.color)
        .map(|c| token(ui, c))
        .unwrap_or(color);
    if clear {
        painter.rect_filled(
            content,
            n.style.radius,
            edit_bg.unwrap_or(ui.visuals().extreme_bg_color),
        );
    }
    for (label, x) in [
        (prefix, content.left()),
        (suffix, input_rect.right() + if clear { 28. } else { 0. }),
    ] {
        if !label.is_empty() {
            painter.text(
                Pos2::new(x, content.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                style_font(&n.style, ui.ctx()),
                color,
            );
        }
    }
    let response = ui
        .add_enabled_ui(r.enabled, |ui| {
            let edit = (if contract::flag(n, state, "multiline", false) {
                egui::TextEdit::multiline(text)
            } else {
                egui::TextEdit::singleline(text)
            })
            .password(contract::flag(n, state, "password", false))
            .interactive(!readonly)
            .text_color(edit_fg)
            .font(style_font(&n.style, ui.ctx()))
            .id(egui::Id::new((&n.id, "input")))
            .hint_text(&r.placeholder)
            .char_limit(contract::number(
                n,
                state,
                "maxLength",
                crate::MAX_INPUT_CHARS as f64,
            ) as usize);
            let edit = if let Some(bg) = edit_bg {
                edit.background_color(bg)
            } else {
                edit
            };
            let edit = if clear { edit.frame(false) } else { edit };
            let edit = if n.style.padding_x.is_some() || n.style.padding_y.is_some() {
                edit.margin(egui::Margin::symmetric(
                    n.style.padding_x.unwrap_or(4.).clamp(0., 127.) as i8,
                    n.style.padding_y.unwrap_or(2.).clamp(0., 127.) as i8,
                ))
            } else {
                edit
            };
            ui.put(input_rect, edit)
        })
        .inner;
    let cleared = clear
        && !readonly
        && ui
            .add_enabled_ui(r.enabled, |ui| {
                ui.put(
                    egui::Rect::from_min_size(
                        Pos2::new(input_rect.right(), content.top()),
                        Vec2::new(28., content.height()),
                    ),
                    egui::Button::new(
                        egui::RichText::new("×")
                            .size(16.)
                            .color(token(ui, Token::TextMuted)),
                    )
                    .frame(false),
                )
            })
            .inner
            .clicked();
    let status = contract::string(n, state, "status", "default");
    let status_error = matches!(status, "error" | "warning");
    if status_error {
        painter.rect_stroke(
            if clear { content } else { input_rect },
            3.,
            Stroke::new(
                1_f32,
                token(
                    ui,
                    if status == "error" {
                        Token::Danger
                    } else {
                        Token::Warning
                    },
                ),
            ),
            StrokeKind::Inside,
        );
    }
    // TextEdit paints its own frame from the widget visuals, so base/state colors and
    // radius would be lost without a matching stroke. The focus ring wins while focused,
    // except for an explicit pressed style, which is transient and must stay visible.
    if !status_error
        && (!response.has_focus() || pressed || clear)
        && let Some(border) = paint_state
            .and_then(|s| s.border_color)
            .or(n.style.border_color)
            .or(clear.then_some(Token::Border))
    {
        painter.rect_stroke(
            if clear { content } else { input_rect },
            n.style.radius,
            Stroke::new(
                n.style.border_width,
                if clear && response.has_focus() {
                    token(ui, Token::Primary)
                } else {
                    token(ui, border)
                },
            ),
            StrokeKind::Inside,
        );
    }
    if cleared {
        text.clear();
    }
    if response.changed() || cleared {
        collector.push(ui, "change", None, json!(text));
    }
    if cleared {
        collector.push(ui, "clear", None, Value::Null);
    }
    if response.gained_focus() {
        collector.push(ui, "focus", None, Value::Null);
    }
    if response.lost_focus() {
        collector.push(ui, "blur", None, Value::Null);
    }
    if contract::flag(n, state, "showCount", false) {
        painter.text(
            content.right_bottom(),
            egui::Align2::RIGHT_BOTTOM,
            format!(
                "{} / {}",
                text.chars().count(),
                contract::number(n, state, "maxLength", crate::MAX_INPUT_CHARS as f64)
            ),
            FontId::proportional(10.),
            token(ui, Token::TextMuted),
        );
    }
    let submit = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if submit {
        collector.push(ui, "submit", None, Value::Null);
    }
    submit
}
