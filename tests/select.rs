use desktop_app_engine::{
    interaction::{Runtime, UiEvent},
    v02,
};
use serde_json::{Value, json};

fn source() -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"Select","window":{"width":600,"height":400}},
        "state":{"selected":"", "seen":"", "options":[{"label":"开发","value":"dev","disabled":false},{"label":"主线","value":"main","disabled":true}]},
        "root":{"type":"select","id":"choice","value":{"kind":"state","key":"selected"},"options":{"kind":"state","key":"options"},"events":{"change":"changed"}},
        "actions":{"changed":{"steps":[{"type":"set","target":"seen","value":{"kind":"state","key":"selected"}}]}}})
}
fn change(r: &mut Runtime, value: Value, sequence: u64) {
    r.commit(vec![UiEvent {
        component_id: "choice".into(),
        target_key: None,
        kind: "change".into(),
        payload: value,
        sequence,
        generation: 1,
    }]);
}

#[test]
fn selection_commits_before_action_and_rejects_disabled_unknown_and_non_string() {
    let mut r = Runtime::new(v02::parse(&source().to_string()).unwrap()).unwrap();
    change(&mut r, json!("dev"), 1);
    assert!(r.error.is_none());
    assert_eq!(r.state["selected"], "dev");
    assert_eq!(r.state["seen"], "dev");
    for (i, value) in [json!("main"), json!("unknown"), json!(42)]
        .into_iter()
        .enumerate()
    {
        change(&mut r, value, i as u64 + 2);
        assert!(r.error.is_some());
        assert_eq!(r.state["selected"], "dev");
    }
}

#[test]
fn lines_bind_to_command_output_and_refresh_without_changing_selection() {
    let mut s = source();
    s["state"]["options"] = json!("");
    s["root"]["optionsFormat"] = json!("lines");
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    let mut resolved = r.tree.resolve(&r.state).unwrap();
    let before = r.version;
    r.set("options", json!(" dev\r\n\nfeature/tms\n")).unwrap();
    assert_eq!(
        r.tree
            .resolve_changed(&r.state, &r.key_versions, before, &mut resolved)
            .unwrap(),
        1
    );
    let options = v02::select_options(&r.tree.nodes[0].spec, &r.state).unwrap();
    assert_eq!(
        options.iter().map(|o| o.value.as_str()).collect::<Vec<_>>(),
        vec!["dev", "feature/tms"]
    );
    change(&mut r, json!("dev"), 1);
    r.set("options", json!("release\n")).unwrap();
    assert_eq!(r.state["selected"], "dev");
    assert!(r.set("options", json!("release\nrelease")).is_err());
    assert_eq!(r.state["options"], "release\n");
}

#[test]
fn options_contract_and_read_only_props_are_validated() {
    for options in [
        json!(42),
        json!(["dev", "dev"]),
        json!([{"label":"缺少 value"}]),
        json!([{"label":"x","value":"x","extra":true}]),
    ] {
        let mut s = source();
        s["state"]["options"] = options;
        assert!(v02::parse(&s.to_string()).is_err());
    }
    let mut s = source();
    s["state"]["options"] = json!(["dev", "main"]);
    assert!(v02::parse(&s.to_string()).is_ok());
    let choice = s["root"].clone();
    s["root"] = json!({"type":"module","id":"module","props":{"selected":{"kind":"state","key":"selected"}},"children":[choice]});
    s["root"]["children"][0]["value"] = json!({"kind":"prop","key":"selected"});
    assert!(v02::parse(&s.to_string()).is_err());
}

#[test]
fn disabled_select_ignores_events_and_scoped_options_resolve() {
    let mut s = source();
    s["root"]["enabled"] = json!({"kind":"literal","value":false});
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    change(&mut r, json!("dev"), 1);
    assert_eq!(r.state["selected"], "");
    s["root"].as_object_mut().unwrap().remove("enabled");
    let choice = s["root"].clone();
    let state = s["state"].take();
    let actions = s["actions"].take();
    s["state"] = json!({});
    s["actions"] = json!({});
    s["root"] =
        json!({"type":"column","id":"scope","state":state,"actions":actions,"children":[choice]});
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    change(&mut r, json!("dev"), 1);
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.state_from("choice", "selected").unwrap(), "dev");
}
