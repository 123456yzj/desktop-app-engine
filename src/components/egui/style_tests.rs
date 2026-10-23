//! Common style rendering tests: text, container, control and state styles.
use super::*;
use serde_json::{Value, json};

fn document(root: Value) -> Value {
    json!({
        "schemaVersion": "0.2",
        "app": {"name": "Style tests", "window": {"width": 640, "height": 480}},
        "state": {},
        "root": root,
        "actions": {}
    })
}
fn app(root: Value) -> DashboardApp {
    DashboardApp::embedded(v02::parse(&document(root).to_string()).unwrap()).unwrap()
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
fn rect_of(app: &DashboardApp, ctx: &egui::Context, id: &str) -> egui::Rect {
    let index = app
        .runtime
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == id)
        .unwrap_or_else(|| panic!("missing node {id}"));
    let boxes = layout::arrange(
        &app.runtime.tree,
        &app.resolved,
        Rect {
            x: 0.,
            y: 0.,
            w: 640.,
            h: 480.,
        },
        &mut app.scroll.clone(),
        &EguiMeasure(ctx.debug_painter()),
        1.,
    );
    er(boxes.boxes[index].content)
}
struct DrawnText {
    text: String,
    font: FontId,
    pos: Pos2,
    size: Vec2,
    rows: usize,
    color: Color32,
    halign: egui::Align,
}
impl DrawnText {
    fn left(&self) -> f32 {
        match self.halign {
            egui::Align::Center => self.pos.x - self.size.x / 2.,
            egui::Align::Max => self.pos.x - self.size.x,
            _ => self.pos.x,
        }
    }
}
fn texts(output: &egui::FullOutput) -> Vec<DrawnText> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(t) => Some(DrawnText {
                text: t.galley.job.text.clone(),
                font: t.galley.job.sections.first()?.format.font_id.clone(),
                pos: t.pos,
                size: t.galley.size(),
                rows: t.galley.rows.len(),
                color: t.galley.job.sections.first()?.format.color,
                halign: t.galley.job.halign,
            }),
            _ => None,
        })
        .collect()
}
fn drawn(output: &egui::FullOutput, text: &str) -> DrawnText {
    texts(output)
        .into_iter()
        .find(|t| t.text == text)
        .unwrap_or_else(|| panic!("missing text {text}"))
}
#[derive(Debug)]
struct DrawnRect {
    fill: Color32,
    stroke: Stroke,
    corner: u8,
}
fn rects(output: &egui::FullOutput) -> Vec<DrawnRect> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(r) => Some(DrawnRect {
                fill: r.fill,
                stroke: r.stroke,
                corner: r.corner_radius.nw,
            }),
            _ => None,
        })
        .collect()
}

fn column(children: Value) -> Value {
    json!({"type":"column","id":"root","layout":{"align":"start"},"children":children})
}

#[test]
fn text_style_controls_size_color_wrap_and_alignment() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"text","id":"aligned","text":{"kind":"literal","value":"Aligned"},"layout":{"width":{"fixed":200},"height":{"fixed":60}},"style":{"fontSize":26,"color":"#123456","textAlign":"center"}},
        {"type":"text","id":"wrapped","text":{"kind":"literal","value":"single line only with wrapping disabled"},"layout":{"width":{"fixed":80},"height":{"fixed":80}},"style":{"fontSize":12,"textWrap":false}},
        {"type":"text","id":"flowed","text":{"kind":"literal","value":"single line only with wrapping enabled"},"layout":{"width":{"fixed":80},"height":{"fixed":80}},"style":{"fontSize":12}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);

    let aligned = drawn(&output, "Aligned");
    assert_eq!(aligned.font.size, 26.);
    assert_eq!(aligned.color, Color32::from_rgb(0x12, 0x34, 0x56));
    let bounds = rect_of(&app, &ctx, "aligned");
    let expected_x = bounds.left() + (bounds.width() - aligned.size.x) / 2.;
    assert!(
        (aligned.left() - expected_x).abs() < 0.6,
        "center alignment expected {expected_x}, got {}",
        aligned.left()
    );
    let expected_y = bounds.top() + (bounds.height() - aligned.size.y) / 2.;
    assert!(
        (aligned.pos.y - expected_y).abs() < 0.6,
        "center alignment must also center vertically"
    );

    assert_eq!(
        drawn(&output, "single line only with wrapping disabled").rows,
        1
    );
    assert!(
        drawn(&output, "single line only with wrapping enabled").rows > 1,
        "default textWrap must wrap inside the measured width"
    );
}

