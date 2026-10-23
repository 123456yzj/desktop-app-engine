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
    let edit = if mentions {
        egui::TextEdit::multiline(&mut text)
            .desired_rows(contract::number(n, state, "rows", 3.) as usize)
    } else {
        egui::TextEdit::singleline(&mut text)
    };
    let response = ui.add_sized(
        [
            rect.width(),
            if mentions { rect.height() * 0.6 } else { 30. },
        ],
        edit.hint_text(&r.placeholder)
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
        let popup = egui::Area::new(egui::Id::new((&n.id, "suggestions")))
            .order(egui::Order::Foreground)
            .fixed_pos(response.rect.left_bottom())
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(response.rect.width());
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
    if contract::flag(n, state, "allowClear", false)
        && !text.is_empty()
        && ui.small_button("清除").clicked()
    {
        text.clear();
        collector.push(ui, "clear", None, Value::Null);
    }
    if text != old {
        *value = json!(text);
    }
}
