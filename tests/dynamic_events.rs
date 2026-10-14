use desktop_app_engine::{
    interaction::{Runtime, UiEvent},
    v02,
};
use serde_json::{Value, json};

fn source() -> Value {
    serde_json::from_str(include_str!("../examples/step-button.json")).unwrap()
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

#[test]
fn same_button_changes_action_label_enabled_and_resets() {
    let mut r = Runtime::new(v02::parse(&source().to_string()).unwrap()).unwrap();
    let mut resolved = r.tree.resolve(&r.state).unwrap();
    let button = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "advance")
        .unwrap();
    for (seq, action, label, enabled) in [
        (1, "toThird", "下一步", true),
        (2, "submit", "提交", true),
        (3, "submit", "已提交", false),
    ] {
        let prior = r.version;
        click(&mut r, "advance", seq);
        assert_eq!(r.state["wizard"]["action"], action);
        assert!(
            r.tree
                .resolve_changed(&r.state, &r.key_versions, prior, &mut resolved)
                .unwrap()
                > 0
        );
        assert_eq!(resolved[button].text, label);
        assert_eq!(resolved[button].enabled, enabled);
    }
    let version = r.version;
    click(&mut r, "advance", 4);
    assert_eq!(r.version, version);
    click(&mut r, "reset", 5);
    assert_eq!(r.state["wizard"]["action"], "toSecond");
    click(&mut r, "advance", 6);
    assert_eq!(r.state["wizard"]["action"], "toThird");
}

#[test]
fn invalid_dynamic_references_rejected_on_load_and_state_write() {
    for value in [json!("missing"), json!(""), json!(42), json!(null)] {
        let mut s = source();
        s["state"]["wizard"]["action"] = value;
        assert!(v02::parse(&s.to_string()).is_err());
    }
    for binding in [
        json!({"kind":"state","key":"missing"}),
        json!({"kind":"state","key":"wizard","path":"/missing"}),
        json!({"kind":"state","key":"wizard","path":"/action","extra":1}),
    ] {
        let mut s = source();
        s["root"]["children"][1]["events"]["click"] = binding;
        assert!(v02::parse(&s.to_string()).is_err());
    }
    let mut r = Runtime::new(v02::parse(&source().to_string()).unwrap()).unwrap();
    let original = r.state["wizard"].clone();
    let version = r.version;
    let mut invalid = original.clone();
    invalid["action"] = json!("missing");
    assert!(r.set("wizard", invalid).unwrap_err().contains("未定义动作"));
    assert_eq!(r.state["wizard"], original);
    assert_eq!(r.version, version);
}

#[test]
fn dynamic_events_resolve_after_input_and_between_ordered_clicks() {
    let mut s = source();
    s["root"]["children"].as_array_mut().unwrap().push(json!({"type":"input","id":"choice","value":{"kind":"state","key":"chosen"},"events":{"change":{"kind":"state","key":"chosen"}}}));
    s["state"]["chosen"] = json!("toSecond");
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    r.commit(vec![UiEvent {
        component_id: "choice".into(),
        target_key: None,
        kind: "change".into(),
        payload: json!("toThird"),
        sequence: 1,
        generation: 1,
    }]);
    assert!(r.error.is_none());
    assert_eq!(r.state["wizard"]["action"], "submit");
    click(&mut r, "reset", 2);
    r.commit(
        vec![4, 3]
            .into_iter()
            .map(|sequence| UiEvent {
                component_id: "advance".into(),
                target_key: None,
                kind: "click".into(),
                payload: Value::Null,
                sequence,
                generation: 1,
            })
            .collect(),
    );
    assert!(r.error.is_none());
    assert_eq!(r.state["wizard"]["action"], "submit");
}

#[test]
fn literal_action_binding_and_fixed_id_are_both_supported() {
    let mut s = source();
    s["root"]["children"][2]["events"]["click"] = json!({"kind":"literal","value":"reset"});
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    click(&mut r, "advance", 1);
    click(&mut r, "reset", 2);
    assert_eq!(r.state["wizard"]["action"], "toSecond");
}
