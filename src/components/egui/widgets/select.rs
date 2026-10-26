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
    let options = protocol::select_options(n, state).unwrap_or_default();
    let multiple = contract::multiple(n, state);
    let tags = contract::string(n, state, "mode", "single") == "tags";
    let searchable = contract::flag(n, state, "showSearch", multiple);
    let loading = contract::flag(n, state, "loading", false);
    let max = contract::number(n, state, "maxCount", 10000.) as usize;
    let mut selected = if multiple {
        strings(value)
    } else {
        value
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| vec![s.to_owned()])
            .unwrap_or_default()
    };
    let id = egui::Id::new((&n.id, "selectPopup"));
    let trigger_id = id.with("trigger");
    let search_id = id.with("search");
    let was_open = ui.memory(|m| m.is_popup_open(id));
    let had_focus = ui.memory(|m| m.has_focus(trigger_id) || m.has_focus(search_id));
    // Consume selector navigation before TextEdit can treat Enter as blur or arrows as cursor movement.
    let (down, up, enter, space) = if r.enabled && had_focus {
        ui.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                !searchable && i.consume_key(egui::Modifiers::NONE, egui::Key::Space),
            )
        })
    } else {
        (false, false, false, false)
    };
    let exit =
        was_open && ui.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Tab));
    let hovered = r.enabled && ui.rect_contains_pointer(rect);
    let paint_state = if !r.enabled {
        n.style.disabled.as_ref()
    } else if hovered && ui.input(|i| i.pointer.primary_down()) {
        n.style.pressed.as_ref()
    } else if hovered {
        n.style.hover.as_ref()
    } else {
        None
    };
    let bg = paint_state
        .and_then(|s| s.background)
        .or(n.style.background)
        .map(|c| token(ui, c))
        .unwrap_or_else(|| {
            if r.enabled {
                if ui.visuals().dark_mode {
                    Color32::from_rgb(31, 31, 31)
                } else {
                    Color32::WHITE
                }
            } else {
                ui.visuals().faint_bg_color
            }
        });
    let fg = paint_state
        .and_then(|s| s.color)
        .or(n.style.color)
        .map(|c| token(ui, c))
        .unwrap_or_else(|| token(ui, Token::TextPrimary));
    let muted = token(ui, Token::TextMuted);
    let accent = Color32::from_rgb(22, 119, 255);
    let font = style_font(&n.style, ui.ctx());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, n.style.radius, bg);
    let suffix = egui::Rect::from_min_max(
        Pos2::new((rect.right() - 30.).max(rect.left()), rect.top()),
        rect.max,
    );
    let mut text_rect = egui::Rect::from_min_max(
        rect.min + Vec2::new(12., 0.),
        Pos2::new((suffix.left() - 4.).max(rect.left() + 12.), rect.bottom()),
    );
    let trigger = ui.interact(
        egui::Rect::from_min_max(rect.min, Pos2::new(suffix.left(), rect.bottom())),
        trigger_id,
        if searchable && was_open {
            egui::Sense::hover()
        } else {
            egui::Sense::click()
        },
    );
    trigger.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, r.enabled, &r.placeholder)
    });
    let suffix_response = ui.interact(suffix, id.with("suffix"), egui::Sense::click());
    let can_clear = r.enabled
        && !loading
        && contract::flag(n, state, "allowClear", false)
        && !selected.is_empty();
    let clear_visible = can_clear && hovered;
    let cleared = clear_visible && suffix_response.clicked();
    let mut toggled = false;
    if cleared {
        selected.clear();
        memory.search.clear();
        memory.select_active = None;
        collector.push(ui, "clear", None, Value::Null);
        // Clearing has its own hit area and must never toggle the dropdown.
        ui.memory_mut(|m| {
            m.request_focus(if was_open && searchable {
                search_id
            } else {
                trigger_id
            })
        });
    } else if (trigger.clicked() && !enter && !space) || suffix_response.clicked() {
        ui.memory_mut(|m| {
            m.toggle_popup(id);
            m.request_focus(trigger_id);
        });
        toggled = true;
    }
    if !r.enabled && was_open {
        ui.memory_mut(|m| m.close_popup());
    }
    if r.enabled
        && searchable
        && had_focus
        && !was_open
        && ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(s) if !s.is_empty()))
        })
    {
        ui.memory_mut(|m| m.open_popup(id));
    }
    if r.enabled
        && multiple
        && had_focus
        && memory.search.is_empty()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace))
        && let Some(index) = selected
            .iter()
            .rposition(|key| !options.iter().any(|o| &o.value == key && o.disabled))
    {
        selected.remove(index);
    }
    let mut open = r.enabled && ui.memory(|m| m.is_popup_open(id));
    let mut removed = None;
    if multiple {
        for (index, key) in selected.iter().enumerate() {
            let label = options
                .iter()
                .find(|o| &o.value == key)
                .map(|o| o.label.as_str())
                .unwrap_or(key);
            let galley = painter.layout_no_wrap(label.to_owned(), font.clone(), fg);
            let width = galley.size().x + 32.;
            if text_rect.width() < width + if searchable { 48. } else { 0. } {
                painter.text(
                    text_rect.left_center(),
                    egui::Align2::LEFT_CENTER,
                    format!("+{}", selected.len() - index),
                    font.clone(),
                    muted,
                );
                text_rect.min.x = (text_rect.left() + 36.).min(text_rect.right());
                break;
            }
            let chip = egui::Rect::from_center_size(
                Pos2::new(text_rect.left() + width / 2., rect.center().y),
                Vec2::new(width, (rect.height() - 8.).clamp(16., 24.)),
            );
            painter.rect_filled(
                chip,
                4.,
                if bg.r() < 128 {
                    Color32::from_rgb(48, 48, 48)
                } else {
                    Color32::from_rgb(245, 245, 245)
                },
            );
            painter.galley(
                Pos2::new(chip.left() + 6., chip.center().y - galley.size().y / 2.),
                galley,
                fg,
            );
            let close =
                egui::Rect::from_min_max(Pos2::new(chip.right() - 22., chip.top()), chip.max);
            let response = ui.interact(close, id.with(("remove", key)), egui::Sense::click());
            painter.text(
                close.center(),
                egui::Align2::CENTER_CENTER,
                "×",
                font.clone(),
                muted,
            );
            if response.clicked()
                && r.enabled
                && !options.iter().any(|o| &o.value == key && o.disabled)
            {
                removed = Some(key.clone());
                // A chip removal is not a selector click.
                if toggled {
                    ui.memory_mut(|m| {
                        if was_open {
                            m.open_popup(id)
                        } else {
                            m.close_popup()
                        }
                    });
                    open = was_open;
                }
            }
            text_rect.min.x += width + 4.;
        }
    }
    if let Some(key) = removed {
        selected.retain(|s| s != &key);
    }
    let selected_label = selected
        .first()
        .map(|key| {
            options
                .iter()
                .find(|o| &o.value == key)
                .map(|o| o.label.as_str())
                .unwrap_or(key.as_str())
        })
        .unwrap_or("");
    if searchable && open {
        let response = ui.put(
            text_rect,
            egui::TextEdit::singleline(&mut memory.search)
                .id(search_id)
                .frame(false)
                .margin(egui::Margin::symmetric(0, 0))
                .vertical_align(egui::Align::Center)
                .font(font.clone())
                .text_color(fg)
                .hint_text(if multiple && !selected.is_empty() {
                    ""
                } else if selected_label.is_empty() {
                    &r.placeholder
                } else {
                    selected_label
                }),
        );
        if !was_open || (cleared && open) || ui.memory(|m| m.has_focus(trigger_id)) {
            response.request_focus();
        }
        if response.changed() {
            memory.select_active = None;
            collector.push(ui, "search", None, json!(memory.search));
        }
    } else if !multiple || selected.is_empty() {
        let placeholder = selected_label.is_empty();
        let label = if placeholder {
            r.placeholder.as_str()
        } else {
            selected_label
        };
        let galley = painter.layout_no_wrap(
            label.to_owned(),
            font.clone(),
            if placeholder || !r.enabled { muted } else { fg },
        );
        painter
            .with_clip_rect(text_rect.intersect(ui.clip_rect()))
            .galley(
                Pos2::new(
                    text_rect.left(),
                    text_rect.center().y - galley.size().y / 2.,
                ),
                galley,
                fg,
            );
    }
    if loading {
        ui.put(
            suffix.shrink(5.),
            egui::Spinner::new().size(14.).color(accent),
        );
    } else if clear_visible {
        painter.text(
            suffix.center(),
            egui::Align2::CENTER_CENTER,
            "×",
            font.clone(),
            muted,
        );
    } else {
        let center = suffix.center();
        let sign = if open { -1. } else { 1. };
        painter.line_segment(
            [
                center + Vec2::new(-4., -2. * sign),
                center + Vec2::new(0., 2. * sign),
            ],
            Stroke::new(1.5_f32, muted),
        );
        painter.line_segment(
            [
                center + Vec2::new(0., 2. * sign),
                center + Vec2::new(4., -2. * sign),
            ],
            Stroke::new(1.5_f32, muted),
        );
    }
    let focused = ui.memory(|m| m.has_focus(trigger_id) || m.has_focus(search_id));
    ui.memory_mut(|m| {
        let filter = egui::EventFilter {
            vertical_arrows: true,
            escape: true,
            ..Default::default()
        };
        m.set_focus_lock_filter(trigger_id, filter);
        m.set_focus_lock_filter(search_id, filter);
    });
    let border = if r.enabled && (open || focused) {
        accent
    } else {
        paint_state
            .and_then(|s| s.border_color)
            .or(if hovered {
                Some(Token::Hex([64, 150, 255, 255]))
            } else {
                n.style.border_color
            })
            .map(|c| token(ui, c))
            .unwrap_or_else(|| token(ui, Token::Border))
    };
    // Draw inside the allocation: egui Button's hover expansion is clipped by the widget boundary.
    painter.rect_stroke(
        rect,
        n.style.radius,
        Stroke::new(n.style.border_width, border),
        StrokeKind::Inside,
    );
    let query = memory.search.to_lowercase();
    let mut rows: Vec<(String, String, bool)> = options
        .iter()
        .filter(|o| o.label.to_lowercase().contains(&query))
        .map(|o| {
            (
                o.value.clone(),
                o.label.clone(),
                o.disabled || (multiple && selected.len() >= max && !selected.contains(&o.value)),
            )
        })
        .collect();
    if tags
        && !memory.search.trim().is_empty()
        && !options.iter().any(|o| o.value == memory.search.trim())
        && !selected.iter().any(|s| s == memory.search.trim())
    {
        rows.insert(
            0,
            (
                memory.search.trim().to_owned(),
                memory.search.trim().to_owned(),
                selected.len() >= max,
            ),
        );
    }
    let candidates: Vec<_> = rows
        .iter()
        .filter(|(_, _, disabled)| !disabled && !loading)
        .map(|(key, _, _)| key.clone())
        .collect();
    let mut choice = None;
    if r.enabled && focused {
        if !open && (down || up || enter || space) {
            ui.memory_mut(|m| m.open_popup(id));
            open = true;
            memory.select_active = selected
                .iter()
                .find(|s| candidates.contains(s))
                .cloned()
                .or_else(|| {
                    if up {
                        candidates.last().cloned()
                    } else {
                        candidates.first().cloned()
                    }
                });
        } else if open {
            if down || up {
                let next = memory
                    .select_active
                    .as_ref()
                    .and_then(|key| candidates.iter().position(|k| k == key))
                    .map(|i| {
                        if down {
                            (i + 1) % candidates.len()
                        } else {
                            (i + candidates.len() - 1) % candidates.len()
                        }
                    })
                    .unwrap_or(if up {
                        candidates.len().saturating_sub(1)
                    } else {
                        0
                    });
                memory.select_active = candidates.get(next).cloned();
            }
            if enter || space {
                choice = memory
                    .select_active
                    .clone()
                    .or_else(|| candidates.first().cloned());
            }
        }
    }
    if exit {
        ui.memory_mut(|m| m.close_popup());
        open = false;
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.memory_mut(|m| m.request_focus(trigger_id));
        }
    }
    if open {
        if !candidates
            .iter()
            .any(|k| Some(k) == memory.select_active.as_ref())
        {
            memory.select_active = selected
                .iter()
                .find(|s| candidates.contains(s))
                .cloned()
                .or_else(|| candidates.first().cloned());
        }
        if !was_open {
            collector.push(ui, "open", None, Value::Null);
        }
        let available = ui.ctx().screen_rect();
        let below = available.bottom() - rect.bottom() - 12.;
        let above = rect.top() - available.top() - 12.;
        let desired = (rows.len().max(1) as f32 * 32.).min(256.);
        let (direction, height) = if below < desired && above > below {
            (egui::AboveOrBelow::Above, above.min(desired))
        } else {
            (egui::AboveOrBelow::Below, below.min(desired))
        };
        // Anchor the popup to the entire selector, including its suffix.
        let mut anchor = trigger.clone();
        anchor.rect = rect;
        anchor.interact_rect = rect.intersect(ui.clip_rect());
        let popup_width = (rect.width() - 16.).max(1.);
        // A fixed light selector may live in a dark ConfigProvider; keep popup contrast consistent.
        let mut popup_visuals = if bg.r() < 128 {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        popup_visuals.window_fill = bg;
        ui.style_mut().visuals = popup_visuals.clone();
        egui::popup::popup_above_or_below_widget(
            ui,
            id,
            &anchor,
            direction,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.style_mut().visuals = popup_visuals;
                ui.set_min_width(popup_width);
                ui.set_max_width(popup_width);
                ui.spacing_mut().item_spacing.y = 0.;
                if loading {
                    ui.spinner();
                    return;
                }
                egui::ScrollArea::vertical()
                    .min_scrolled_height(height.max(32.))
                    .max_height(height.max(32.))
                    .show(ui, |ui| {
                        if rows.is_empty() {
                            ui.weak(contract::string(n, state, "notFoundContent", "暂无选项"));
                        }
                        for (key, label, disabled) in &rows {
                            let checked = selected.contains(key);
                            let active = memory.select_active.as_ref() == Some(key);
                            let response = ui
                                .add_enabled_ui(!disabled, |ui| {
                                    ui.allocate_exact_size(
                                        Vec2::new(popup_width, 32.),
                                        egui::Sense::click(),
                                    )
                                    .1
                                })
                                .inner;
                            let fill = if checked {
                                if ui.visuals().dark_mode {
                                    Color32::from_rgb(17, 42, 69)
                                } else {
                                    Color32::from_rgb(230, 244, 255)
                                }
                            } else if response.hovered() || active {
                                ui.visuals().faint_bg_color
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(response.rect, 4., fill);
                            let text_color = if *disabled { muted } else { fg };
                            let text_clip = egui::Rect::from_min_max(
                                response.rect.min + Vec2::new(10., 0.),
                                response.rect.max - Vec2::new(28., 0.),
                            );
                            let galley = ui.painter().layout_no_wrap(
                                label.clone(),
                                font.clone(),
                                text_color,
                            );
                            ui.painter()
                                .with_clip_rect(text_clip.intersect(ui.clip_rect()))
                                .galley(
                                    Pos2::new(
                                        text_clip.left(),
                                        text_clip.center().y - galley.size().y / 2.,
                                    ),
                                    galley,
                                    text_color,
                                );
                            if checked {
                                ui.painter().text(
                                    response.rect.right_center() - Vec2::new(12., 0.),
                                    egui::Align2::CENTER_CENTER,
                                    "✓",
                                    font.clone(),
                                    accent,
                                );
                            }
                            if response.hovered() && ui.input(|i| i.pointer.delta() != Vec2::ZERO) {
                                memory.select_active = Some(key.clone());
                            }
                            if active
                                && focused
                                && ui.input(|i| {
                                    i.key_pressed(egui::Key::ArrowDown)
                                        || i.key_pressed(egui::Key::ArrowUp)
                                })
                            {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if response.clicked() {
                                choice = Some(key.clone());
                            }
                        }
                    });
                if searchable
                    && ui.rect_contains_pointer(ui.min_rect())
                    && ui.input(|i| i.pointer.any_pressed() || i.pointer.any_click())
                {
                    ui.memory_mut(|m| m.request_focus(search_id));
                }
            },
        );
    }
    if let Some(key) = choice.filter(|k| candidates.contains(k)) {
        if multiple {
            toggle(&mut selected, &key);
        } else {
            selected = vec![key];
            ui.memory_mut(|m| m.close_popup());
        }
        memory.search.clear();
        ui.memory_mut(|m| {
            m.request_focus(if multiple && searchable {
                search_id
            } else {
                trigger_id
            })
        });
    }
    memory.was_open = ui.memory(|m| m.is_popup_open(id));
    if !memory.was_open {
        memory.search.clear();
        memory.select_active = None;
    }
    *value = if multiple {
        json!(selected)
    } else {
        json!(selected.first().cloned().unwrap_or_default())
    };
}