#[test]
fn text_font_family_and_weight_select_distinct_fonts() {
    let ctx = egui::Context::default();
    #[cfg(windows)]
    fonts::install_chinese_font(&ctx).unwrap();
    let mut app = app(column(json!([
        {"type":"text","id":"mono","text":{"kind":"literal","value":"mono"},"style":{"fontFamily":"monospace"}},
        {"type":"text","id":"bold","text":{"kind":"literal","value":"bold"},"style":{"fontWeight":"bold"}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    assert_eq!(
        drawn(&output, "mono").font.family,
        egui::FontFamily::Monospace
    );
    #[cfg(windows)]
    assert_eq!(
        drawn(&output, "bold").font.family,
        egui::FontFamily::Name("bold".into())
    );
}

#[test]
fn style_does_not_inherit_and_size_supplies_default_font() {
    let ctx = egui::Context::default();
    let mut app = app(json!({
        "type":"column","id":"root",
        "style":{"fontSize":30,"color":"#ff0000"},
        "children":[
            {"type":"text","id":"plain","text":{"kind":"literal","value":"plain"}},
            {"type":"text","id":"small","size":"small","text":{"kind":"literal","value":"small"}},
            {"type":"text","id":"large","size":"large","text":{"kind":"literal","value":"large"}}
        ]
    }));
    let output = frame(&ctx, &mut app, vec![]);
    assert_eq!(
        drawn(&output, "plain").font.size,
        16.,
        "style must not inherit"
    );
    assert_ne!(drawn(&output, "plain").color, Color32::from_rgb(255, 0, 0));
    assert_eq!(drawn(&output, "small").font.size, 12.);
    assert_eq!(drawn(&output, "large").font.size, 18.);
}

#[test]
fn explicit_font_size_overrides_component_size() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"button","id":"btn","size":"large","text":{"kind":"literal","value":"Tap"},"style":{"fontSize":15}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    assert_eq!(drawn(&output, "Tap").font.size, 15.);
}

#[test]
fn container_style_paints_one_border_with_background_and_radius() {
    for kind in ["panel", "card"] {
        let ctx = egui::Context::default();
        let mut app = app(column(json!([
            {"type":kind,"id":"box","layout":{"width":{"fixed":200},"height":{"fixed":100}},
             "style":{"background":"#112233","radius":12,"borderWidth":3,"borderColor":"#445566"}}
        ])));
        let output = frame(&ctx, &mut app, vec![]);
        let border = rects(&output)
            .into_iter()
            .filter(|r| r.stroke.width > 0.)
            .collect::<Vec<_>>();
        assert_eq!(
            border.len(),
            1,
            "{kind} must draw exactly one border, got {border:?}"
        );
        let border = &border[0];
        assert_eq!(border.stroke.width, 3.);
        assert_eq!(border.stroke.color, Color32::from_rgb(0x44, 0x55, 0x66));
        assert!(border.corner >= 12, "radius must reach the painted frame");
        let fill = rects(&output)
            .into_iter()
            .find(|r| r.fill == Color32::from_rgb(0x11, 0x22, 0x33))
            .unwrap_or_else(|| panic!("{kind} must fill with style.background"));
        assert!(fill.corner >= 12, "radius must reach the painted fill");
    }
}

#[test]
fn bordered_false_suppresses_the_container_border() {
    for kind in ["panel", "card"] {
        let ctx = egui::Context::default();
        let mut app = app(column(json!([
            {"type":kind,"id":"box","bordered":false,"layout":{"width":{"fixed":200},"height":{"fixed":100}},
             "style":{"borderWidth":3,"borderColor":"#445566"}}
        ])));
        let output = frame(&ctx, &mut app, vec![]);
        let bordered = rects(&output)
            .into_iter()
            .filter(|r| r.stroke.width > 0.)
            .count();
        assert_eq!(
            bordered, 0,
            "{kind} with bordered=false must not paint a border"
        );
    }
}

#[test]
fn button_padding_offsets_text_and_state_styles_recolor() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"button","id":"btn","text":{"kind":"literal","value":"Padded"},
         "layout":{"width":{"fixed":200},"height":{"fixed":50}},
         "style":{"paddingX":20,"paddingY":10,"background":"#101010","hover":{"background":"#202020"},"pressed":{"background":"#303030"}}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    let bounds = rect_of(&app, &ctx, "btn");
    let text = drawn(&output, "Padded");
    assert!(
        (text.left() - (bounds.left() + 20.)).abs() < 0.6,
        "paddingX must offset button text"
    );
    assert!(
        (text.pos.y - (bounds.top() + 10.)).abs() < 0.6,
        "paddingY must offset button text"
    );
    assert!(
        rects(&output)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(0x10, 0x10, 0x10)),
        "base background style must fill the button"
    );

    let center = bounds.center();
    let output = frame(&ctx, &mut app, vec![egui::Event::PointerMoved(center)]);
    assert!(
        rects(&output)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(0x20, 0x20, 0x20)),
        "hover style must replace the button fill"
    );
    let output = frame(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }],
    );
    assert!(
        rects(&output)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(0x30, 0x30, 0x30)),
        "pressed style must replace the button fill"
    );
}

