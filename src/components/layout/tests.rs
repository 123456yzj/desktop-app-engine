use super::*;
use crate::{
    protocol,
    runtime::{Runtime, UiEvent},
};
use serde_json::{Value, json};

struct Text;
impl TextMeasure for Text {
    fn text(&self, text: &str, width: f32, _: f32) -> Size {
        let natural = text.chars().count() as f32 * 8.0;
        Size {
            w: natural.min(width),
            h: 20.0 * (natural / width.max(1.0)).ceil().max(1.0),
        }
    }
}
fn document(root: Value) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"Layout test","window":{"width":600,"height":400}},
        "state":{"ratio":0.25,"captured":0.0},"actions":{},"root":root})
}
fn runtime(root: Value) -> Runtime {
    Runtime::new(protocol::parse(&document(root).to_string()).unwrap()).unwrap()
}
fn leaf(id: &str, height: f32) -> Value {
    json!({"type":"text","id":id,"text":{"kind":"literal","value":id},"layout":{"width":{"fixed":40},"height":{"fixed":height}}})
}
fn boxes(runtime: &Runtime, w: f32, h: f32) -> LayoutResult {
    let resolved = runtime.tree.resolve(runtime.state()).unwrap();
    arrange(
        &runtime.tree,
        &resolved,
        Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
        },
        &mut vec![Size::default(); resolved.len()],
        &Text,
        1.0,
    )
}
fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
}

#[test]
fn closable_badge_reserves_close_control_in_natural_width() {
    let r = runtime(json!({"type":"space","id":"root","children":[
        {"type":"badge","id":"plain","text":{"kind":"literal","value":"Tag"}},
        {"type":"badge","id":"closable","text":{"kind":"literal","value":"Tag"},"closable":true}
    ]}));
    let b = boxes(&r, 600., 100.);
    near(b.boxes[2].rect.w - b.boxes[1].rect.w, 20.);
    near(b.boxes[2].rect.h, b.boxes[1].rect.h);
}

#[test]
fn flex_distributes_main_axis_free_space() {
    for (justify, first, step) in [("spaceAround", 30.0, 100.0), ("spaceEvenly", 45.0, 85.0)] {
        let r = runtime(
            json!({"type":"flex","id":"root","layout":{"justify":justify},"children":[leaf("a",20.0),leaf("b",20.0),leaf("c",20.0)]}),
        );
        let b = boxes(&r, 300.0, 100.0);
        for i in 0..3 {
            near(b.boxes[i + 1].rect.x, first + step * i as f32);
        }
    }
    let r = runtime(
        json!({"type":"flex","id":"root","vertical":true,"layout":{"justify":"spaceEvenly","align":"end"},"children":[leaf("a",20.0),leaf("b",20.0)]}),
    );
    let b = boxes(&r, 300.0, 100.0);
    near(b.boxes[1].rect.y, 20.0);
    near(b.boxes[2].rect.y, 60.0);
    near(b.boxes[1].rect.x, 260.0);
}

#[test]
fn space_defaults_and_explicit_zero_gap() {
    for (layout, expected) in [(json!({}), 48.0), (json!({"gap":0}), 40.0)] {
        let r = runtime(
            json!({"type":"space","id":"root","layout":layout,"children":[leaf("a",20.0),leaf("b",20.0)]}),
        );
        let b = boxes(&r, 300.0, 100.0);
        near(b.boxes[2].rect.x, expected);
        near(b.boxes[1].rect.y, 40.0);
    }
}

#[test]
fn wrap_uses_independent_row_and_column_gaps() {
    let r = runtime(
        json!({"type":"flex","id":"root","layout":{"wrap":true,"columnGap":10,"rowGap":11},
        "children":[leaf("a",20.0),leaf("b",20.0),leaf("c",20.0)]}),
    );
    let b = boxes(&r, 90.0, 100.0);
    near(b.boxes[2].rect.x, 50.0);
    near(b.boxes[3].rect.y, 31.0);
    near(b.boxes[0].extent.h, 51.0);
    let measured = measure(
        &r.tree,
        &r.tree.resolve(r.state()).unwrap(),
        0,
        Constraint {
            width: Limit::Bounded(90.0),
            height: Limit::Unbounded,
        },
        &Text,
    );
    near(measured.h, 51.0);
}

#[test]
fn layout_regions_allocate_remaining_space() {
    let r = runtime(json!({"type":"layout","id":"root","children":[
        {"type":"header","id":"header"},
        {"type":"layout","id":"body","children":[{"type":"sider","id":"sider"},{"type":"content","id":"content"}]},
        {"type":"footer","id":"footer"}]}));
    let b = boxes(&r, 300.0, 200.0);
    near(b.boxes[1].rect.h, 64.0);
    near(b.boxes[2].rect.h, 88.0);
    near(b.boxes[3].rect.w, 200.0);
    near(b.boxes[4].rect.x, 200.0);
    near(b.boxes[4].rect.w, 100.0);
    near(b.boxes[5].rect.y, 152.0);
    near(b.boxes[5].rect.h, 48.0);
}

