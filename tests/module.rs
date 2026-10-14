use desktop_app_engine::{
    interaction::{Runtime, UiEvent},
    layout::{self, Rect, Size, TextMeasure},
    v02,
};
use serde_json::{Value, json};

fn source() -> Value {
    serde_json::from_str(include_str!("../examples/modules.json")).unwrap()
}
fn runtime(s: Value) -> Runtime {
    Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap()
}
fn click(r: &mut Runtime, component: &str, sequence: u64) {
    r.commit(vec![UiEvent {
        component_id: component.into(),
        target_key: None,
        kind: "click".into(),
        payload: Value::Null,
        sequence,
        generation: 1,
    }]);
    assert!(r.error.is_none(), "{:?}", r.error);
}
struct Text;
impl TextMeasure for Text {
    fn text(&self, s: &str, width: f32, font: f32) -> Size {
        Size {
            w: (s.chars().count() as f32 * font).min(width),
            h: font,
        }
    }
}

#[test]
fn nested_module_layout_and_reactivity_are_independent() {
    let mut r = runtime(source());
    assert_eq!(r.tree.nodes[0].spec.kind, v02::Kind::Module);
    let mut resolved = r.tree.resolve(&r.state).unwrap();
    let a = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "buttonA")
        .unwrap();
    let b = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "buttonB")
        .unwrap();
    let mut scroll = vec![Size::default(); r.tree.nodes.len()];
    let boxes = layout::arrange(
        &r.tree,
        &resolved,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 800.0,
            h: 600.0,
        },
        &mut scroll,
        &Text,
        1.0,
    );
    assert!(boxes.boxes[a].rect.h > 0.0);
    assert!(boxes.boxes[b].rect.y > boxes.boxes[a].rect.y);
    let prior = r.version;
    click(&mut r, "buttonA", 1);
    assert_eq!(
        r.tree
            .resolve_changed(&r.state, &r.key_versions, prior, &mut resolved)
            .unwrap(),
        1
    );
    assert_eq!(resolved[a].text, "提交");
    assert_eq!(resolved[b].text, "下一步");
    click(&mut r, "buttonA", 2);
    assert_eq!(
        r.state_from("moduleA", "message").unwrap(),
        "A：已提交（本地演示）"
    );
    assert_eq!(r.state_from("moduleB", "message").unwrap(), "B：尚未更新");
    assert!(r.state_from("page", "message").is_err());
}

#[test]
fn module_inherits_page_state_and_shadows_nested_state() {
    let mut s = source();
    s["root"]["children"][1]["children"]
        .as_array_mut()
        .unwrap()
        .push(
            json!({"type":"module","id":"inner","state":{"message":"inner"},"children":[
                {"type":"text","id":"innerMessage","text":{"kind":"state","key":"message"}},
                {"type":"text","id":"innerHeading","text":{"kind":"state","key":"pageTitle"}}
            ]}),
        );
    let mut r = runtime(s);
    r.set_from("inner", "message", json!("changed")).unwrap();
    assert_eq!(r.state_from("innerMessage", "message").unwrap(), "changed");
    assert_eq!(r.state_from("moduleA", "message").unwrap(), "A：尚未更新");
    assert_eq!(
        r.state_from("innerHeading", "pageTitle").unwrap(),
        "模块 A、B 的状态和动作分别管理"
    );
}

#[test]
fn module_rejects_unsupported_fields_and_business_events() {
    for (field, value) in [
        ("title", json!({"kind":"literal","value":"bad"})),
        ("events", json!({"click":"next"})),
        ("columns", json!([])),
    ] {
        let mut s = source();
        s["root"]["children"][1][field] = value;
        assert!(v02::parse(&s.to_string()).is_err(), "{field}");
    }
    let mut s = source();
    s["root"]["layout"]["wrap"] = json!(true);
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][0]["text"] = json!({"kind":"state","key":"message"});
    assert!(v02::parse(&s.to_string()).is_err());
}

#[test]
fn empty_and_disabled_modules_follow_container_rules() {
    let mut s = source();
    s["root"]["children"][1]["enabled"] = json!({"kind":"literal","value":false});
    s["root"]["children"]
        .as_array_mut()
        .unwrap()
        .push(json!({"type":"module","id":"empty","state":{"local":null}}));
    let mut r = runtime(s);
    let version = r.version;
    click(&mut r, "buttonA", 1);
    assert_eq!(r.version, version);
    click(&mut r, "buttonB", 2);
    assert_eq!(r.state_from("buttonB", "button").unwrap()["text"], "提交");
    assert!(r.state_from("empty", "local").unwrap().is_null());
    let resolved = r.tree.resolve(&r.state).unwrap();
    let mut scroll = vec![Size::default(); r.tree.nodes.len()];
    let boxes = layout::arrange(
        &r.tree,
        &resolved,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 800.0,
            h: 600.0,
        },
        &mut scroll,
        &Text,
        1.0,
    );
    assert!(
        boxes
            .boxes
            .iter()
            .all(|b| b.rect.w.is_finite() && b.rect.h.is_finite())
    );
}
