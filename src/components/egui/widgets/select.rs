use super::*;
component!(
    &[
        ("mode", Enum(&["single", "multiple", "tags"])),
        ("showSearch", Bool),
        ("allowClear", Bool),
        ("loading", Bool),
        ("maxCount", Integer(1, 10000)),
        ("notFoundContent", Text)
    ],
    &["change", "open", "search", "clear"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        memory,
        value,
        collector,
    } = c;
    control_theme(ui, &n.style, rect.height());
    let options = v02::select_options(n, state).unwrap_or_default();
    let multiple = contract::multiple(n, state);
    let mut selected = if multiple {
        strings(value)
    } else {
        vec![value.as_str().unwrap_or("").into()]
    };
    let label = selected
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| {
            options
                .iter()
                .find(|o| &o.value == s)
                .map(|o| o.label.clone())
                .unwrap_or_else(|| s.clone())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let clear = contract::flag(n, state, "allowClear", false) && !label.is_empty();
    let clear_width = if clear { 48_f32.min(rect.width()) } else { 0. };
    let button_rect = egui::Rect::from_min_max(rect.min, rect.max - Vec2::new(clear_width, 0.));
    let response = ui.put(
        button_rect,
        egui::Button::new(if label.is_empty() {
            &r.placeholder
        } else {
            &label
        }),
    );
    let id = egui::Id::new((&n.id, "selectPopup"));
    if response.clicked() {
        ui.memory_mut(|m| m.toggle_popup(id));
    }
    let shown = egui::popup::popup_above_or_below_widget(
        ui,
        id,
        &response,
        egui::AboveOrBelow::Below,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_min_width(rect.width());
            if contract::flag(n, state, "showSearch", false)
                && ui.text_edit_singleline(&mut memory.search).changed()
            {
                collector.push(ui, "search", None, json!(memory.search));
            }
            if contract::flag(n, state, "loading", false) {
                ui.spinner();
                return;
            }
            let mut count = 0;
            egui::ScrollArea::vertical()
                .max_height(260.)
                .show(ui, |ui| {
                    for option in options.iter().filter(|o| {
                        o.label
                            .to_lowercase()
                            .contains(&memory.search.to_lowercase())
                    }) {
                        count += 1;
                        let selected_now = selected.contains(&option.value);
                        if ui
                            .add_enabled(
                                !option.disabled,
                                egui::SelectableLabel::new(selected_now, &option.label),
                            )
                            .clicked()
                        {
                            if multiple {
                                if selected_now
                                    || selected.len()
                                        < contract::number(n, state, "maxCount", 10000.) as usize
                                {
                                    toggle(&mut selected, &option.value);
                                }
                            } else {
                                selected = vec![option.value.clone()];
                                ui.memory_mut(|m| m.close_popup());
                            }
                        }
                    }
                    if count == 0 {
                        ui.weak(contract::string(n, state, "notFoundContent", "暂无选项"));
                    }
                    if contract::string(n, state, "mode", "single") == "tags"
                        && !memory.search.trim().is_empty()
                        && ui.button(format!("添加 {}", memory.search)).clicked()
                    {
                        let tag = memory.search.trim().to_owned();
                        if !selected.contains(&tag)
                            && selected.len()
                                < contract::number(n, state, "maxCount", 10000.) as usize
                        {
                            selected.push(tag);
                        }
                        memory.search.clear();
                    }
                });
        },
    )
    .is_some();
    if shown && !memory.was_open {
        collector.push(ui, "open", None, Value::Null);
    }
    memory.was_open = shown;
    if clear
        && ui
            .put(
                egui::Rect::from_min_max(Pos2::new(button_rect.right(), rect.top()), rect.max),
                egui::Button::new("清除"),
            )
            .clicked()
    {
        selected.clear();
        collector.push(ui, "clear", None, Value::Null);
    }
    *value = if multiple {
        json!(selected)
    } else {
        json!(selected.first().cloned().unwrap_or_default())
    };
}

