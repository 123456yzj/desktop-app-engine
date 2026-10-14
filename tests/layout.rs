use desktop_app_engine::layout::*;
#[test]
fn weighted_freeze_and_negative_space() {
    let i = |base, min, max, grow, shrink| FlexItem {
        base,
        min,
        max,
        grow,
        shrink,
    };
    assert_eq!(
        flex(
            &[i(20.0, 0.0, 30.0, 1.0, 1.0), i(20.0, 0.0, 100.0, 1.0, 1.0)],
            100.0,
            10.0
        ),
        vec![30.0, 60.0]
    );
    assert_eq!(
        flex(
            &[
                i(80.0, 60.0, 100.0, 0.0, 1.0),
                i(80.0, 10.0, 100.0, 0.0, 1.0)
            ],
            100.0,
            0.0
        ),
        vec![60.0, 40.0]
    );
    assert_eq!(
        flex(
            &[
                i(80.0, 60.0, 100.0, 0.0, 1.0),
                i(80.0, 60.0, 100.0, 0.0, 1.0)
            ],
            100.0,
            0.0
        ),
        vec![60.0, 60.0]
    );
    assert!(flex(&[], 100.0, 12.0).is_empty());
}
#[test]
fn wrap_clip_scroll_and_remainder() {
    assert_eq!(
        wrap_lines(&[40.0, 40.0, 120.0, 20.0], 100.0, 10.0),
        vec![0..2, 2..3, 3..4]
    );
    assert_eq!(
        Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0
        }
        .intersect(Rect {
            x: 90.0,
            y: 90.0,
            w: 50.0,
            h: 50.0
        }),
        Rect {
            x: 90.0,
            y: 90.0,
            w: 10.0,
            h: 10.0
        }
    );
    let mut s = 90.0;
    assert_eq!(scroll_axis(&mut s, 30.0, 200.0, 100.0), 20.0);
    assert_eq!(s, 100.0);
}
struct Text;
impl TextMeasure for Text {
    fn text(&self, s: &str, width: f32, font: f32) -> Size {
        let w = s.chars().count() as f32 * font;
        Size {
            w: w.min(width),
            h: (w / width.max(1.0)).ceil().max(1.0) * font,
        }
    }
}
#[test]
fn grid_rectangles_hidden_empty_and_unbounded_fill() {
    let source = r#"{"schemaVersion":"0.2","app":{"name":"布局","window":{"width":640,"height":480}},"state":{},"actions":{},"root":{"type":"grid","id":"g","columns":[{"fixed":100},{"fr":1}],"layout":{"gap":10,"padding":10},"children":[{"type":"text","id":"a","text":{"kind":"literal","value":"甲"}},{"type":"text","id":"b","text":{"kind":"literal","value":"乙"}},{"type":"text","id":"hidden","visible":{"kind":"literal","value":false},"text":{"kind":"literal","value":"隐藏"}}]}}"#;
    let d = desktop_app_engine::v02::parse(source).unwrap();
    let tree = desktop_app_engine::component::Tree::new(&d.root);
    let r = tree.resolve(&d.state).unwrap();
    let mut scroll = vec![Size::default(); tree.nodes.len()];
    let l = arrange(
        &tree,
        &r,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 200.0,
        },
        &mut scroll,
        &Text,
        1.0,
    );
    assert_eq!(
        l.boxes[1].rect,
        Rect {
            x: 10.0,
            y: 10.0,
            w: 100.0,
            h: 16.0
        }
    );
    assert_eq!(l.boxes[2].rect.x, 120.0);
    assert_eq!(l.boxes[2].rect.w, 190.0);
    assert_eq!(l.boxes[3].rect, Rect::default());
    let mut root = d.root.clone();
    root.layout.width = Some(desktop_app_engine::v02::Track {
        fr: Some(1.0),
        fixed: None,
        auto: None,
    });
    root.children.clear();
    root.kind = desktop_app_engine::v02::Kind::Column;
    let tree = desktop_app_engine::component::Tree::new(&root);
    let r = tree.resolve(&d.state).unwrap();
    assert_eq!(
        measure(
            &tree,
            &r,
            0,
            Constraint {
                width: Limit::Unbounded,
                height: Limit::Unbounded
            },
            &Text
        ),
        Size { w: 20.0, h: 20.0 }
    );
}

#[test]
fn grid_unbounded_fill_uses_preferred_and_empty_padding() {
    let source = r#"{"schemaVersion":"0.2","app":{"name":"布局","window":{"width":640,"height":480}},"state":{},"actions":{},"root":{"type":"grid","id":"g","columns":[{"fr":1}],"layout":{"padding":10},"children":[{"type":"text","id":"t","text":{"kind":"literal","value":"甲乙"}}]}}"#;
    let mut d = desktop_app_engine::v02::parse(source).unwrap();
    let run = |d: &desktop_app_engine::v02::Description| {
        let tree = desktop_app_engine::component::Tree::new(&d.root);
        let r = tree.resolve(&d.state).unwrap();
        measure(
            &tree,
            &r,
            0,
            Constraint {
                width: Limit::Unbounded,
                height: Limit::Unbounded,
            },
            &Text,
        )
    };
    assert_eq!(run(&d), Size { w: 52.0, h: 36.0 });
    d.root.children.clear();
    assert_eq!(run(&d), Size { w: 20.0, h: 20.0 });
}
