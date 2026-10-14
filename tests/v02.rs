use desktop_app_engine::{
    binding::{Binding, Format, FormatKind},
    interaction::{Runtime, UiEvent},
    loader, v02,
};
use serde_json::json;
fn source() -> serde_json::Value {
    json!({"schemaVersion":"0.2","app":{"name":"测试","window":{"width":640,"height":480}},"state":{"name":"初始","result":"","n":0.42},"root":{"type":"column","id":"root","children":[{"type":"input","id":"input","value":{"kind":"state","key":"name"}},{"type":"button","id":"button","text":{"kind":"literal","value":"确认"},"events":{"click":"copy"}}]},"actions":{"copy":{"steps":[{"type":"set","target":"result","value":{"kind":"state","key":"name"}}]}}})
}
#[test]
fn strict_versions_fields_types_and_references() {
    assert!(matches!(
        loader::parse_document(&source().to_string()).unwrap(),
        loader::Document::V02(_)
    ));
    for (field, value) in [
        ("bogus", json!(1)),
        ("rows", json!({"kind":"literal","value":[]})),
    ] {
        let mut s = source();
        s["root"][field] = value;
        assert!(
            v02::parse(&s.to_string())
                .unwrap_err()
                .contains("unknown field")
        );
    }
    let mut s = source();
    s["root"]["children"][0]["value"]["key"] = json!("missing");
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("input"));
    let mut s = source();
    s["root"]["children"][0]["value"]["key"] = json!("n");
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("字符串"));
    let mut s = source();
    s["actions"]["copy"]["steps"][0]["target"] = json!("n");
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("类型"));
}
#[test]
fn pointers_null_missing_and_formatting() {
    let state =
        std::collections::BTreeMap::from([("a".into(), json!({"x/y":{"~":null},"n":0.425}))]);
    assert!(
        Binding::State {
            key: "a".into(),
            path: "/x~1y/~0".into()
        }
        .resolve(&state)
        .unwrap()
        .is_null()
    );
    for path in ["/missing", "/x~2y", "x"] {
        assert!(
            Binding::State {
                key: "a".into(),
                path: path.into()
            }
            .resolve(&state)
            .is_err()
        );
    }
    let f = Format {
        kind: FormatKind::Percent,
        precision: 1,
        unit: String::new(),
    };
    assert_eq!(f.apply(&json!(0.425)).unwrap(), "42.5%");
    assert_eq!(f.apply(&json!(null)).unwrap(), "—");
    assert!(f.apply(&json!("0.4")).is_err());
}
#[test]
fn ordered_input_then_click_once_and_stale_generation() {
    let mut r = Runtime::new(v02::parse(&source().to_string()).unwrap()).unwrap();
    let event = |id: &str, kind: &str, payload, sequence, generation| UiEvent {
        component_id: id.into(),
        target_key: None,
        kind: kind.into(),
        payload,
        sequence,
        generation,
    };
    r.commit(vec![
        event("button", "click", json!(null), 2, 1),
        event("input", "change", json!("新值"), 1, 1),
    ]);
    assert_eq!(r.state["result"], json!("新值"));
    r.set("result", json!("保留")).unwrap();
    r.commit(vec![
        event("button", "click", json!(null), 2, 1),
        event("button", "click", json!(null), 3, 0),
    ]);
    assert_eq!(r.state["result"], json!("保留"));
    assert!(r.set("n", json!("42")).is_err());
    let before = r.state.clone();
    for _ in 0..5 {
        r.tree.resolve(&r.state).unwrap();
    }
    assert_eq!(r.state, before);
}

#[test]
fn related_binding_versions_and_parent_disabled() {
    let mut s = source();
    s["state"]["active"] = json!(true);
    s["root"]["enabled"] = json!({"kind":"state","key":"active"});
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    let mut resolved = r.tree.resolve(&r.state).unwrap();
    r.set("n", json!(0.5)).unwrap();
    assert_eq!(
        r.tree
            .resolve_changed(&r.state, &r.key_versions, 1, &mut resolved)
            .unwrap(),
        0
    );
    let prior = r.version;
    r.set("name", json!("变更")).unwrap();
    assert_eq!(
        r.tree
            .resolve_changed(&r.state, &r.key_versions, prior, &mut resolved)
            .unwrap(),
        1
    );
    r.set("active", json!(false)).unwrap();
    r.commit(vec![UiEvent {
        component_id: "button".into(),
        target_key: None,
        kind: "click".into(),
        payload: json!(null),
        sequence: 1,
        generation: 1,
    }]);
    assert_eq!(r.state["result"], json!(""));
}
