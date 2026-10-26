use super::*;
component!(
    &[
        ("loading", Bool),
        ("checkedChildren", Text),
        ("unCheckedChildren", Text)
    ],
    &["change"]
);
pub(in crate::components::egui) fn control(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
) {
    let checked = value.as_bool().unwrap_or(false);
    let loading = resolved.api.get("loading") == Some(&Value::Bool(true));
    let response = ui.interact(
        rect,
        egui::Id::new((&node.id, "switch")),
        if loading {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        },
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::Checkbox,
            resolved.enabled && !loading,
            checked,
            &resolved.text,
        )
    });
    if response.clicked() {
        *value = json!(!checked);
        response.request_focus();
    }
    let track = egui::Rect::from_center_size(
        Pos2::new(rect.left() + 22., rect.center().y),
        Vec2::new(44., 24.),
    );
    let fill = if checked {
        token(ui, Token::Accent)
    } else {
        token(ui, Token::Border)
    };
    ui.painter().rect_filled(track, 12., fill);
    ui.painter().rect_stroke(
        track,
        12.,
        Stroke::new(
            1_f32,
            token(
                ui,
                if response.has_focus() {
                    Token::Focus
                } else {
                    Token::Border
                },
            ),
        ),
        StrokeKind::Inside,
    );
    let thumb = Pos2::new(
        track.left() + if checked { 32. } else { 12. },
        track.center().y,
    );
    ui.painter().circle_filled(thumb, 9., Color32::WHITE);
    if loading {
        ui.put(
            egui::Rect::from_center_size(thumb, Vec2::splat(12.)),
            egui::Spinner::new()
                .size(12.)
                .color(token(ui, Token::Accent)),
        );
    }
    label_line(
        ui,
        &ui.painter().clone(),
        egui::Rect::from_min_max(rect.min + Vec2::new(52., 0.), rect.max),
        resolved
            .api
            .get(if checked {
                "checkedChildren"
            } else {
                "unCheckedChildren"
            })
            .and_then(Value::as_str)
            .unwrap_or(&resolved.text),
        node.style.font_size.unwrap_or(16.),
        token(ui, node.style.color.unwrap_or(Token::TextPrimary)),
    );
}
