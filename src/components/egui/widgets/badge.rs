use super::*;
component!(
    &[
        ("closable", Bool),
        ("checked", Bool),
        ("checkable", Bool),
        ("icon", Text)
    ],
    &["close", "change"]; None, &[("change", "checked")]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        rect: content,
        state,
        memory,
        collector,
        ..
    } = c;
    if memory.dismissed {
        return;
    }
    let painter = ui.painter().clone();
    let checked = local(n, state, memory, "checked", json!(true)) == true;
    let tone = match n.tone.unwrap_or_default() {
        Tone::Accent => Token::Accent,
        Tone::Success => Token::Success,
        Tone::Warning => Token::Warning,
        Tone::Danger => Token::Danger,
        Tone::Muted => Token::TextMuted,
    };
    painter.rect_filled(
        content,
        n.style.radius,
        n.style
            .background
            .map(|c| token(ui, c))
            .unwrap_or_else(|| token(ui, tone).gamma_multiply(if checked { 0.15 } else { 0.04 })),
    );
    let closable = contract::flag(n, state, "closable", false);
    let tag_rect = if closable {
        egui::Rect::from_min_max(content.min, content.max - Vec2::new(20., 0.))
    } else {
        content
    };
    if r.enabled
        && contract::flag(n, state, "checkable", false)
        && ui
            .interact(
                tag_rect,
                egui::Id::new((&n.id, "tag")),
                egui::Sense::click(),
            )
            .clicked()
    {
        memory.properties.insert("checked".into(), json!(!checked));
        collector.push(ui, "change", None, json!(!checked));
    }
    if closable
        && ui
            .add_enabled_ui(r.enabled, |ui| {
                ui.put(
                    egui::Rect::from_min_max(
                        Pos2::new(tag_rect.right(), content.top()),
                        content.max,
                    ),
                    egui::Button::new("×").frame(false),
                )
            })
            .inner
            .clicked()
    {
        memory.dismissed = true;
        collector.push(ui, "close", None, Value::Null);
    }
    let mut style = n.style.clone();
    style.text_wrap = Some(false);
    style.text_align = Some(protocol::TextAlign::Center);
    styled_label(
        ui,
        &painter,
        tag_rect.shrink2(Vec2::new(
            n.style.padding_x.unwrap_or(10.),
            n.style.padding_y.unwrap_or(2.),
        )),
        &format!("{}{}", contract::string(n, state, "icon", ""), r.text),
        &style,
        token(ui, n.style.color.unwrap_or(tone)),
    );
}