#[test]
fn disabled_style_applies_to_buttons() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"button","id":"btn","enabled":{"kind":"literal","value":false},"text":{"kind":"literal","value":"Off"},
         "layout":{"width":{"fixed":200},"height":{"fixed":50}},
         "style":{"disabled":{"background":"#909090","color":"#0a0a0a"},"background":"#101010"}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    assert!(
        rects(&output)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(0x90, 0x90, 0x90)),
        "disabled style must replace the button fill"
    );
    assert_eq!(
        drawn(&output, "Off").color,
        Color32::from_rgb(0x0a, 0x0a, 0x0a)
    );
}

#[test]
fn input_style_sets_frame_colors_and_padding() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"input","id":"field","value":{"kind":"literal","value":"hello"},
         "layout":{"width":{"fixed":240},"height":{"fixed":40}},
         "style":{"background":"#010203","borderColor":"#040506","borderWidth":2,"radius":8,"paddingX":16,"paddingY":8}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    let bounds = rect_of(&app, &ctx, "field");
    let frame_rect = rects(&output)
        .into_iter()
        .find(|r| r.fill == Color32::from_rgb(0x01, 0x02, 0x03))
        .expect("input background style must paint the edit frame");
    assert_eq!(frame_rect.stroke.color, Color32::from_rgb(0x04, 0x05, 0x06));
    assert_eq!(frame_rect.stroke.width, 2.);
    assert!(frame_rect.corner >= 8, "radius must reach the edit frame");

    let text = drawn(&output, "hello");
    assert!(
        (text.left() - (bounds.left() + 16.)).abs() < 0.6,
        "paddingX must offset the input text, got {} in {bounds:?}",
        text.left()
    );
    assert!(
        (text.pos.y - (bounds.top() + 8.)).abs() < 1.2,
        "paddingY must offset the input text, got {} in {bounds:?}",
        text.pos.y
    );
}

#[test]
fn input_state_styles_recolor_the_edit_frame() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"input","id":"field","value":{"kind":"literal","value":"hi"},
         "layout":{"width":{"fixed":240},"height":{"fixed":40}},
         "style":{"background":"#010101","borderColor":"#020202","borderWidth":1,
                   "hover":{"background":"#111111","borderColor":"#121212"},
                   "pressed":{"background":"#212121","borderColor":"#222222"}}}
    ])));
    let base = frame(&ctx, &mut app, vec![]);
    assert!(
        rects(&base)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(1, 1, 1)),
        "base background must reach the edit frame"
    );
    assert!(
        rects(&base)
            .iter()
            .any(|r| r.stroke.color == Color32::from_rgb(2, 2, 2)),
        "base border color must reach the edit frame"
    );

    let center = rect_of(&app, &ctx, "field").center();
    let hover = frame(&ctx, &mut app, vec![egui::Event::PointerMoved(center)]);
    assert!(
        rects(&hover)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(0x11, 0x11, 0x11)),
        "hover style must replace the edit frame fill"
    );
    assert!(
        rects(&hover)
            .iter()
            .any(|r| r.stroke.color == Color32::from_rgb(0x12, 0x12, 0x12)),
        "hover style must replace the edit frame border"
    );

    let pressed = frame(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: center,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }],
    );
    assert!(
        rects(&pressed)
            .iter()
            .any(|r| r.fill == Color32::from_rgb(0x21, 0x21, 0x21)),
        "pressed style must replace the edit frame fill"
    );
    assert!(
        rects(&pressed)
            .iter()
            .any(|r| r.stroke.color == Color32::from_rgb(0x22, 0x22, 0x22)),
        "pressed style must replace the edit frame border"
    );
}

