use super::*;
use crate::layout::splitter;
component!(&[], &["change"], LayoutKind::Splitter);
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    node: &protocol::Node,
    content: egui::Rect,
    first_rect: egui::Rect,
    panels: [&protocol::Layout; 2],
    enabled: bool,
) -> Option<f32> {
    let horizontal = !node.vertical;
    let total = if horizontal {
        content.width()
    } else {
        content.height()
    };
    let bar = splitter::bar_size(node, total);
    let available = (total - bar).max(0.);
    let handle = if horizontal {
        egui::Rect::from_min_size(
            Pos2::new(first_rect.right(), content.top()),
            Vec2::new(bar, content.height()),
        )
    } else {
        egui::Rect::from_min_size(
            Pos2::new(content.left(), first_rect.bottom()),
            Vec2::new(content.width(), bar),
        )
    };
    let enabled = enabled && node.resizable != Some(false) && available > 0.;
    let response = ui.interact(
        handle.intersect(painter.clip_rect()),
        egui::Id::new((&node.id, "splitter")),
        if enabled {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        },
    );
    if enabled && (response.hovered() || response.dragged()) {
        ui.ctx().set_cursor_icon(if horizontal {
            egui::CursorIcon::ResizeHorizontal
        } else {
            egui::CursorIcon::ResizeVertical
        });
    }
    if response.clicked() {
        response.request_focus();
    }
    painter.rect_filled(
        handle,
        0.,
        token(
            ui,
            if enabled && (response.hovered() || response.dragged() || response.has_focus()) {
                Token::Accent
            } else {
                Token::Border
            },
        )
        .gamma_multiply(0.5),
    );
    if !enabled {
        return None;
    }
    let mut delta = if horizontal {
        response.drag_delta().x
    } else {
        response.drag_delta().y
    };
    if response.has_focus() {
        ui.input(|input| {
            let step = if input.modifiers.shift { 1. } else { 10. };
            if input.key_pressed(if horizontal {
                egui::Key::ArrowRight
            } else {
                egui::Key::ArrowDown
            }) {
                delta += step;
            }
            if input.key_pressed(if horizontal {
                egui::Key::ArrowLeft
            } else {
                egui::Key::ArrowUp
            }) {
                delta -= step;
            }
        });
    }
    if delta == 0. && !response.double_clicked() {
        return None;
    }
    let current = if horizontal {
        first_rect.width()
    } else {
        first_rect.height()
    };
    let desired = if response.double_clicked() {
        0.5
    } else {
        ((current + delta) / available).clamp(0., 1.)
    };
    let sizes = splitter::sizes(
        available,
        desired,
        splitter::limits(panels[0], horizontal),
        splitter::limits(panels[1], horizontal),
    );
    let ratio = (sizes[0] / available).clamp(0., 1.);
    ((sizes[0] - current).abs() > 0.01).then_some(ratio)
}
