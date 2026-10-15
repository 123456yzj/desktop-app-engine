use desktop_app_engine::{component::Tree, layout::*, v02};
use serde_json::{Value, json};
fn document(style: Value) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"style","window":{"width":640,"height":480}},"state":{},"actions":{},"root":{"type":"text","id":"text","style":style,"text":{"kind":"literal","value":"style"}}})
}
#[test]
fn strict_style_colors_states_and_dimensions() {
    for color in ["surface", "#202a39", "#FFFFFF", "#12345678"] {
        v02::parse(&document(json!({"background":color,"borderColor":color,"fontFamily":"monospace","fontWeight":"bold","textAlign":"center","textWrap":false,"paddingX":8,"paddingY":4,"hover":{"background":color},"pressed":{"color":color},"disabled":{"borderColor":color}})).to_string()).unwrap();
    }
    for style in [
        json!({"background":"#123"}),
        json!({"color":"#zzzzzz"}),
        json!({"color":"red"}),
        json!({"fontFamily":"unknown"}),
        json!({"paddingX":-1}),
        json!({"paddingY":-1}),
        json!({"hover":{"unknown":true}}),
        json!({"textWrap":"false"}),
        json!({"textOverflowTooltip":"false"}),
        json!({"fontWeight":900}),
        json!({"textAlign":"invalid"}),
    ] {
        assert!(
            v02::parse(&document(style.clone()).to_string()).is_err(),
            "{style}"
        );
    }
}
#[test]
fn actual_button_paint_respects_hover_pressed_disabled_and_font() {
    use desktop_app_engine::gui_v02::DashboardApp;
    use eframe::egui;
    let mut source = document(
        json!({"background":"#123456","color":"#ffffff","borderColor":"#334455","borderWidth":1,"fontFamily":"monospace","paddingX":8,"paddingY":4,"hover":{"background":"#234567"},"pressed":{"background":"#345678"},"disabled":{"background":"#456789","color":"#aabbcc"}}),
    );
    source["root"]["type"] = json!("button");
    source["root"]["layout"] = json!({"width":{"fixed":180},"height":{"fixed":42}});
    let ctx = egui::Context::default();
    let mut app = DashboardApp::embedded(v02::parse(&source.to_string()).unwrap()).unwrap();
    let frame = |app: &mut DashboardApp, events: Vec<egui::Event>| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        )
    };
    let contains = |output: &egui::FullOutput, color: egui::Color32| {
        output
            .shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::epaint::Shape::Rect(r)if r.fill==color))
    };
    let output = frame(&mut app, vec![]);
    assert!(contains(&output, egui::Color32::from_rgb(18, 52, 86)));
    assert!(output.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t)if t.galley.job.sections[0].format.font_id.family==egui::FontFamily::Monospace)));
    let pos = egui::pos2(40.0, 20.0);
    let output = frame(&mut app, vec![egui::Event::PointerMoved(pos)]);
    assert!(contains(&output, egui::Color32::from_rgb(35, 69, 103)));
    let output = frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(contains(&output, egui::Color32::from_rgb(52, 86, 120)));
    source["root"]["enabled"] = json!({"kind":"literal","value":false});
    app.replace_description(v02::parse(&source.to_string()).unwrap())
        .unwrap();
    let output = frame(&mut app, vec![]);
    assert!(contains(&output, egui::Color32::from_rgb(69, 103, 137)));
    assert!(app.take_events().is_empty());
}
struct Text;
#[test]
fn select_popup_shows_ten_rows_before_scrolling() {
    use desktop_app_engine::gui_v02::DashboardApp;
    use eframe::egui;
    let mut source = document(json!({"fontSize":11,"paddingY":6}));
    source["root"] = json!({"type":"select","id":"branches","layout":{"width":{"fixed":180},"height":{"fixed":30}},"style":{"fontSize":11,"paddingY":6},"value":{"kind":"literal","value":""},"placeholder":{"kind":"literal","value":"Select"},"options":{"kind":"literal","value":(0..20).map(|n| format!("branch-{n:02}")).collect::<Vec<_>>()}});
    let ctx = egui::Context::default();
    let mut initial = source.clone();
    initial["root"]["options"]["value"] = json!(["Loading"]);
    let mut app = DashboardApp::embedded(v02::parse(&initial.to_string()).unwrap()).unwrap();
    let frame = |app: &mut DashboardApp, events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 900.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        )
    };
    frame(&mut app, vec![]);
    let pos = egui::pos2(40.0, 15.0);
    for pressed in [true, false] {
        frame(
            &mut app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    frame(&mut app, vec![]);
    frame(&mut app, vec![]);
    app.replace_description(v02::parse(&source.to_string()).unwrap())
        .unwrap();
    for _ in 0..3 {
        frame(&mut app, vec![]);
    }
    let output = frame(&mut app, vec![]);
    let visible = output
        .shapes
        .iter()
        .filter(|s| {
            let egui::epaint::Shape::Text(t) = &s.shape else {
                return false;
            };
            t.galley.job.text.starts_with("branch-")
                && s.clip_rect
                    .contains_rect(t.galley.rect.translate(t.pos.to_vec2()))
        })
        .count();
    assert_eq!(visible, 10, "popup must show ten complete option labels");
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(egui::pos2(60.0, 80.0)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -2000.0),
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    for _ in 0..30 {
        frame(&mut app, vec![]);
    }
    let output = frame(&mut app, vec![]);
    let last = output
        .shapes
        .iter()
        .find_map(|s| {
            let egui::epaint::Shape::Text(t) = &s.shape else {
                return None;
            };
            let rect = t.galley.rect.translate(t.pos.to_vec2());
            (t.galley.job.text == "branch-19" && s.clip_rect.contains_rect(rect))
                .then_some(rect.center())
        })
        .expect("scrolling reaches the last branch");
    for pressed in [true, false] {
        frame(
            &mut app,
            vec![
                egui::Event::PointerMoved(last),
                egui::Event::PointerButton {
                    pos: last,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    assert!(
        app.take_events()
            .iter()
            .any(|e| e.kind == "change" && e.payload == "branch-19")
    );
}
#[test]
fn overflow_tooltip_can_be_disabled_for_log_text() {
    use desktop_app_engine::gui_v02::DashboardApp;
    use eframe::egui;
    let text = "log-text-too-long-for-this-small-box";
    for enabled in [true, false] {
        let mut source =
            document(json!({"fontSize":11,"textWrap":false,"textOverflowTooltip":enabled}));
        source["root"]["layout"] = json!({"width":{"fixed":60},"height":{"fixed":18}});
        source["root"]["text"]["value"] = json!(text);
        let ctx = egui::Context::default();
        let mut app = DashboardApp::embedded(v02::parse(&source.to_string()).unwrap()).unwrap();
        let mut output = None;
        for n in 0..30 {
            output = Some(ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 480.0),
                    )),
                    time: Some(n as f64 * 0.1),
                    events: vec![egui::Event::PointerMoved(egui::pos2(10.0, 8.0))],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            ));
        }
        let count = output
            .unwrap()
            .shapes
            .iter()
            .filter(|s| matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text==text))
            .count();
        assert_eq!(
            count,
            if enabled { 2 } else { 1 },
            "tooltip enabled: {enabled}"
        );
    }
}

