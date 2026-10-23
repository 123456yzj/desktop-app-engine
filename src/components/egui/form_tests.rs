//! Component input and form integration tests.
use super::*;
use serde_json::{Value, json};

fn document(control: Value, state: Value) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"Input tests","window":{"width":600,"height":500}},"state":state,"actions":{},
        "root":{"type":"column","id":"root","layout":{"align":"start"},"children":[control]}})
}
fn app(mut control: Value, state: Value) -> DashboardApp {
    control["layout"] = json!({"width":{"fixed":240},"height":{"fixed":if control["type"] == "inputNumber" { 34 } else { 120 }}});
    DashboardApp::embedded(v02::parse(&document(control, state).to_string()).unwrap()).unwrap()
}
fn frame(ctx: &egui::Context, app: &mut DashboardApp, events: Vec<egui::Event>) {
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(600.0, 500.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    );
    assert!(app.runtime.error.is_none(), "{:?}", app.runtime.error);
}
fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}
fn click(ctx: &egui::Context, app: &mut DashboardApp, pos: Pos2) {
    frame(ctx, app, vec![egui::Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            ctx,
            app,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            }],
        );
    }
}
fn rect(ctx: &egui::Context, app: &DashboardApp, name: &str) -> egui::Rect {
    let id = app
        .runtime
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == name)
        .unwrap();
    let b = layout::arrange(
        &app.runtime.tree,
        &app.resolved,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 600.0,
            h: 500.0,
        },
        &mut app.scroll.clone(),
        &EguiMeasure(ctx.debug_painter()),
        1.0,
    );
    er(b.boxes[id].content)
}

#[test]
fn boolean_widgets_support_pointer_keyboard_and_parent_disabling() {
    for kind in ["checkbox", "switch"] {
        let ctx = egui::Context::default();
        let mut app = app(
            json!({"type":kind,"id":"control","text":{"kind":"literal","value":"Enabled"},"value":{"kind":"state","key":"value"}}),
            json!({"value":false}),
        );
        frame(&ctx, &mut app, vec![]);
        click(&ctx, &mut app, Pos2::new(12.0, 60.0));
        assert_eq!(app.state()["value"], json!(true), "{kind}");
        // The clicked widget keeps focus for Space activation.
        frame(&ctx, &mut app, vec![key(egui::Key::Space)]);
        assert_eq!(app.state()["value"], json!(false), "{kind}");
        assert_eq!(
            app.take_events()
                .iter()
                .filter(|e| e.kind == "change")
                .count(),
            2
        );
        let mut d = document(
            json!({"type":kind,"id":"control","value":{"kind":"state","key":"value"},"layout":{"width":{"fixed":240},"height":{"fixed":34}}}),
            json!({"value":false}),
        );
        d["root"]["enabled"] = json!({"kind":"literal","value":false});
        let mut disabled = DashboardApp::embedded(v02::parse(&d.to_string()).unwrap()).unwrap();
        frame(&ctx, &mut disabled, vec![]);
        click(&ctx, &mut disabled, Pos2::new(12.0, 17.0));
        assert_eq!(disabled.state()["value"], json!(false));
        assert!(disabled.take_events().is_empty());
    }
}

#[test]
fn literal_controls_keep_local_values_without_writing_state() {
    let ctx = egui::Context::default();
    let mut app = app(
        json!({"type":"switch","id":"control","value":{"kind":"literal","value":false}}),
        json!({"other":false}),
    );
    frame(&ctx, &mut app, vec![]);
    click(&ctx, &mut app, Pos2::new(12.0, 60.0));
    frame(&ctx, &mut app, vec![]);
    assert_eq!(app.control_values["control"], json!(true));
    assert_eq!(app.state()["other"], json!(false));
}

#[test]
fn radio_keyboard_skips_disabled_options_and_keeps_focus() {
    let ctx = egui::Context::default();
    let mut app = app(
        json!({"type":"radio","id":"control","vertical":true,"value":{"kind":"state","key":"value"},"options":{"kind":"literal","value":["a",{"label":"b","value":"b","disabled":true},"c"]}}),
        json!({"value":"a"}),
    );
    frame(&ctx, &mut app, vec![]);
    ctx.memory_mut(|m| m.request_focus(egui::Id::new(("control", "radio", 0_usize))));
    frame(&ctx, &mut app, vec![key(egui::Key::ArrowRight)]);
    assert_eq!(app.state()["value"], json!("c"));
    frame(&ctx, &mut app, vec![key(egui::Key::ArrowLeft)]);
    assert_eq!(app.state()["value"], json!("a"));
}

#[test]
fn input_number_steps_rounds_and_restores_invalid_text() {
    let ctx = egui::Context::default();
    let mut app = app(
        json!({"type":"inputNumber","id":"control","value":{"kind":"state","key":"value"},"min":0,"max":1,"step":0.1,"precision":2}),
        json!({"value":0.1}),
    );
    frame(&ctx, &mut app, vec![]);
    click(&ctx, &mut app, Pos2::new(228.0, 8.0));
    assert_eq!(app.state()["value"], json!(0.2));
    ctx.memory_mut(|m| m.request_focus(egui::Id::new(("control", "number"))));
    frame(&ctx, &mut app, vec![key(egui::Key::ArrowUp)]);
    assert_eq!(app.state()["value"], json!(0.3));
    let select_all = egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::CTRL,
    };
    frame(
        &ctx,
        &mut app,
        vec![select_all, egui::Event::Text("abc".into())],
    );
    assert_eq!(app.state()["value"], json!(0.3));
    frame(&ctx, &mut app, vec![key(egui::Key::Enter)]);
    assert_eq!(app.number_edits["control"].text, "0.30");
    app.runtime.set("value", json!(0.8)).unwrap();
    frame(&ctx, &mut app, vec![]);
    assert_eq!(app.number_edits["control"].text, "0.80");
}