#[test]
fn hidden_tab_bar_releases_its_layout_space() {
    for hidden in [false, true] {
        for placement in ["top", "start"] {
            let r = runtime(json!({"type":"tabs","id":"root","hideTabBar":hidden,
                "tabPlacement":placement,"activeKey":"page","items":[{"key":"page","label":"Page"}],
                "children":[leaf("page",20.0)]}));
            let b = boxes(&r, 300.0, 200.0);
            near(
                b.boxes[1].rect.y,
                if !hidden && placement == "top" {
                    44.0
                } else {
                    0.0
                },
            );
            near(
                b.boxes[1].rect.x,
                if !hidden && placement == "start" {
                    120.0
                } else {
                    0.0
                },
            );
            let measured = measure(
                &r.tree,
                &r.tree.resolve(r.state()).unwrap(),
                0,
                Constraint {
                    width: Limit::Bounded(300.0),
                    height: Limit::Unbounded,
                },
                &Text,
            );
            near(measured.h, if hidden { 20.0 } else { 64.0 });
        }
    }
}

#[test]
fn fractional_children_in_scroll_columns_retain_natural_extent() {
    let r = runtime(
        json!({"type":"column","id":"root","layout":{"overflow":"scroll"},"children":[
            {"type":"column","id":"body","layout":{"height":{"fr":1}},"children":[leaf("large",500.0)]}
        ]}),
    );
    let b = boxes(&r, 300.0, 200.0);
    near(b.boxes[1].rect.h, 500.0);
    near(b.boxes[0].extent.h, 500.0);
}

#[test]
fn grid_supports_independent_row_and_column_gaps() {
    let r = runtime(
        json!({"type":"grid","id":"root","columns":[{"fr":1},{"fr":1}],
        "layout":{"rowGap":10,"columnGap":20},"children":[leaf("a",20.0),leaf("b",30.0),leaf("c",40.0),leaf("d",50.0)]}),
    );
    let b = boxes(&r, 220.0, 200.0);
    near(b.boxes[2].rect.x, 120.0);
    near(b.boxes[3].rect.y, 40.0);
    near(b.boxes[0].extent.h, 90.0);
}

#[test]
fn wrapped_fractional_items_use_the_same_basis_in_measure_and_arrange() {
    let children: Vec<_> = (0..3).map(|i| json!({"type":"text","id":format!("item{i}"),
        "text":{"kind":"literal","value":"item"},"layout":{"width":{"fr":1},"minWidth":80,"height":{"fixed":20}}})).collect();
    let r = runtime(
        json!({"type":"flex","id":"root","layout":{"gap":10,"wrap":true},"children":children}),
    );
    let b = boxes(&r, 200.0, 100.0);
    near(b.boxes[1].rect.w, 95.0);
    near(b.boxes[2].rect.x, 105.0);
    near(b.boxes[3].rect.y, 30.0);
    let measured = measure(
        &r.tree,
        &r.tree.resolve(r.state()).unwrap(),
        0,
        Constraint {
            width: Limit::Bounded(200.0),
            height: Limit::Unbounded,
        },
        &Text,
    );
    near(measured.h, b.boxes[0].extent.h);
}

#[test]
fn splitter_measures_text_at_the_constrained_panel_width() {
    let r = runtime(
        json!({"type":"splitter","id":"root","value":{"kind":"literal","value":0.25},"children":[
            {"type":"text","id":"a","text":{"kind":"literal","value":"abcdefghijklmnopqrstuvwxyz"}},
            {"type":"text","id":"b","text":{"kind":"literal","value":"short"}}
        ]}),
    );
    let measured = measure(
        &r.tree,
        &r.tree.resolve(r.state()).unwrap(),
        0,
        Constraint {
            width: Limit::Bounded(208.0),
            height: Limit::Unbounded,
        },
        &Text,
    );
    near(measured.h, 100.0);
}

#[test]
fn masonry_uses_shortest_column_and_reflows_on_resize() {
    let children: Vec<_> = [100.0, 30.0, 60.0, 40.0]
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            let mut child = leaf(&format!("item{i}"), h);
            child["layout"].as_object_mut().unwrap().remove("width");
            child
        })
        .collect();
    let mut root = json!({"type":"masonry","id":"root","columnCount":3,"layout":{"gap":10},"children":children});
    let r = runtime(root.clone());
    let b = boxes(&r, 320.0, 200.0);
    near(b.boxes[1].rect.w, 100.0);
    near(b.boxes[4].rect.x, 110.0);
    near(b.boxes[4].rect.y, 40.0);
    near(b.boxes[0].extent.h, 100.0);
    root["minColumnWidth"] = json!(150);
    let r = runtime(root);
    near(boxes(&r, 320.0, 200.0).boxes[1].rect.w, 155.0);
    let narrow = boxes(&r, 140.0, 400.0);
    near(narrow.boxes[1].rect.w, 140.0);
    near(narrow.boxes[2].rect.y, 110.0);
    near(narrow.boxes[0].extent.h, 260.0);
}

