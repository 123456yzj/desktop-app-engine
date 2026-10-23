//! Native component catalogue and event integration tests.
use super::*;
use serde_json::{Value, json};

fn document(control: Value, state: Value) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"Widgets","window":{"width":640,"height":480}},"root":control,"state":state,"actions":{}})
}
fn app(d: Value) -> DashboardApp {
    DashboardApp::embedded(v02::parse(&d.to_string()).unwrap()).unwrap()
}
fn frame(
    ctx: &egui::Context,
    app: &mut DashboardApp,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(640., 480.))),
            events,
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    );
    assert!(app.runtime.error.is_none(), "{:?}", app.runtime.error);
    output
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

fn showcase_document() -> Value {
    let mut d: Value =
        serde_json::from_str(include_str!("../../../examples/all-components.json")).unwrap();
    d["plugins"] = json!([]);
    fn remove_plugins(node: &mut Value) {
        if node["type"] == "plugin" {
            *node = json!({"type":"panel","id":"pluginPlaceholder"});
        }
        if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
            for child in children {
                remove_plugins(child);
            }
        }
    }
    remove_plugins(&mut d["root"]);
    d
}

#[test]
fn showcase_header_text_fits_its_clip_at_native_scales() {
    let d = showcase_document();
    for scale in [1., 1.25, 1.5, 1.75, 2.] {
        let ctx = egui::Context::default();
        #[cfg(windows)]
        fonts::install_chinese_font(&ctx).unwrap();
        ctx.set_pixels_per_point(scale);
        let mut app = app(d.clone());
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(1360., 900.),
                )),
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        let mut checked = 0;
        for shape in &output.shapes {
            if let egui::Shape::Text(text) = &shape.shape
                && [
                    "组件展示中心",
                    "布局与容器：Row / Column / Grid / Flex / Space / Layout / Masonry / Splitter",
                ]
                .contains(&text.galley.job.text.as_str())
            {
                let bounds = text.galley.rect.translate(text.pos.to_vec2());
                checked += 1;
                assert!(
                    shape.clip_rect.expand(1.).contains_rect(bounds),
                    "scale={scale} text={} bounds={bounds:?} clip={:?}",
                    text.galley.job.text,
                    shape.clip_rect
                );
            }
        }
        assert_eq!(
            checked, 2,
            "both header and page introduction must be visible"
        );
    }
}

#[test]
fn showcase_form_submit_invalid_and_reset_and_tab_menu_stay_in_sync() {
    fn draw(
        ctx: &egui::Context,
        app: &mut DashboardApp,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(1360., 900.),
                )),
                events,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        )
    }
    fn press(ctx: &egui::Context, app: &mut DashboardApp, text: &str) {
        let output = draw(ctx, app, vec![]);
        let position = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == text => {
                    Some(t.pos + t.galley.rect.center().to_vec2())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing {text}"));
        draw(ctx, app, vec![egui::Event::PointerMoved(position)]);
        for pressed in [true, false] {
            draw(
                ctx,
                app,
                vec![egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                }],
            );
        }
        assert!(app.runtime.error.is_none(), "{:?}", app.runtime.error);
    }
    let ctx = egui::Context::default();
    #[cfg(windows)]
    fonts::install_chinese_font(&ctx).unwrap();
    let mut app = app(showcase_document());
    press(&ctx, &mut app, "录入");
    assert_eq!(app.state()["selectedMenu"], json!(["pageForm"]));
    press(&ctx, &mut app, "校验并提交");
    assert_eq!(app.state()["status"], "表单校验通过，姓名：Ada");
    press(&ctx, &mut app, "×");
    assert_eq!(app.state()["formName"], "");
    press(&ctx, &mut app, "校验并提交");
    assert_eq!(
        app.state()["status"],
        "表单校验未通过，请检查姓名、年龄和条款。"
    );
    press(&ctx, &mut app, "重置表单");
    assert_eq!(app.state()["formName"], "Ada");
    assert_eq!(app.state()["status"], "表单字段已恢复初始值。");
}

#[test]
fn component_owned_titles_do_not_consume_trigger_space_or_repeat_results() {
    let ctx = egui::Context::default();
    let mut app = app(document(
        json!({"type":"column","id":"root","children":[
            {"type":"popover","id":"popup","title":"Popup heading","text":"Open popup"},
            {"type":"result","id":"result","title":"Completed","subTitle":"Ready","layout":{"height":{"fixed":140}}}
        ]}),
        json!({}),
    ));
    let output = frame(&ctx, &mut app, vec![]);
    let mut titles = 0;
    for s in output.shapes {
        if let egui::Shape::Text(t) = s.shape {
            if t.galley.job.text == "Completed" {
                titles += 1;
            }
            if t.galley.job.text == "Open popup" {
                assert!(
                    s.clip_rect
                        .expand(1.)
                        .contains_rect(t.galley.rect.translate(t.pos.to_vec2()))
                );
            }
            assert_ne!(
                t.galley.job.text, "Popup heading",
                "closed popup title belongs inside the popup"
            );
        }
    }
    assert_eq!(titles, 1);
}

