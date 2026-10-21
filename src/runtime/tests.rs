use super::*;
use serde_json::json;

fn document(root: Value, state: Value, actions: Value) -> Description {
    crate::v02::parse(&json!({"schemaVersion":"0.2","app":{"name":"Test","window":{"width":800,"height":600}},"state":state,"root":root,"actions":actions}).to_string()).unwrap()
}
fn event(id: &str, sequence: u64, generation: u64) -> UiEvent {
    UiEvent {
        component_id: id.into(),
        kind: "click".into(),
        target_key: None,
        payload: Value::Null,
        sequence,
        generation,
    }
}

#[test]
fn compilation_is_repeatable_and_authored_input_stays_unresolved() {
    let root = json!({"type":"module","id":"page","state":{"message":"initial"},"children":[{"type":"text","id":"label","text":{"kind":"state","key":"message"}}]});
    let authored = document(root, json!({}), json!({}));
    let mut first = Runtime::new(authored.clone()).unwrap();
    let second = Runtime::new(authored.clone()).unwrap();
    first
        .set_from("label", "message", json!("updated"))
        .unwrap();
    assert_eq!(second.state_from("label", "message").unwrap(), "initial");
    assert_eq!(authored.root.state["message"], "initial");
    assert!(authored.state.is_empty());
}

#[test]
fn rejected_batch_keeps_state_and_all_versions_unchanged() {
    let mut runtime = Runtime::new(document(
        json!({"type":"column","id":"root"}),
        json!({"a":"old","b":1}),
        json!({}),
    ))
    .unwrap();
    let before = runtime.state().clone();
    let versions = runtime.key_versions.clone();
    let version = runtime.version();
    assert!(
        runtime
            .set_many(BTreeMap::from([
                ("a".into(), json!("new")),
                ("b".into(), json!(false))
            ]))
            .is_err()
    );
    assert_eq!(runtime.state(), &before);
    assert_eq!(runtime.version(), version);
    assert_eq!(runtime.key_versions, versions);
    runtime.set("a", json!("old")).unwrap();
    assert_eq!(runtime.version(), version);
    runtime.set("a", json!("new")).unwrap();
    assert_eq!(runtime.version(), version + 1);
    assert_eq!(runtime.key_versions["b"], versions["b"]);
}

#[test]
fn ordered_events_resolve_local_actions_and_ignore_stale_generations() {
    let module = |id: &str| json!({"type":"module","id":id,"state":{"value":"old"},"actions":{"update":{"steps":[{"type":"set","target":"value","value":{"kind":"literal","value":"new"}}]}},"children":[{"type":"button","id":format!("{id}-button"),"text":{"kind":"literal","value":"Update"},"events":{"click":"update"}}]});
    let mut runtime = Runtime::new(document(
        json!({"type":"column","id":"root","children":[module("left"),module("right")]}),
        json!({}),
        json!({}),
    ))
    .unwrap();
    runtime.commit(vec![
        event("right-button", 2, 0),
        event("left-button", 1, 1),
    ]);
    assert_eq!(runtime.state_from("left-button", "value").unwrap(), "new");
    assert_eq!(runtime.state_from("right-button", "value").unwrap(), "old");
    runtime.commit(vec![event("right-button", 1, 1)]);
    assert_eq!(runtime.state_from("right-button", "value").unwrap(), "old");
    runtime.commit(vec![event("right-button", 2, 1)]);
    assert_eq!(runtime.state_from("right-button", "value").unwrap(), "new");
}

#[test]
fn module_emit_uses_parent_listener_metadata_and_readonly_props() {
    let root = json!({"type":"module","id":"editor","state":{"draft":"hello"},"props":{"message":{"kind":"state","key":"shared"}},"emits":{"changed":""},"on":{"changed":"receive"},"actions":{"publish":{"steps":[{"type":"emit","event":"changed","payload":{"kind":"state","key":"draft"}}]}},"children":[{"type":"button","id":"publish","text":{"kind":"prop","key":"message"},"events":{"click":"publish"}}]});
    let actions = json!({"receive":{"steps":[{"type":"set","target":"shared","value":{"kind":"event","path":""}}]}});
    let mut runtime = Runtime::new(document(root, json!({"shared":"old"}), actions)).unwrap();
    runtime.commit(vec![event("publish", 1, 1)]);
    assert_eq!(runtime.error(), None);
    assert_eq!(runtime.state()["shared"], "hello");
    assert!(
        runtime
            .set_from("publish", "message", json!("overwrite"))
            .is_err()
    );
}

#[test]
fn parsing_and_semantic_validation_are_distinct_and_metadata_is_not_authorable() {
    let source = json!({"schemaVersion":"0.2","app":{"name":"Test","window":{"width":800,"height":600}},"state":{},"root":{"type":"text","id":"root","text":{"kind":"state","key":"missing"}},"actions":{}});
    let parsed = crate::v02::parse(&source.to_string()).unwrap();
    assert!(compiler::compile(parsed).is_err());
    assert!(crate::loader::parse_document(&source.to_string()).is_err());
    let mut source = source;
    source["root"]["registration"] = json!({});
    assert!(crate::v02::parse(&source.to_string()).is_err());
}
