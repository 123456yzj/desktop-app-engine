use super::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::components::egui::widgets) fn paint(
    ui: &mut egui::Ui,
    n: &protocol::Node,
    state: &crate::binding::State,
    items: &[Value],
    selected: &mut Vec<String>,
    open: &mut Vec<String>,
    collector: &mut Collector<'_>,
    depth: usize,
) {
    if depth > 32 {
        return;
    }
    for item in items {
        if item["type"] == "divider" {
            ui.separator();
            continue;
        }
        let key = item_key(item);
        let children = item["children"].as_array();
        if item["type"] == "group" {
            ui.weak(item_label(item));
            if let Some(children) = children {
                paint(ui, n, state, children, selected, open, collector, depth + 1);
            }
            continue;
        }
        let has_children = children.is_some_and(|c| !c.is_empty());
        let label = if contract::flag(n, state, "inlineCollapsed", false) {
            item["icon"].as_str().unwrap_or("•").to_owned()
        } else {
            format!(
                "{}{}{}",
                item["icon"]
                    .as_str()
                    .map(|s| format!("{s} "))
                    .unwrap_or_default(),
                item_label(item),
                if has_children { " ▸" } else { "" }
            )
        };
        let active = selected.iter().any(|s| s == key);
        let horizontal = contract::string(n, state, "mode", "vertical") == "horizontal";
        let width = if horizontal { 0. } else { ui.available_width() };
        let response = ui
            .scope(|ui| {
                if item["danger"] == true {
                    ui.visuals_mut().override_text_color = Some(token(ui, Token::Danger));
                }
                shared::navigation_item(ui, &label, active, !item_disabled(item), width, 36.)
            })
            .inner;
        if active && !horizontal {
            let mark = egui::Rect::from_min_size(
                response.rect.left_center() - Vec2::new(0., 8.),
                Vec2::new(3., 16.),
            );
            ui.painter()
                .rect_filled(mark, 2., token(ui, Token::Primary));
        }
        if response.clicked() {
            if has_children {
                toggle(open, key);
            } else {
                collector.push(ui, "click", None, json!(key));
                if contract::flag(n, state, "selectable", true) {
                    let deselect = contract::flag(n, state, "multiple", false)
                        && selected.iter().any(|s| s == key);
                    if contract::flag(n, state, "multiple", false) {
                        toggle(selected, key);
                    } else {
                        *selected = vec![key.into()];
                    }
                    if n.kind == Kind::Menu {
                        collector.push(
                            ui,
                            if deselect { "deselect" } else { "select" },
                            None,
                            json!({"key":key,"selectedKeys":selected}),
                        );
                    }
                }
            }
        }
        if contract::flag(n, state, "inlineCollapsed", false) {
            response.on_hover_text(item_label(item));
        }
        if let Some(children) = children.filter(|_| open.iter().any(|k| k == key)) {
            ui.scope(|ui| {
                ui.spacing_mut().indent = contract::number(n, state, "inlineIndent", 24.) as f32;
                ui.indent(key, |ui| {
                    paint(ui, n, state, children, selected, open, collector, depth + 1)
                });
            });
        }
    }
}
