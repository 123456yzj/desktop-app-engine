use super::*;
component!(&[("optionType", Enum(&["default", "button"]))], &["change"]);
pub(in crate::components::egui) fn control(
    ui: &mut egui::Ui,
    node: &v02::Node,
    resolved: &Resolved,
    rect: egui::Rect,
    value: &mut Value,
) {
    let mut x = rect.left();
    let mut y = rect.top();
    let mut focus = None;
    for (index, option) in resolved.options.iter().enumerate() {
        let galley = ui.painter().layout(
            option.label.clone(),
            style_font(&node.style, ui.ctx()),
            ui.visuals().text_color(),
            if node.style.text_wrap == Some(false) {
                100_000.
            } else {
                (rect.width() - 30.).max(1.)
            },
        );
        let h = galley.size().y.max(28.);
        let w = if node.vertical {
            rect.width()
        } else {
            galley.size().x + 30.
        };
        let option_rect = egui::Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
        let response = ui
            .add_enabled_ui(!option.disabled, |ui| {
                ui.interact(
                    option_rect,
                    egui::Id::new((&node.id, "radio", index)),
                    egui::Sense::click(),
                )
            })
            .inner;
        let selected = value.as_str() == Some(&option.value);
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::RadioButton,
                resolved.enabled && !option.disabled,
                selected,
                &option.label,
            )
        });
        let visuals = ui.style().interact(&response);
        let center = Pos2::new(x + 10., y + h / 2.);
        let color = if option.disabled || !resolved.enabled {
            token(ui, Token::TextMuted)
        } else if selected {
            token(ui, Token::Accent)
        } else {
            visuals.fg_stroke.color
        };
        let button = resolved.api.get("optionType").and_then(Value::as_str) == Some("button");
        if button {
            ui.painter().rect(
                option_rect,
                4.,
                if selected {
                    color.gamma_multiply(0.15)
                } else {
                    visuals.bg_fill
                },
                Stroke::new(1_f32, color),
                StrokeKind::Inside,
            );
        } else {
            ui.painter().circle(
                center,
                8.,
                visuals.bg_fill,
                Stroke::new(
                    1_f32,
                    if response.has_focus() {
                        token(ui, Token::Focus)
                    } else {
                        color
                    },
                ),
            );
        }
        if selected && !button {
            ui.painter().circle_filled(center, 4., color);
        }
        ui.painter().galley_with_override_text_color(
            Pos2::new(
                x + if button { 15. } else { 30. },
                y + (h - galley.size().y) / 2.,
            ),
            galley,
            color,
        );
        if response.clicked() {
            *value = json!(option.value);
            response.request_focus();
        }
        if response.has_focus() && resolved.enabled && !option.disabled {
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
            let direction = ui.input(|i| {
                if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowDown) {
                    1
                } else if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowUp) {
                    -1
                } else {
                    0
                }
            });
            if direction != 0 {
                let count = resolved.options.len();
                for offset in 1..=count {
                    let next = (index as isize + direction * offset as isize)
                        .rem_euclid(count as isize) as usize;
                    if !resolved.options[next].disabled {
                        *value = json!(resolved.options[next].value);
                        focus = Some(egui::Id::new((&node.id, "radio", next)));
                        break;
                    }
                }
            }
        }
        if node.vertical {
            y += h + 8.;
        } else {
            x += w + 8.;
        }
    }
    if let Some(focus) = focus {
        ui.memory_mut(|m| m.request_focus(focus));
    }
}