#[test]
fn sliders_use_steps_from_the_minimum_with_pointer_and_keyboard() {
    for vertical in [false, true] {
        let ctx = egui::Context::default();
        let mut app = app(
            json!({"type":"slider","id":"control","vertical":vertical,"value":{"kind":"state","key":"value"},"min":0.15,"max":1.0,"step":0.2}),
            json!({"value":0.15}),
        );
        frame(&ctx, &mut app, vec![]);
        let p = if vertical {
            Pos2::new(120.0, 40.0)
        } else {
            Pos2::new(120.0, 60.0)
        };
        click(&ctx, &mut app, p);
        let value = app.state()["value"].as_f64().unwrap();
        assert!(value > 0.15 && value <= 1.0, "{value}");
        assert!(
            (value - 1.0).abs() < 1e-9
                || (((value - 0.15) / 0.2).round() - (value - 0.15) / 0.2).abs() < 1e-9
        );
        frame(
            &ctx,
            &mut app,
            vec![key(if vertical {
                egui::Key::ArrowDown
            } else {
                egui::Key::ArrowLeft
            })],
        );
        assert!(app.state()["value"].as_f64().unwrap() < value);
    }
}

#[test]
fn form_buttons_and_enter_route_through_validation_and_reset() {
    let ctx = egui::Context::default();
    let mut d = document(
        json!({"type":"form","id":"form","events":{"submit":"save","invalid":"invalid"},"children":[
            {"type":"formItem","id":"item","label":{"kind":"literal","value":"Name"},"rules":[{"required":true}],"children":[{"type":"input","id":"name","value":{"kind":"state","key":"value"}}]},
            {"type":"button","id":"save","htmlType":"submit","text":{"kind":"literal","value":"Save"}},
            {"type":"button","id":"reset","htmlType":"reset","text":{"kind":"literal","value":"Reset"}}
        ]}),
        json!({"value":"","status":"idle"}),
    );
    d["actions"] = json!({"save":{"steps":[{"type":"set","target":"status","value":{"kind":"state","key":"value"}}]},"invalid":{"steps":[{"type":"set","target":"status","value":{"kind":"literal","value":"invalid"}}]}});
    let mut app = DashboardApp::embedded(v02::parse(&d.to_string()).unwrap()).unwrap();
    frame(&ctx, &mut app, vec![]);
    let save = rect(&ctx, &app, "save").center();
    click(&ctx, &mut app, save);
    assert_eq!(app.state()["status"], json!("invalid"));
    assert!(app.runtime.form_error("item").is_some());
    ctx.memory_mut(|m| m.request_focus(egui::Id::new(("name", "input"))));
    frame(&ctx, &mut app, vec![egui::Event::Text("Ada".into())]);
    frame(&ctx, &mut app, vec![key(egui::Key::Enter)]);
    assert_eq!(app.state()["status"], json!("Ada"));
    let reset = rect(&ctx, &app, "reset").center();
    click(&ctx, &mut app, reset);
    assert_eq!(app.state()["value"], json!(""));
    assert!(app.runtime.form_error("item").is_none());
    assert_eq!(
        app.take_events()
            .iter()
            .filter(|e| e.component_id == "form" && e.kind == "submit")
            .count(),
        2
    );
}

#[test]
fn input_tutorials_render_without_events_across_themes_sizes_and_scales() {
    let pages = [
        include_str!("../../../docs/components/27-checkbox.md"),
        include_str!("../../../docs/components/28-radio.md"),
        include_str!("../../../docs/components/29-switch.md"),
        include_str!("../../../docs/components/30-input-number.md"),
        include_str!("../../../docs/components/31-slider.md"),
        include_str!("../../../docs/components/32-form.md"),
    ];
    for page in pages {
        for block in page.split("```json\n").skip(1) {
            let source = block.split("```").next().unwrap();
            for dark in [true, false] {
                for scale in [1.0, 1.5, 2.0] {
                    let ctx = egui::Context::default();
                    ctx.set_pixels_per_point(scale);
                    ctx.set_visuals(if dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    });
                    let mut app = DashboardApp::embedded(v02::parse(source).unwrap()).unwrap();
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
                        assert!(
                            app.take_events().is_empty(),
                            "Painting must not submit input"
                        );
                        let b = layout::arrange(
                            &app.runtime.tree,
                            &app.resolved,
                            Rect {
                                x: 0.0,
                                y: 0.0,
                                w,
                                h,
                            },
                            &mut app.scroll.clone(),
                            &EguiMeasure(ctx.debug_painter()),
                            scale,
                        );
                        for item in b.boxes {
                            assert!(
                                [
                                    item.rect.x,
                                    item.rect.y,
                                    item.rect.w,
                                    item.rect.h,
                                    item.content.w,
                                    item.content.h,
                                    item.extent.w,
                                    item.extent.h
                                ]
                                .into_iter()
                                .all(f32::is_finite)
                            );
                            assert!(
                                item.rect.w >= 0.0
                                    && item.rect.h >= 0.0
                                    && item.content.w >= 0.0
                                    && item.content.h >= 0.0
                            );
                        }
                    }
                }
            }
        }
    }
}
