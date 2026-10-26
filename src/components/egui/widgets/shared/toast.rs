use super::*;
pub(in crate::components::egui::widgets) const PROPERTIES: &[(&str, PropertyType)] = &[
    ("open", Bool),
    ("duration", Number(0., 3600.)),
    ("description", Text),
    (
        "status",
        Enum(&["success", "info", "warning", "error", "loading"]),
    ),
    ("closable", Bool),
    (
        "placement",
        Enum(&[
            "topLeft",
            "topRight",
            "bottomLeft",
            "bottomRight",
            "top",
            "bottom",
        ]),
    ),
];

pub(in crate::components::egui::widgets) fn paint(
    c: WidgetContext<'_, '_>,
    placement: &str,
    closable: bool,
) {
    let WidgetContext {
        ui,
        n,
        r,
        state,
        memory,
        collector,
        ..
    } = c;
    memory.body = None;
    if !local(n, state, memory, "open", json!(true))
        .as_bool()
        .unwrap_or(true)
    {
        memory.started = None;
        memory.dismissed = false;
        return;
    }
    let now = ui.input(|i| i.time);
    let start = *memory.started.get_or_insert(now);
    let duration = contract::number(n, state, "duration", 3.);
    if memory.dismissed {
        return;
    }
    let mut close = duration > 0. && now - start >= duration;
    if !close {
        // Keep bottom notifications above the application's status/footer area.
        let placement = contract::string(n, state, "placement", placement);
        let (anchor, offset) = match placement {
            "topLeft" => (egui::Align2::LEFT_TOP, Vec2::new(16., 16.)),
            "bottomLeft" => (egui::Align2::LEFT_BOTTOM, Vec2::new(16., -88.)),
            "bottomRight" => (egui::Align2::RIGHT_BOTTOM, Vec2::new(-16., -88.)),
            "bottom" => (egui::Align2::CENTER_BOTTOM, Vec2::new(0., -88.)),
            "top" => (egui::Align2::CENTER_TOP, Vec2::new(0., 16.)),
            _ => (egui::Align2::RIGHT_TOP, Vec2::new(-16., 16.)),
        };
        let style = ui.style().clone();
        let available = ui
            .ctx()
            .style()
            .spacing
            .default_area_size
            .min(ui.ctx().screen_rect().size());
        let mut measure = floating_measure_ui(ui, n, available);
        content(&mut measure, n, r, state, closable);
        let rect = anchor
            .align_size_within_rect(measure.min_size(), ui.ctx().screen_rect())
            .translate(offset);
        let bottom = matches!(placement, "bottom" | "bottomLeft" | "bottomRight");
        let rect = if bottom {
            avoid_bottom_obstacles(rect, &memory.floating_reserved)
        } else {
            rect
        };
        let area = egui::Area::new(egui::Id::new((&n.id, "toast")))
            .order(egui::Order::Foreground)
            .default_size(rect.size())
            .enabled(r.enabled);
        let area = if bottom {
            // Clamping back down to the viewport could undo obstacle clearance.
            area.fixed_pos(rect.min).constrain(false)
        } else {
            area.anchor(anchor, offset)
        };
        let shown = area.show(ui.ctx(), |ui| {
            ui.set_style(style);
            ui.set_max_width(available.x);
            close |= content(ui, n, r, state, closable);
        });
        memory.body = Some(shown.response.rect);
        if duration > 0. {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    if close {
        memory.dismissed = true;
        emit(ui, n, memory, collector, "openChange", json!(false));
        collector.push(ui, "close", None, Value::Null);
    }
}

fn content(
    ui: &mut egui::Ui,
    n: &protocol::Node,
    r: &Resolved,
    state: &crate::binding::State,
    closable: bool,
) -> bool {
    let mut close = false;
    egui::Frame::popup(ui.style())
        .inner_margin(12.)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Sizing passes otherwise shrink the row to its text height,
                // while normal horizontal layouts reserve interact_size.y.
                // Keep obstacle measurement and the visible card identical.
                ui.set_min_height(ui.spacing().interact_size.y);
                ui.colored_label(
                    token(
                        ui,
                        shared::status_color(contract::string(n, state, "status", "info")),
                    ),
                    if r.title.is_empty() {
                        &r.text
                    } else {
                        &r.title
                    },
                );
                if contract::flag(n, state, "closable", closable) && ui.small_button("×").clicked()
                {
                    close = true;
                }
            });
            if !r.title.is_empty() && !r.text.is_empty() {
                ui.label(&r.text);
            }
            let description = contract::string(n, state, "description", "");
            if !description.is_empty() {
                ui.label(description);
            }
        });
    close
}