#[test]
fn text_alignment_survives_typography_flags() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"text","id":"styled","text":{"kind":"literal","value":"Strong centered"},
         "strong":true,"underline":true,
         "layout":{"width":{"fixed":240},"height":{"fixed":60}},
         "style":{"fontSize":20,"textAlign":"center"}},
        {"type":"text","id":"clipped","text":{"kind":"literal","value":"clipped text stays single line"},
         "ellipsis":true,
         "layout":{"width":{"fixed":90},"height":{"fixed":40}},
         "style":{"fontSize":12,"textWrap":false}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);

    let bounds = rect_of(&app, &ctx, "styled");
    let text = drawn(&output, "Strong centered");
    let expected = bounds.left() + (bounds.width() - text.size.x) / 2.;
    assert!(
        (text.left() - expected).abs() < 0.6,
        "textAlign must keep working with strong/underline, expected {expected} got {}",
        text.left()
    );

    let clipped = drawn(&output, "clipped text stays single line");
    assert_eq!(
        clipped.rows, 1,
        "ellipsis with textWrap:false must not wrap inside a narrow box"
    );
}

#[test]
fn metric_scales_value_from_style_font_size() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"metric","id":"m","label":{"kind":"literal","value":"CPU"},
         "value":{"kind":"literal","value":42},"layout":{"width":{"fixed":200},"height":{"fixed":100}},
         "style":{"fontSize":20,"color":"#c0c0c0"}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    assert_eq!(drawn(&output, "CPU").font.size, 20.);
    assert_eq!(drawn(&output, "42").font.size, 37.5);
    assert_eq!(
        drawn(&output, "42").color,
        Color32::from_rgb(0xc0, 0xc0, 0xc0)
    );
}

#[test]
fn progress_uses_style_color_and_stroke_width() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"progress","id":"p","value":{"kind":"literal","value":0.5},
         "layout":{"width":{"fixed":200},"height":{"fixed":40}},
         "style":{"color":"#00ff00","fontSize":18}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    let info = drawn(&output, "50%");
    assert_eq!(info.color, Color32::from_rgb(0, 0xff, 0));
    assert_eq!(info.font.size, 18.);
    let track = rects(&output)
        .into_iter()
        .find(|r| r.fill != Color32::TRANSPARENT)
        .expect("linear progress must paint a track");
    assert_eq!(track.corner, 4, "progress track keeps its rounded corners");
}

#[test]
fn badge_forces_single_line_and_honors_padding() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"badge","id":"tag","text":{"kind":"literal","value":"a very long badge label"},
         "layout":{"width":{"fixed":70}},
         "style":{"fontSize":12,"paddingX":6,"paddingY":2}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    let text = drawn(&output, "a very long badge label");
    assert_eq!(text.rows, 1, "badge text must stay on one line");
    assert_eq!(text.font.size, 12.);
    let bounds = rect_of(&app, &ctx, "tag");
    assert!(
        (text.left() - (bounds.left() + 6.)).abs() < 0.6,
        "paddingX must offset badge text, got {} in {bounds:?}",
        text.left()
    );
}

#[test]
fn table_style_sets_font_and_text_color() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"table","id":"t",
         "rows":{"kind":"literal","value":[{"id":"a","name":"Ada"}]},
         "rowKey":"id",
         "columns":[{"id":"name","field":"name","title":"Name","width":{"fr":1}}],
         "layout":{"width":{"fixed":300},"height":{"fixed":160}},
         "style":{"fontSize":12,"color":"#123123"}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    assert_eq!(drawn(&output, "Name").font.size, 12.);
    assert_eq!(drawn(&output, "Ada").font.size, 12.);
    assert_eq!(
        drawn(&output, "Ada").color,
        Color32::from_rgb(0x12, 0x31, 0x23)
    );
}

#[test]
fn boolean_control_labels_use_style_font_and_color() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"checkbox","id":"cb","text":{"kind":"literal","value":"Check me"},
         "value":{"kind":"literal","value":false},
         "layout":{"width":{"fixed":220},"height":{"fixed":40}},
         "style":{"fontSize":13,"color":"#0b0c0d"}},
        {"type":"switch","id":"sw","text":{"kind":"literal","value":"Switch me"},
         "value":{"kind":"literal","value":true},
         "layout":{"width":{"fixed":220},"height":{"fixed":40}},
         "style":{"fontSize":13,"color":"#0b0c0d"}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    for text in ["Check me", "Switch me"] {
        let drawn = drawn(&output, text);
        assert_eq!(drawn.font.size, 13., "{text} font size");
        assert_eq!(
            drawn.color,
            Color32::from_rgb(0x0b, 0x0c, 0x0d),
            "{text} color"
        );
    }
}