#[test]
fn masonry_remeasures_bound_text_and_ignores_hidden_children() {
    let mut doc = document(
        json!({"type":"masonry","id":"root","columnCount":2,"layout":{"gap":10},"children":[
            {"type":"text","id":"a","text":{"kind":"state","key":"message"}},
            {"type":"text","id":"hidden","visible":{"kind":"literal","value":false},"text":{"kind":"literal","value":"hidden"},"layout":{"height":{"fixed":500}}},
            {"type":"text","id":"b","text":{"kind":"literal","value":"short"}},
            {"type":"text","id":"c","text":{"kind":"literal","value":"last"}}
        ]}),
    );
    doc["state"]["message"] = json!("short");
    let mut r = Runtime::new(protocol::parse(&doc.to_string()).unwrap()).unwrap();
    near(boxes(&r, 210.0, 200.0).boxes[4].rect.x, 0.0);
    r.set("message", json!("abcdefghijklmnopqrstuvwxyz"))
        .unwrap();
    let b = boxes(&r, 210.0, 200.0);
    near(b.boxes[1].rect.h, 60.0);
    near(b.boxes[4].rect.x, 110.0);
    near(b.boxes[4].rect.y, 30.0);
    near(b.boxes[2].rect.h, 0.0);
}

#[test]
fn splitter_respects_constraints_and_hidden_panel() {
    let mut root = json!({"type":"splitter","id":"root","value":{"kind":"state","key":"ratio"},"children":[
        {"type":"column","id":"a","layout":{"minWidth":100}},
        {"type":"column","id":"b","layout":{"minWidth":80}}]});
    let r = runtime(root.clone());
    let b = boxes(&r, 300.0, 200.0);
    near(b.boxes[1].rect.w, 100.0);
    near(b.boxes[2].rect.x, 108.0);
    near(b.boxes[2].rect.w, 192.0);
    let tiny = boxes(&r, 100.0, 200.0);
    near(tiny.boxes[1].rect.w, 100.0);
    near(tiny.boxes[2].rect.w, 80.0);
    root["children"][0]["visible"] = json!({"kind":"literal","value":false});
    let b = boxes(&runtime(root), 300.0, 200.0);
    near(b.boxes[2].rect.w, 300.0);
    near(b.boxes[2].rect.x, 0.0);
}

#[test]
fn splitter_change_updates_state_before_action_and_rejects_invalid_ratio() {
    let mut doc = document(
        json!({"type":"splitter","id":"root","value":{"kind":"state","key":"ratio"},
        "events":{"change":"capture"},"children":[{"type":"column","id":"a"},{"type":"column","id":"b"}]}),
    );
    doc["actions"]["capture"] = json!({"steps":[{"type":"set","target":"captured","value":{"kind":"state","key":"ratio"}}]});
    let mut r = Runtime::new(protocol::parse(&doc.to_string()).unwrap()).unwrap();
    for (sequence, payload) in [(0, json!(0.75)), (1, json!(2)), (2, json!("0.4"))] {
        r.commit(vec![UiEvent {
            component_id: "root".into(),
            target_key: None,
            kind: "change".into(),
            payload,
            sequence,
            generation: r.generation(),
        }]);
        assert_eq!(r.state()["ratio"], json!(0.75));
        assert_eq!(r.state()["captured"], json!(0.75));
    }
    near(boxes(&r, 408.0, 200.0).boxes[1].rect.w, 300.0);
    assert!(r.set("ratio", json!(-0.1)).is_err());
    assert_eq!(r.state()["ratio"], json!(0.75));
}

#[test]
fn invalid_layout_component_contracts_are_rejected() {
    for root in [
        json!({"type":"masonry","id":"root","columnCount":0}),
        json!({"type":"masonry","id":"root","minColumnWidth":0}),
        json!({"type":"splitter","id":"root","children":[]}),
        json!({"type":"splitter","id":"root","barSize":2,"children":[{"type":"column","id":"a"},{"type":"column","id":"b"}]}),
        json!({"type":"divider","id":"root","vertical":true,"text":{"kind":"literal","value":"title"}}),
        json!({"type":"divider","id":"root","layout":{"rowGap":3}}),
        json!({"type":"flex","id":"root","vertical":true,"layout":{"wrap":true}}),
        json!({"type":"masonry","id":"root","layout":{"justify":"center"}}),
        json!({"type":"space","id":"root","vertical":"yes"}),
    ] {
        let source = document(root.clone()).to_string();
        assert!(
            protocol::parse(&source)
                .and_then(crate::compiler::compile)
                .is_err(),
            "accepted {root}"
        );
    }
}