#[test]
fn compact_badge_text_fits_inside_its_clip_rect() {
    use desktop_app_engine::gui_v02::DashboardApp;
    use eframe::egui;
    let mut source = document(json!({"fontSize":11}));
    source["root"]["type"] = json!("badge");
    source["root"]["text"]["value"] = json!("刷新中…");
    source["root"]["layout"] = json!({"width":{"fixed":76},"height":{"fixed":22}});
    let ctx = egui::Context::default();
    desktop_app_engine::gui::install_chinese_font(&ctx).unwrap();
    let mut app = DashboardApp::embedded(v02::parse(&source.to_string()).unwrap()).unwrap();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(640.0, 480.0),
            )),
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    );
    let shape = output
        .shapes
        .iter()
        .find(
            |s| matches!(&s.shape, egui::epaint::Shape::Text(t) if t.galley.job.text == "刷新中…"),
        )
        .expect("badge text is painted");
    let egui::epaint::Shape::Text(text) = &shape.shape else {
        unreachable!()
    };
    assert_eq!(text.galley.rows.len(), 1);
    assert!(
        shape
            .clip_rect
            .contains_rect(text.galley.rect.translate(text.pos.to_vec2())),
        "badge text must fit vertically and horizontally"
    );
}
impl TextMeasure for Text {
    fn text(&self, _: &str, _: f32, _: f32) -> Size {
        Size { w: 50.0, h: 20.0 }
    }
}
#[test]
fn grid_fractional_rows_fill_remaining_height_and_respect_fixed_rows() {
    let source = json!({"schemaVersion":"0.2","app":{"name":"grid","window":{"width":640,"height":480}},"state":{},"actions":{},"root":{"type":"grid","id":"grid","columns":[{"fr":1},{"fr":1}],"layout":{"gap":10},"children":[
      {"type":"text","id":"fixed1","layout":{"height":{"fixed":40}},"text":{"kind":"literal","value":"fixed"}},
      {"type":"text","id":"fixed2","text":{"kind":"literal","value":"auto"}},
      {"type":"column","id":"fill1","layout":{"height":{"fr":1}},"children":[]},
      {"type":"column","id":"fill2","layout":{"height":{"fr":1}},"children":[]}
    ]}});
    let d = v02::parse(&source.to_string()).unwrap();
    let tree = Tree::new(&d.root);
    let resolved = tree.resolve(&d.state).unwrap();
    for h in [400.0, 800.0] {
        let mut scroll = vec![Size::default(); tree.nodes.len()];
        let result = arrange(
            &tree,
            &resolved,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 640.0,
                h,
            },
            &mut scroll,
            &Text,
            1.0,
        );
        assert_eq!(result.boxes[1].rect.h, 40.0);
        assert_eq!(result.boxes[3].rect.h, h - 50.0);
        assert_eq!(result.boxes[3].rect.h, result.boxes[4].rect.h);
        assert_eq!(result.boxes[4].rect.y + result.boxes[4].rect.h, h);
    }
}
