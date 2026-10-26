use super::*;
component!(
    &[
        ("showSearch", Bool),
        ("titles", Array),
        ("oneWay", Bool),
        ("showSelectAll", Bool)
    ],
    &["change", "search", "selectChange"]
);
pub(super) fn paint(c: WidgetContext<'_, '_>) {
    let WidgetContext {
        ui,
        n,
        rect,
        state,
        memory,
        value,
        collector,
        ..
    } = c;
    let options = options(n, state);
    let mut target = strings(value);
    let search = contract::flag(n, state, "showSearch", false);
    if search {
        if ui
            .add_sized(
                [rect.width(), 30.],
                egui::TextEdit::singleline(&mut memory.search).hint_text("搜索选项"),
            )
            .changed()
        {
            collector.push(ui, "search", None, json!(memory.search));
        }
    }
    let top = rect.top() + if search { 38. } else { 0. };
    let panel_width = ((rect.width() - 64.) / 2.).max(1.);
    let height = (rect.bottom() - top).max(1.);
    let mut cols = [0., panel_width + 12., panel_width + 64.].map(|x| {
        ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
            Pos2::new(rect.left() + x, top),
            Vec2::new(
                if x == panel_width + 12. {
                    40.
                } else {
                    panel_width
                },
                height,
            ),
        )))
    });
    for column in [0, 2] {
        let panel = cols[column].max_rect();
        cols[column].painter().rect_stroke(
            panel,
            4.,
            Stroke::new(1.0_f32, token(ui, Token::Border)),
            egui::StrokeKind::Inside,
        );
        cols[column] = ui.new_child(
            egui::UiBuilder::new()
                .id_salt(column)
                .max_rect(panel.shrink(10.)),
        );
    }
    {
        let titles = contract::property(n, state, "titles");
        cols[0].label(
            titles
                .and_then(|t| t.get(0))
                .and_then(Value::as_str)
                .unwrap_or("可选项"),
        );
        cols[2].label(
            titles
                .and_then(|t| t.get(1))
                .and_then(Value::as_str)
                .unwrap_or("已选项"),
        );
        for (column, right) in [(0, false), (2, true)] {
            egui::ScrollArea::vertical()
                .id_salt(column)
                .max_height((height - 48.).max(1.))
                .show(&mut cols[column], |ui| {
                    for option in options {
                        let (key, label, disabled) = option_parts(option);
                        if target.iter().any(|s| s == key) != right
                            || !label.to_lowercase().contains(&memory.search.to_lowercase())
                        {
                            continue;
                        }
                        let mut checked = memory.checks.contains(key);
                        if ui
                            .add_enabled(!disabled, egui::Checkbox::new(&mut checked, label))
                            .changed()
                        {
                            if checked {
                                memory.checks.insert(key.into());
                            } else {
                                memory.checks.remove(key);
                            }
                            collector.push(
                                ui,
                                "selectChange",
                                None,
                                json!(memory.checks.iter().collect::<Vec<_>>()),
                            );
                        }
                    }
                });
        }
        cols[1].add_space((height / 2. - 32.).max(0.));
        if cols[1]
            .add_sized([40., 30.], egui::Button::new("→"))
            .clicked()
        {
            for option in options {
                let (key, _, disabled) = option_parts(option);
                if !disabled && memory.checks.contains(key) && !target.iter().any(|s| s == key) {
                    target.push(key.into());
                }
            }
            memory.checks.clear();
        }
        if !contract::flag(n, state, "oneWay", false)
            && cols[1]
                .add_sized([40., 30.], egui::Button::new("←"))
                .clicked()
        {
            target.retain(|key| {
                !memory.checks.contains(key)
                    || options.iter().any(|o| {
                        let (k, _, disabled) = option_parts(o);
                        k == key && disabled
                    })
            });
            memory.checks.clear();
        }
    }
    *value = json!(target);
}