#[test]
fn badge_count_reserves_space_for_its_indicator_and_label() {
    for (count, dot, show_zero, expected) in [
        (84, false, false, Some("84")),
        (120, false, false, Some("99+")),
        (0, false, false, None),
        (0, false, true, Some("0")),
        (0, true, false, None),
    ] {
        let ctx = egui::Context::default();
        let mut app = app(document(
            json!({"type":"row","id":"row","children":[{"type":"badgeCount","id":"badge","count":count,"dot":dot,"showZero":show_zero,"text":"Unread messages"}]}),
            json!({}),
        ));
        let output = frame(&ctx, &mut app, vec![]);
        let label = output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) if t.galley.job.text == "Unread messages" => {
                    let bounds = t.galley.rect.translate(t.pos.to_vec2());
                    assert!(s.clip_rect.expand(1.).contains_rect(bounds));
                    Some(bounds)
                }
                _ => None,
            })
            .expect("badge label");
        let marker = output.shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if expected == Some(t.galley.job.text.as_str()) => {
                Some(t.galley.rect.translate(t.pos.to_vec2()))
            }
            _ => None,
        });
        if expected.is_some() {
            assert!(marker.expect("count indicator").right() < label.left());
        }
        if dot {
            let circle = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Circle(c) if c.radius == 4. => Some(c.center.x + c.radius),
                    _ => None,
                })
                .expect("dot indicator");
            assert!(circle < label.left());
        }
    }
}

#[test]
fn one_menu_click_commits_change_click_and_select_in_order() {
    let ctx = egui::Context::default();
    let mut d = document(
        json!({"type":"menu","id":"menu","items":[{"key":"a","label":"Alpha"},{"key":"b","label":"Beta"}],"selectedKeys":{"kind":"state","key":"selected"},"events":{"click":"capture","select":"capture"}}),
        json!({"selected":[],"captured":[]}),
    );
    d["actions"] = json!({"capture":{"steps":[{"type":"set","target":"captured","value":{"kind":"state","key":"selected"}}]}});
    let mut app = app(d);
    frame(&ctx, &mut app, vec![]);
    assert!(app.take_events().is_empty());
    click(&ctx, &mut app, Pos2::new(25., 10.));
    assert_eq!(app.state()["selected"], json!(["a"]));
    assert_eq!(app.state()["captured"], json!(["a"]));
    let events = app.take_events();
    assert_eq!(
        events.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(),
        ["change", "click", "select"]
    );
    assert!(events.windows(2).all(|e| e[0].sequence < e[1].sequence));
    click(&ctx, &mut app, Pos2::new(25., 32.));
    assert_eq!(app.state()["selected"], json!(["b"]));
}

#[test]
fn modal_blocks_background_controls_and_escape_commits_close_events() {
    let ctx = egui::Context::default();
    let mut d = document(
        json!({"type":"column","id":"root","children":[{"type":"button","id":"background","text":{"kind":"literal","value":"Background"},"events":{"click":"background"}},{"type":"modal","id":"modal","title":"Dialog","open":{"kind":"state","key":"open"},"children":[{"type":"input","id":"input","value":{"kind":"state","key":"text"}}]}]}),
        json!({"open":true,"clicked":false,"text":""}),
    );
    d["actions"] = json!({"background":{"steps":[{"type":"set","target":"clicked","value":{"kind":"literal","value":true}}]}});
    let mut app = app(d);
    for _ in 0..3 {
        frame(&ctx, &mut app, vec![]);
    }
    click(&ctx, &mut app, Pos2::new(40., 12.));
    assert_eq!(app.state()["clicked"], false);
    assert_eq!(app.state()["open"], true);
    frame(
        &ctx,
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert_eq!(app.state()["open"], false);
    let events = app.take_events();
    assert_eq!(
        events.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(),
        ["openChange", "cancel", "close"]
    );
}

#[test]
fn collector_keeps_many_events_unique_across_frame_boundaries() {
    let ctx = egui::Context::default();
    let mut events = vec![];
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut collector = Collector {
                events: &mut events,
                base: 10,
                generation: 1,
                component: "control",
            };
            for _ in 0..30 {
                collector.push(ui, "change", None, json!(true));
            }
            collector.push_for(ui, "form", "submit");
            collector.push(ui, "close", None, Value::Null);
        });
    });
    assert_eq!(events.len(), 32);
    assert!(events.windows(2).all(|e| e[0].sequence < e[1].sequence));
}

