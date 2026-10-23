//! Layout component input and tutorial regressions.
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
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(600., 400.))),
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
        let start = Pos2::new(152., 100.);
        frame(&ctx, &mut app, vec![egui::Event::PointerMoved(start)]);
        frame(&ctx, &mut app, vec![button(start, true)]);
        frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(Pos2::new(232., 100.))],
        );
        frame(&ctx, &mut app, vec![button(Pos2::new(232., 100.), false)]);
        let ratio = app.state()["ratio"].as_f64().unwrap();
        if enabled {
            assert!((ratio - 228. / 592.).abs() < 0.002, "{ratio}");
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
    assert!((ratio - 108. / 392.).abs() < 0.002, "{ratio}");
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
                let mut app = DashboardApp::embedded(crate::v02::parse(source).unwrap()).unwrap();
                for (w, h) in [(960., 640.), (300., 250.), (720., 480.)] {
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
                        crate::layout::Rect { x: 0., y: 0., w, h },
                        &mut app.scroll.clone(),
                        &text::EguiMeasure(ctx.debug_painter()),
                        1.,
                    );
                    for b in boxes.boxes {
                        assert!(
                            [
                                b.rect.x, b.rect.y, b.rect.w, b.rect.h, b.extent.w, b.extent.h
                            ]
                            .into_iter()
                            .all(f32::is_finite)
                        );
                        assert!(b.rect.w >= 0. && b.rect.h >= 0.);
                    }
                }
            }
        }
    }
}
