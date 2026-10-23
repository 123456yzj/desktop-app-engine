use super::*;
component!(
    &[("range", Bool), ("reverse", Bool), ("marks", Array)],
    &["change"]
);
pub(in crate::components::egui) fn control(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
) {
    if resolved.api.get("range") == Some(&json!(true)) {
        range_slider(ui, node, resolved, rect, value);
    } else {
        slider(ui, node, resolved, rect, value);
    }
    if let Some(marks) = resolved.api.get("marks").and_then(Value::as_array) {
        let (min, max) = node.number_range();
        for mark in marks {
            if let Some(v) = mark["value"].as_f64() {
                let mut t = ((v - min) / (max - min)) as f32;
                if resolved.api.get("reverse") == Some(&json!(true)) {
                    t = 1. - t;
                }
                let position = if node.vertical {
                    Pos2::new(rect.right(), rect.bottom() - 7. - t * (rect.height() - 14.))
                } else {
                    Pos2::new(rect.left() + 7. + t * (rect.width() - 14.), rect.bottom())
                };
                ui.painter().text(
                    position,
                    egui::Align2::CENTER_BOTTOM,
                    mark["label"].as_str().unwrap_or("•"),
                    FontId::proportional(10.),
                    token(ui, Token::TextMuted),
                );
            }
        }
    }
}
fn range_slider(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
) {
    let Some(array) = value.as_array() else {
        return;
    };
    if array.len() != 2 {
        return;
    }
    let mut values = [
        array[0].as_f64().unwrap_or(0.),
        array[1].as_f64().unwrap_or(0.),
    ];
    let before = values;
    let (min, max) = node.number_range();
    let reverse = resolved.api.get("reverse") == Some(&json!(true));
    let point = |v: f64| {
        let mut fraction = ((v - min) / (max - min)) as f32;
        if reverse {
            fraction = 1. - fraction;
        }
        if node.vertical {
            Pos2::new(
                rect.center().x,
                rect.bottom() - 7. - fraction * (rect.height() - 14.),
            )
        } else {
            Pos2::new(
                rect.left() + 7. + fraction * (rect.width() - 14.),
                rect.center().y,
            )
        }
    };
    ui.painter().line_segment(
        [point(min), point(max)],
        Stroke::new(4_f32, token(ui, Token::Border)),
    );
    ui.painter().line_segment(
        [point(values[0]), point(values[1])],
        Stroke::new(4_f32, token(ui, Token::Accent)),
    );
    let id = egui::Id::new((&node.id, "rangeSlider"));
    let response = ui.interact(rect, id, egui::Sense::click_and_drag());
    if resolved.enabled
        && (response.clicked() || response.dragged())
        && let Some(pointer) = response.interact_pointer_pos()
    {
        let active = if response.drag_started() || response.clicked() {
            let index = usize::from(
                pointer.distance(point(values[0])) > pointer.distance(point(values[1])),
            );
            ui.data_mut(|data| data.insert_temp(id.with("handle"), index));
            index
        } else {
            ui.data(|data| data.get_temp::<usize>(id.with("handle")).unwrap_or(0))
        };
        let mut fraction = if node.vertical {
            (rect.bottom() - 7. - pointer.y) / (rect.height() - 14.).max(1.)
        } else {
            (pointer.x - rect.left() - 7.) / (rect.width() - 14.).max(1.)
        };
        if reverse {
            fraction = 1. - fraction;
        }
        values[active] =
            node.normalize_number(min + f64::from(fraction.clamp(0., 1.)) * (max - min));
        response.request_focus();
        if active == 0 {
            values[0] = values[0].min(values[1]);
        } else {
            values[1] = values[1].max(values[0]);
        }
    }
    if resolved.enabled && response.has_focus() {
        let active = ui.data(|data| data.get_temp::<usize>(id.with("handle")).unwrap_or(0));
        let delta = ui.input(|i| {
            i32::from(i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp))
                - i32::from(
                    i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown),
                )
        });
        let delta = if reverse { -delta } else { delta };
        if delta != 0 {
            values[active] =
                node.normalize_number(values[active] + f64::from(delta) * node.step.unwrap_or(1.));
        }
        if active == 0 {
            values[0] = values[0].min(values[1]);
        } else {
            values[1] = values[1].max(values[0]);
        }
    }
    for v in values {
        ui.painter()
            .circle_filled(point(v), 7., token(ui, Token::Accent));
    }
    if values != before {
        *value = json!(values);
    }
}
fn slider(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
) {
    let enabled = resolved.enabled;
    let reverse = resolved.api.get("reverse") == Some(&json!(true));
    let response = ui.interact(
        rect,
        egui::Id::new((&node.id, "slider")),
        egui::Sense::click_and_drag(),
    );
    let (min, max) = node.number_range();
    let mut number = value.as_f64().unwrap();
    let radius = 7_f32
        .min(if node.vertical {
            rect.height() / 2.
        } else {
            rect.width() / 2.
        })
        .max(0.);
    let start = if node.vertical {
        rect.bottom() - radius
    } else {
        rect.left() + radius
    };
    let end = if node.vertical {
        rect.top() + radius
    } else {
        rect.right() - radius
    };
    let (start, end) = if reverse { (end, start) } else { (start, end) };
    if enabled
        && (response.clicked() || response.dragged())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let point = if node.vertical { pos.y } else { pos.x };
        if (end - start).abs() > f32::EPSILON {
            number = node.normalize_number(
                min + f64::from(((point - start) / (end - start)).clamp(0., 1.)) * (max - min),
            );
        }
        response.request_focus();
    }
    if enabled && response.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            )
        });
        ui.input(|i| {
            let delta = i32::from(
                i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp),
            ) - i32::from(
                i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown),
            );
            if delta != 0 {
                let delta = if reverse { -delta } else { delta };
                number = node.normalize_number(number + f64::from(delta) * node.step.unwrap_or(1.));
            }
            if i.key_pressed(egui::Key::Home) {
                number = min;
            }
            if i.key_pressed(egui::Key::End) {
                number = max;
            }
        });
    }
    if value.as_f64() != Some(number) {
        *value = json!(number);
    }
    response.widget_info(|| egui::WidgetInfo::slider(enabled, number, ""));
    let t = ((number - min) / (max - min)) as f32;
    let point = start + t * (end - start);
    let (a, b, center) = if node.vertical {
        (
            Pos2::new(rect.center().x, start),
            Pos2::new(rect.center().x, end),
            Pos2::new(rect.center().x, point),
        )
    } else {
        (
            Pos2::new(start, rect.center().y),
            Pos2::new(end, rect.center().y),
            Pos2::new(point, rect.center().y),
        )
    };
    ui.painter()
        .line_segment([a, b], Stroke::new(4_f32, token(ui, Token::Border)));
    ui.painter()
        .line_segment([a, center], Stroke::new(4_f32, token(ui, Token::Accent)));
    ui.painter().circle(
        center,
        radius,
        ui.visuals().widgets.inactive.bg_fill,
        Stroke::new(
            if response.has_focus() { 2_f32 } else { 1_f32 },
            token(
                ui,
                if response.has_focus() {
                    Token::Focus
                } else {
                    Token::Accent
                },
            ),
        ),
    );
    response.on_hover_text(number.to_string());
}