#[test]
fn dropdown_open_and_outside_close_emit_once_for_controlled_and_local_values() {
    for controlled in [true, false] {
        let ctx = egui::Context::default();
        let mut control = json!({"type":"dropdown","id":"dropdown","text":"Actions","items":[{"key":"a","label":"Alpha"}],"layout":{"width":{"fixed":200},"height":{"fixed":40}}});
        if controlled {
            control["open"] = json!({"kind":"state","key":"open"});
        }
        let mut app = app(document(control, json!({"open":false})));
        frame(&ctx, &mut app, vec![]);
        assert!(app.take_events().is_empty());
        click(&ctx, &mut app, Pos2::new(40., 18.));
        for _ in 0..3 {
            frame(&ctx, &mut app, vec![]);
        }
        let events = app.take_events();
        assert_eq!(
            events
                .iter()
                .map(|e| (&*e.kind, e.payload.clone()))
                .collect::<Vec<_>>(),
            vec![("openChange", json!(true))],
            "controlled={controlled}"
        );
        if controlled {
            assert_eq!(app.state()["open"], true);
        }
        click(&ctx, &mut app, Pos2::new(600., 400.));
        frame(&ctx, &mut app, vec![]);
        let events = app.take_events();
        assert_eq!(
            events
                .iter()
                .map(|e| (&*e.kind, e.payload.clone()))
                .collect::<Vec<_>>(),
            vec![("openChange", json!(false))],
            "controlled={controlled}"
        );
        if controlled {
            assert_eq!(app.state()["open"], false);
        }
    }
}

