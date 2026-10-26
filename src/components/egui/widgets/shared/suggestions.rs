use super::*;
pub(in crate::components::egui::widgets) fn paint(c: WidgetContext<'_, '_>, mentions: bool) {
    let WidgetContext {
        ui,
        n,
        r,
        rect,
        state,
        memory,
        value,
        collector,
        ..
    } = c;
    let options = options(n, state);
    let mut text = value.as_str().unwrap_or("").to_owned();
    let old = text.clone();
    let clear = contract::flag(n, state, "allowClear", false) && !text.is_empty();
    let edit = if mentions {
        egui::TextEdit::multiline(&mut text)
            .desired_rows(contract::number(n, state, "rows", 3.) as usize)
    } else {
        egui::TextEdit::singleline(&mut text).vertical_align(egui::Align::Center)
    };
    let input_rect = egui::Rect::from_min_max(
        rect.min,
        Pos2::new(
            (rect.right() - if clear { 64. } else { 0. }).max(rect.left() + 1.),
            rect.bottom(),
        ),
    );
    let response = ui.put(
        input_rect,
        edit.font(style_font(&n.style, ui.ctx()))
            .margin(egui::Margin::symmetric(
                n.style.padding_x.unwrap_or(8.).clamp(0., 127.) as i8,
                n.style.padding_y.unwrap_or(6.).clamp(0., 127.) as i8,
            ))
            .hint_text(&r.placeholder)
            .char_limit(crate::MAX_INPUT_CHARS),
    );
    if response.changed() {
        collector.push(ui, "search", None, json!(text));
    }
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        collector.push(ui, "submit", None, json!(text));
    }
    let prefix = contract::string(n, state, "prefix", "@");
    let query = if mentions {
        text.rsplit_once(prefix).map(|(_, q)| q).unwrap_or("")
    } else {
        &text
    };
    if response.has_focus() || memory.was_open {
        let query = query.to_lowercase();
        let style = ui.style().clone();
        let popup = egui::Area::new(egui::Id::new((&n.id, "suggestions")))
            .order(egui::Order::Foreground)
            .fixed_pos(input_rect.left_bottom())
            .show(ui.ctx(), |ui| {
                ui.set_style(style);
                let frame = egui::Frame::popup(ui.style());
                let width = (input_rect.width() - frame.total_margin().sum().x).max(1.);
                frame.show(ui, |ui| {
                    ui.set_width(width);
                    for option in options.iter().take(100) {
                        let (key, label, disabled) = option_parts(option);
                        if contract::flag(n, state, "filterOption", true)
                            && !label.to_lowercase().contains(&query)
                        {
                            continue;
                        }
                        if ui
                            .add_enabled(!disabled, egui::Button::new(label).frame(false))
                            .clicked()
                        {
                            if mentions {
                                if let Some((head, _)) = text.rsplit_once(prefix) {
                                    text = format!(
                                        "{head}{prefix}{key}{}",
                                        contract::string(n, state, "split", " ")
                                    );
                                } else {
                                    text.push_str(&format!("{prefix}{key} "));
                                }
                            } else {
                                text = key.into();
                            }
                            collector.push(ui, "select", None, json!(key));
                            memory.was_open = false;
                        }
                    }
                });
            });
        if ui.input(|i| i.pointer.any_pressed())
            && !popup.response.contains_pointer()
            && !response.contains_pointer()
        {
            memory.was_open = false;
        } else if response.has_focus() {
            memory.was_open = true;
        }
    }
    if clear
        && ui
            .put(
                egui::Rect::from_min_size(
                    rect.right_top() - Vec2::new(60., 0.),
                    Vec2::new(60., rect.height().min(40.)),
                ),
                egui::Button::new("清除"),
            )
            .clicked()
    {
        text.clear();
        collector.push(ui, "clear", None, Value::Null);
    }
    if text != old {
        *value = json!(text);
    }
}