pub(in crate::components::egui) fn paint_simple(
    ui: &mut egui::Ui,
    n: &v02::Node,
    r: &Resolved,
    content: egui::Rect,
    state: &crate::binding::State,
    selected: &mut String,
    collector: &mut Collector<'_>,
) -> bool {
    control_theme(ui, &n.style, content.height());
    let options = v02::select_options(n, state).unwrap_or_default();
    let before = selected.clone();
    let display = options
        .iter()
        .find(|o| o.value == *selected)
        .map(|o| o.label.as_str())
        .unwrap_or(if selected.is_empty() {
            &r.placeholder
        } else {
            selected
        })
        .to_owned();
    let mut open = false;
    ui.add_enabled_ui(r.enabled, |ui| {
        ui.push_id((&n.id, "select"), |ui| {
            let row_height = (ui.text_style_height(&egui::TextStyle::Button)
                + 2. * ui.spacing().button_padding.y)
                .max(ui.spacing().interact_size.y);
            let row_spacing = ui.spacing().item_spacing.y;
            let response = ui.put(content, egui::Button::new(""));
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, r.enabled, &display)
            });
            let visuals = ui.style().interact(&response);
            let text_rect = egui::Rect::from_min_max(
                content.min + Vec2::new(ui.spacing().button_padding.x, 0.),
                content.max - Vec2::new(24., 0.),
            );
            let galley = ui.painter().layout_no_wrap(
                display.clone(),
                style_font(&n.style, ui.ctx()),
                visuals.text_color(),
            );
            ui.painter()
                .with_clip_rect(text_rect.intersect(ui.clip_rect()))
                .galley(
                    Pos2::new(
                        text_rect.left(),
                        text_rect.center().y - galley.size().y / 2.,
                    ),
                    galley,
                    visuals.text_color(),
                );
            let center = Pos2::new(content.right() - 13., content.center().y);
            ui.painter().add(egui::Shape::convex_polygon(
                vec![
                    center + Vec2::new(-4., -2.),
                    center + Vec2::new(4., -2.),
                    center + Vec2::new(0., 3.),
                ],
                visuals.text_color(),
                Stroke::NONE,
            ));
            let popup_id = ui.make_persistent_id("popup");
            if response.clicked() {
                ui.memory_mut(|m| m.toggle_popup(popup_id));
            }
            let screen = ui.ctx().screen_rect();
            let below = (screen.bottom() - response.rect.bottom() - 12.).max(0.);
            let above = (response.rect.top() - screen.top() - 12.).max(0.);
            let count = options.len().clamp(1, 10) as f32;
            let desired = count * row_height + (count - 1.) * row_spacing;
            let (direction, available) = if below < desired && above > below {
                (egui::AboveOrBelow::Above, above)
            } else {
                (egui::AboveOrBelow::Below, below)
            };
            let height = desired.min(available);
            let popup_style = ui.style().clone();
            open = egui::popup::popup_above_or_below_widget(
                ui,
                popup_id,
                &response,
                direction,
                egui::PopupCloseBehavior::CloseOnClickOutside,
                |ui| {
                    ui.set_style(popup_style);
                    ui.spacing_mut().interact_size.y = row_height;
                    ui.spacing_mut().item_spacing.y = row_spacing;
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    // Reset both bounds: a prior loading row must not constrain the loaded options.
                    egui::ScrollArea::vertical()
                        .min_scrolled_height(height)
                        .max_height(height)
                        .show(ui, |ui| {
                            for option in &options {
                                ui.add_enabled_ui(!option.disabled, |ui| {
                                    if ui
                                        .selectable_value(
                                            selected,
                                            option.value.clone(),
                                            &option.label,
                                        )
                                        .clicked()
                                    {
                                        ui.memory_mut(|m| m.close_popup());
                                    }
                                });
                            }
                        });
                },
            )
            .is_some();
        });
    });
    if *selected != before {
        collector.push(ui, "change", None, json!(selected));
    }
    open
}
