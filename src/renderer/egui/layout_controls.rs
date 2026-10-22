use super::{style_font, token};
use crate::{
    layout::splitter,
    v02::{Layout, LineVariant, Node, TextAlign, Token},
};
use eframe::egui::{self, Pos2, Stroke, Vec2};

pub(super) fn divider(
    ui: &egui::Ui,
    painter: &egui::Painter,
    node: &Node,
    rect: egui::Rect,
    text: &str,
) {
    let stroke = Stroke::new(
        node.style.border_width.max(1.0),
        token(ui, node.style.border_color.unwrap_or(Token::Border)),
    );
    let line = |a: Pos2, b: Pos2| line(painter, a, b, stroke, node.variant);
    if node.vertical {
        line(
            Pos2::new(rect.center().x, rect.top()),
            Pos2::new(rect.center().x, rect.bottom()),
        );
    } else if text.is_empty() {
        line(
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        );
    } else {
        let color = token(ui, node.style.color.unwrap_or(Token::TextPrimary));
        let galley =
            painter.layout_no_wrap(text.to_owned(), style_font(&node.style, ui.ctx()), color);
        let width = galley.size().x.min((rect.width() - 32.0).max(0.0));
        let left = match node.title_placement.unwrap_or(TextAlign::Center) {
            TextAlign::Start => rect.left() + 16.0,
            TextAlign::Center => rect.center().x - width / 2.0,
            TextAlign::End => rect.right() - 16.0 - width,
        };
        let title =
            egui::Rect::from_min_size(Pos2::new(left, rect.top()), Vec2::new(width, rect.height()));
        painter
            .with_clip_rect(title.intersect(painter.clip_rect()))
            .galley(
                Pos2::new(left, rect.center().y - galley.size().y / 2.0),
                galley,
                color,
            );
        line(
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new((left - 8.0).max(rect.left()), rect.center().y),
        );
        line(
            Pos2::new((left + width + 8.0).min(rect.right()), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        );
    }
}

fn line(painter: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke, variant: LineVariant) {
    if matches!(variant, LineVariant::Solid) {
        painter.line_segment([a, b], stroke);
        return;
    }
    let horizontal = a.y == b.y;
    let clip = painter.clip_rect();
    let (start, end) = if horizontal {
        (a.x.max(clip.left()), b.x.min(clip.right()))
    } else {
        (a.y.max(clip.top()), b.y.min(clip.bottom()))
    };
    let dash = if matches!(variant, LineVariant::Dotted) {
        stroke.width
    } else {
        6.0
    };
    let step = dash + 4.0;
    for i in 0..(((end - start).max(0.0) / step).ceil() as usize).min(4096) {
        let from = start + i as f32 * step;
        let to = (from + dash).min(end);
        let points = if horizontal {
            [Pos2::new(from, a.y), Pos2::new(to, a.y)]
        } else {
            [Pos2::new(a.x, from), Pos2::new(a.x, to)]
        };
        painter.line_segment(points, stroke);
    }
}

pub(super) fn splitter(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    node: &Node,
    content: egui::Rect,
    first_rect: egui::Rect,
    panels: [&Layout; 2],
    enabled: bool,
) -> Option<f32> {
    let horizontal = !node.vertical;
    let total = if horizontal {
        content.width()
    } else {
        content.height()
    };
    let bar = splitter::bar_size(node, total);
    let available = (total - bar).max(0.0);
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
    let enabled = enabled && node.resizable != Some(false) && available > 0.0;
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
        0.0,
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
            let step = if input.modifiers.shift { 1.0 } else { 10.0 };
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
    if delta == 0.0 && !response.double_clicked() {
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
        ((current + delta) / available).clamp(0.0, 1.0)
    };
    let sizes = splitter::sizes(
        available,
        desired,
        splitter::limits(panels[0], horizontal),
        splitter::limits(panels[1], horizontal),
    );
    let ratio = (sizes[0] / available).clamp(0.0, 1.0);
    ((sizes[0] - current).abs() > 0.01).then_some(ratio)
}

#[cfg(test)]
mod tests {
    use super::super::DashboardApp;
    use super::*;
    use serde_json::json;

    fn app(vertical: bool, enabled: bool, bound: bool) -> DashboardApp {
        let mut root = json!({"type":"splitter","id":"split","vertical":vertical,
            "enabled":{"kind":"literal","value":enabled},"children":[{"type":"column","id":"a"},{"type":"column","id":"b"}]});
        root["value"] = if bound {
            json!({"kind":"state","key":"ratio"})
        } else {
            json!({"kind":"literal","value":0.25})
        };
        DashboardApp::embedded(crate::v02::parse(&json!({"schemaVersion":"0.2",
            "app":{"name":"Test","window":{"width":600,"height":400}},"state":{"ratio":0.25},"root":root,"actions":{}}).to_string()).unwrap()).unwrap()
    }
    fn frame(ctx: &egui::Context, app: &mut DashboardApp, events: Vec<egui::Event>) {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(600.0, 400.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
    }
    fn button(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        }
    }

    #[test]
    fn pointer_drag_commits_ratio_and_disabled_splitter_ignores_input() {
        for enabled in [true, false] {
            let ctx = egui::Context::default();
            let mut app = app(false, enabled, true);
            frame(&ctx, &mut app, vec![]);
            let start = Pos2::new(152.0, 100.0);
            frame(&ctx, &mut app, vec![egui::Event::PointerMoved(start)]);
            frame(&ctx, &mut app, vec![button(start, true)]);
            frame(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(Pos2::new(232.0, 100.0))],
            );
            frame(&ctx, &mut app, vec![button(Pos2::new(232.0, 100.0), false)]);
            let ratio = app.state()["ratio"].as_f64().unwrap();
            if enabled {
                assert!((ratio - 228.0 / 592.0).abs() < 0.002, "{ratio}");
                assert_eq!(
                    app.take_events()
                        .iter()
                        .filter(|e| e.kind == "change")
                        .count(),
                    1
                );
            } else {
                assert_eq!(ratio, 0.25);
                assert!(app.take_events().is_empty());
            }
        }
    }

    #[test]
    fn vertical_splitter_responds_to_keyboard_and_retains_literal_value() {
        let ctx = egui::Context::default();
        let mut app = app(true, true, false);
        frame(&ctx, &mut app, vec![]);
        ctx.memory_mut(|m| m.request_focus(egui::Id::new(("split", "splitter"))));
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::ArrowDown,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        frame(&ctx, &mut app, vec![]);
        let ratio = app.splitters["split"];
        assert!((ratio - 108.0 / 392.0).abs() < 0.002, "{ratio}");
        assert_eq!(app.state()["ratio"], json!(0.25));
        assert_eq!(app.take_events().len(), 1);
        assert_eq!(app.resolved[0].value, json!(ratio).into());
    }

    #[test]
    fn layout_tutorials_render_in_both_themes_and_multiple_viewports() {
        let pages = [
            include_str!("../../../docs/components/21-flex.md"),
            include_str!("../../../docs/components/22-space.md"),
            include_str!("../../../docs/components/23-divider.md"),
            include_str!("../../../docs/components/24-layout.md"),
            include_str!("../../../docs/components/25-splitter.md"),
            include_str!("../../../docs/components/26-masonry.md"),
        ];
        for page in pages {
            for block in page.split("```json\n").skip(1) {
                let source = block.split("```").next().unwrap();
                for dark in [true, false] {
                    let ctx = egui::Context::default();
                    ctx.set_visuals(if dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    });
                    let mut app =
                        DashboardApp::embedded(crate::v02::parse(source).unwrap()).unwrap();
                    for (w, h) in [(960.0, 640.0), (300.0, 250.0), (720.0, 480.0)] {
                        let output = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    Pos2::ZERO,
                                    Vec2::new(w, h),
                                )),
                                ..Default::default()
                            },
                            |ctx| app.draw(ctx),
                        );
                        assert!(app.runtime.error.is_none(), "{:?}", app.runtime.error);
                        assert!(!output.shapes.is_empty());
                        let boxes = crate::layout::arrange(
                            &app.runtime.tree,
                            &app.resolved,
                            crate::layout::Rect {
                                x: 0.0,
                                y: 0.0,
                                w,
                                h,
                            },
                            &mut app.scroll.clone(),
                            &super::super::text::EguiMeasure(ctx.debug_painter()),
                            1.0,
                        );
                        for b in boxes.boxes {
                            assert!(
                                [
                                    b.rect.x, b.rect.y, b.rect.w, b.rect.h, b.extent.w, b.extent.h
                                ]
                                .into_iter()
                                .all(f32::is_finite)
                            );
                            assert!(b.rect.w >= 0.0 && b.rect.h >= 0.0);
                        }
                    }
                }
            }
        }
    }
}