fn avoid_bottom_obstacles(mut rect: egui::Rect, reserved: &[egui::Rect]) -> egui::Rect {
    const GAP: f32 = 8.;
    // Each move is strictly upward. Repeat to catch obstacles reached by a move,
    // independent of their order in the component tree.
    loop {
        let bottom = reserved
            .iter()
            .filter(|obstacle| {
                rect.left() < obstacle.right()
                    && obstacle.left() < rect.right()
                    && rect.top() < obstacle.bottom() + GAP
                    && obstacle.top() - GAP < rect.bottom()
            })
            .map(|obstacle| obstacle.top() - GAP)
            .fold(rect.bottom(), f32::min);
        if bottom >= rect.bottom() {
            return rect;
        }
        rect = egui::Rect::from_min_max(
            Pos2::new(rect.left(), bottom - rect.height()),
            Pos2::new(rect.right(), bottom),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::avoid_bottom_obstacles;
    use eframe::egui::{Rect, pos2, vec2};

    #[test]
    fn bottom_toast_clears_example_button_and_description() {
        let toast = Rect::from_min_size(pos2(684., 632.), vec2(300., 80.));
        // 1000x800 screen, offsetX=24, offsetY=65; content extends below 48px.
        let button = Rect::from_min_size(pos2(912., 671.), vec2(64., 72.));
        let moved = avoid_bottom_obstacles(toast, &[button]);
        assert_eq!(moved.bottom(), button.top() - 8.);
        assert_eq!(moved.size(), toast.size());
        assert_eq!(moved.left(), toast.left());
    }

    #[test]
    fn empty_or_unrelated_obstacles_keep_footer_spacing() {
        let toast = Rect::from_min_size(pos2(684., 632.), vec2(300., 80.));
        let left = Rect::from_min_size(pos2(16., 671.), vec2(48., 48.));
        let below = Rect::from_min_size(pos2(912., 750.), vec2(48., 48.));
        assert_eq!(avoid_bottom_obstacles(toast, &[]), toast);
        assert_eq!(avoid_bottom_obstacles(toast, &[left, below]), toast);
    }

    #[test]
    fn chained_obstacles_are_order_independent() {
        let toast = Rect::from_min_size(pos2(684., 632.), vec2(300., 80.));
        let lower = Rect::from_min_size(pos2(912., 671.), vec2(48., 48.));
        let upper = Rect::from_min_size(pos2(912., 570.), vec2(48., 48.));
        let moved = avoid_bottom_obstacles(toast, &[upper, lower]);
        assert_eq!(moved.bottom(), upper.top() - 8.);
        assert_eq!(moved, avoid_bottom_obstacles(toast, &[lower, upper]));
        assert_eq!(avoid_bottom_obstacles(moved, &[upper, lower]), moved);
    }

    #[test]
    fn updated_geometry_replaces_old_geometry() {
        let toast = Rect::from_min_size(pos2(684., 632.), vec2(300., 80.));
        let button = Rect::from_min_size(pos2(912., 671.), vec2(48., 48.));
        assert!(avoid_bottom_obstacles(toast, &[button]).bottom() < toast.bottom());
        let shifted = button.translate(vec2(-500., 0.));
        assert_eq!(avoid_bottom_obstacles(toast, &[shifted]), toast);
        assert_eq!(avoid_bottom_obstacles(toast, &[]), toast);
    }
}