#[test]
fn table_pagination_pointer_changes_page_and_selects_a_row_from_that_page() {
    let ctx = egui::Context::default();
    let control = json!({"type":"table","id":"table","rows":{"kind":"literal","value":[{"id":"a","name":"Ada"},{"id":"b","name":"Alan"},{"id":"c","name":"Grace"}]},"rowKey":"id","columns":[{"id":"name","field":"name","title":"Name","width":{"fr":1}}],"current":{"kind":"state","key":"page"},"pageSize":2,"layout":{"width":{"fixed":300},"height":{"fixed":220}}});
    let mut app = app(document(control, json!({"page":1})));
    let output = frame(&ctx, &mut app, vec![]);
    let next = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "›" => {
                Some(text.pos + text.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .expect("next page button");
    click(&ctx, &mut app, next);
    assert_eq!(app.state()["page"], 2);
    assert_eq!(
        app.take_events()
            .iter()
            .map(|e| (&*e.kind, e.payload.clone()))
            .collect::<Vec<_>>(),
        vec![("pageChange", json!(2))]
    );
    click(&ctx, &mut app, Pos2::new(40., 48.));
    let events = app.take_events();
    assert!(
        events
            .iter()
            .any(|e| e.kind == "rowSelect" && e.payload == json!("c")),
        "{events:?}"
    );
}

#[test]
fn component_tutorials_render_in_both_themes_at_narrow_and_wide_viewports() {
    for text in [
        include_str!("../../../docs/components/06-panel.md"),
        include_str!("../../../docs/components/07-card.md"),
        include_str!("../../../docs/components/09-text.md"),
        include_str!("../../../docs/components/10-button.md"),
        include_str!("../../../docs/components/11-input.md"),
        include_str!("../../../docs/components/12-select.md"),
        include_str!("../../../docs/components/13-metric.md"),
        include_str!("../../../docs/components/14-badge.md"),
        include_str!("../../../docs/components/15-progress.md"),
        include_str!("../../../docs/components/16-table.md"),
        include_str!("../../../docs/components/33-navigation.md"),
        include_str!("../../../docs/components/34-native-inputs.md"),
        include_str!("../../../docs/components/35-native-display.md"),
        include_str!("../../../docs/components/36-native-feedback.md"),
        include_str!("../../../docs/components/37-native-context.md"),
    ] {
        for block in text.split("```json").skip(1) {
            let source = block.split("```").next().unwrap();
            for dark in [true, false] {
                let ctx = egui::Context::default();
                ctx.set_visuals(if dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                });
                let mut app = app(serde_json::from_str(source).unwrap());
                for (w, h, scale) in [(900., 760., 1.), (300., 240., 1.5)] {
                    ctx.set_pixels_per_point(scale);
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
                    let events = app.take_events();
                    assert!(
                        events.is_empty(),
                        "{}: {events:?}",
                        text.lines().next().unwrap()
                    );
                    let _ = ctx.tessellate(output.shapes, scale);
                }
            }
        }
    }
}

#[test]
fn extended_catalogue_renders_without_synthetic_input_at_multiple_sizes() {
    let controls = vec![
        json!({"type":"floatButton","icon":"+"}),
        json!({"type":"icon","name":"check","rotate":35}),
        json!({"type":"anchor","items":[{"key":"section","label":"Section"}]}),
        json!({"type":"breadcrumb","items":[{"key":"home","label":"Home"}]}),
        json!({"type":"dropdown","text":"Menu","items":[{"key":"a","label":"A"}]}),
        json!({"type":"menu","items":[{"key":"a","label":"A"}]}),
        json!({"type":"pagination","total":100}),
        json!({"type":"steps","items":[{"key":"a","label":"A"}]}),
        json!({"type":"tabs","items":[{"key":"tab","label":"Tab"}],"children":[{"type":"text","id":"tab","text":{"kind":"literal","value":"Body"}}]}),
        json!({"type":"autoComplete","value":"","options":["a"]}),
        json!({"type":"cascader","value":[],"options":[{"value":"a","children":[{"value":"b"}]}]}),
        json!({"type":"colorPicker","value":"#1677ff","presets":["#ff0000"]}),
        json!({"type":"datePicker","value":"2026-10-22"}),
        json!({"type":"mentions","value":"","options":["a"]}),
        json!({"type":"rate","value":2}),
        json!({"type":"timePicker","value":"12:00:00"}),
        json!({"type":"transfer","value":[],"options":["a","b"]}),
        json!({"type":"treeSelect","value":"","options":[{"value":"a"}]}),
        json!({"type":"upload","value":[]}),
        json!({"type":"avatar","text":"AB"}),
        json!({"type":"badgeCount","count":120}),
        json!({"type":"calendar","value":"2026-10-22"}),
        json!({"type":"carousel","children":[{"type":"text","id":"slide","text":{"kind":"literal","value":"Slide"}}]}),
        json!({"type":"collapse","items":[{"key":"panel","label":"Panel"}],"activeKey":["panel"],"children":[{"type":"text","id":"panel","text":{"kind":"literal","value":"Body"}}]}),
        json!({"type":"descriptions","items":[{"label":"Name","content":"Ada"}]}),
        json!({"type":"empty"}),
        json!({"type":"image","alt":"Image"}),
        json!({"type":"list","items":[{"key":"a","label":"A","description":"Body"}]}),
        json!({"type":"listy","items":[{"key":"a","label":"A"}]}),
        json!({"type":"popover","text":"Details"}),
        json!({"type":"qrCode","value":"desktop-app-engine"}),
        json!({"type":"segmented","value":"a","options":["a","b"]}),
        json!({"type":"timeline","items":[{"label":"Started"}]}),
        json!({"type":"tooltip","text":"Help"}),
        json!({"type":"tour","steps":[{"key":"a","label":"Start"}]}),
        json!({"type":"tree","treeData":[{"key":"a","title":"A"}]}),
        json!({"type":"alert","text":"Notice"}),
        json!({"type":"drawer","open":false}),
        json!({"type":"message","text":"Saved","duration":0}),
        json!({"type":"modal","open":false}),
        json!({"type":"notification","text":"Notice","duration":0}),
        json!({"type":"popconfirm","text":"Delete?"}),
        json!({"type":"result","title":"Success","status":"success"}),
        json!({"type":"skeleton","avatar":true}),
        json!({"type":"spin","spinning":false}),
        json!({"type":"watermark","content":"Private"}),
        json!({"type":"affix","offsetTop":10}),
        json!({"type":"app"}),
        json!({"type":"borderBeam"}),
        json!({"type":"configProvider","theme":"dark"}),
    ];
    assert_eq!(controls.len(), 50);
    for (index, mut control) in controls.into_iter().enumerate() {
        control["id"] = json!(format!("widget{index}"));
        control["layout"] = json!({"width":{"fixed":300},"height":{"fixed":300}});
        for dark in [true, false] {
            let ctx = egui::Context::default();
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            let mut app = app(document(control.clone(), json!({})));
            for (w, h, scale) in [(640., 480., 1.), (280., 220., 1.5), (960., 720., 2.)] {
                ctx.set_pixels_per_point(scale);
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
                        ..Default::default()
                    },
                    |ctx| app.draw(ctx),
                );
                assert!(
                    app.runtime.error.is_none(),
                    "{control}: {:?}",
                    app.runtime.error
                );
                let events = app.take_events();
                assert!(events.is_empty(), "painting {control} generated {events:?}");
                let _ = ctx.tessellate(output.shapes, scale);
            }
        }
    }
}