#[test]
fn input_number_style_colors_the_edit_frame() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"inputNumber","id":"num","value":{"kind":"literal","value":3},
         "layout":{"width":{"fixed":200},"height":{"fixed":34}},
         "style":{"background":"#050607","borderColor":"#08090a","borderWidth":2,"radius":7}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    let frame_rect = rects(&output)
        .into_iter()
        .find(|r| r.fill == Color32::from_rgb(0x05, 0x06, 0x07))
        .expect("inputNumber background must paint the edit frame");
    assert_eq!(frame_rect.stroke.color, Color32::from_rgb(0x08, 0x09, 0x0a));
    assert_eq!(frame_rect.stroke.width, 2.);
    assert!(frame_rect.corner >= 7, "radius must reach the edit frame");
    assert_eq!(drawn(&output, "3").font.size, 16.);
}

#[test]
fn select_style_colors_type_and_padding() {
    let ctx = egui::Context::default();
    let mut app = app(column(json!([
        {"type":"select","id":"pick","value":{"kind":"literal","value":"a"},
         "options":{"kind":"literal","value":["a","b"]},
         "layout":{"width":{"fixed":240},"height":{"fixed":40}},
         "style":{"background":"#0a0b0c","color":"#0d0e0f","borderColor":"#101112","borderWidth":2,"radius":9,"paddingX":18}}
    ])));
    let output = frame(&ctx, &mut app, vec![]);
    let bounds = rect_of(&app, &ctx, "pick");
    let frame_rect = rects(&output)
        .into_iter()
        .find(|r| r.fill == Color32::from_rgb(0x0a, 0x0b, 0x0c))
        .expect("select background must paint the trigger");
    assert_eq!(frame_rect.stroke.color, Color32::from_rgb(0x10, 0x11, 0x12));
    assert_eq!(frame_rect.stroke.width, 2.);
    assert!(frame_rect.corner >= 9, "radius must reach the trigger");

    let label = drawn(&output, "a");
    assert_eq!(label.color, Color32::from_rgb(0x0d, 0x0e, 0x0f));
    assert!(
        (label.left() - (bounds.left() + 18.)).abs() < 0.6,
        "paddingX must offset the select label, got {} in {bounds:?}",
        label.left()
    );
}

#[test]
fn showcase_applies_declared_text_styles_at_native_scales() {
    let mut source: Value =
        serde_json::from_str(include_str!("../../../examples/all-components.json")).unwrap();
    source["plugins"] = json!([]);
    fn drop_plugins(node: &mut Value) {
        if node["type"] == "plugin" {
            *node = json!({"type":"panel","id":"pluginPlaceholder"});
        }
        if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
            for child in children {
                drop_plugins(child);
            }
        }
    }
    drop_plugins(&mut source["root"]);
    fn find_text(node: &Value, id: &str) -> String {
        if node["id"] == id {
            return node["text"]["value"]
                .as_str()
                .expect("literal text node")
                .to_owned();
        }
        node.get("children")
            .and_then(Value::as_array)
            .and_then(|children| children.iter().find_map(|c| find_text_opt(c, id)))
            .unwrap_or_else(|| panic!("missing node {id}"))
    }
    fn find_text_opt(node: &Value, id: &str) -> Option<String> {
        if node["id"] == id {
            return node["text"]["value"].as_str().map(str::to_owned);
        }
        node.get("children")
            .and_then(Value::as_array)?
            .iter()
            .find_map(|c| find_text_opt(c, id))
    }
    let title_text = find_text(&source["root"], "title");
    let sidebar_text = find_text(&source["root"], "sidebarTitle");
    let status_text = source["state"]["status"].as_str().unwrap().to_owned();
    assert_eq!(title_text, "组件展示中心");
    let ctx = egui::Context::default();
    #[cfg(windows)]
    fonts::install_chinese_font(&ctx).unwrap();
    let mut app = DashboardApp::embedded(v02::parse(&source.to_string()).unwrap()).unwrap();
    for scale in [1., 1.5] {
        ctx.set_pixels_per_point(scale);
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
        assert!(app.runtime.error.is_none(), "{:?}", app.runtime.error);
        let title = drawn(&output, &title_text);
        assert_eq!(title.font.size, 26., "scale {scale} title font size");
        #[cfg(windows)]
        assert_eq!(
            title.font.family,
            egui::FontFamily::Name("bold".into()),
            "scale {scale} title must stay bold"
        );
        assert_eq!(
            drawn(&output, &sidebar_text).font.size,
            18.,
            "scale {scale} sidebar title font size"
        );
        let status = drawn(&output, &status_text);
        assert_eq!(status.font.size, 14., "scale {scale} status font size");
        assert_eq!(
            status.rows, 1,
            "scale {scale} status must truncate to one line"
        );
    }
}
