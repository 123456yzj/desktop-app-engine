use desktop_app_engine::{
    interaction::{Runtime, UiEvent},
    v02,
};
use serde_json::{Value, json};

fn source() -> Value {
    serde_json::from_str(include_str!("../examples/module-linkage.json")).unwrap()
}
fn runtime(s: Value) -> Runtime {
    Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap()
}
fn input(r: &mut Runtime, value: &str, sequence: u64) {
    r.commit(vec![UiEvent {
        component_id: "draftInput".into(),
        target_key: None,
        kind: "change".into(),
        payload: json!(value),
        sequence,
        generation: 1,
    }]);
}

#[test]
fn input_emit_parent_handler_and_sibling_props_update_in_one_commit() {
    let mut r = runtime(source());
    let mut resolved = r.tree.resolve(&r.state).unwrap();
    let preview = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "previewText")
        .unwrap();
    assert_eq!(resolved[preview].text, "等待编辑模块发送内容");
    let before = r.version;
    input(&mut r, "中文新内容", 1);
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.state_from("page", "sharedMessage").unwrap(), "中文新内容");
    assert_eq!(r.state_from("editor", "draft").unwrap(), "中文新内容");
    assert!(
        r.tree
            .resolve_changed(&r.state, &r.key_versions, before, &mut resolved)
            .unwrap()
            > 0
    );
    assert_eq!(resolved[preview].text, "中文新内容");
    assert!(r.state_from("page", "draft").is_err());
    assert!(r.state_from("preview", "draft").is_err());
    r.set_from("page", "sharedMessage", json!("父级主动更新"))
        .unwrap();
    let resolved = r.tree.resolve(&r.state).unwrap();
    assert_eq!(resolved[preview].text, "父级主动更新");
    assert_eq!(r.state_from("editor", "draft").unwrap(), "中文新内容");
}

#[test]
fn props_are_readonly_and_declarations_resolve_in_parent_scope() {
    let mut r = runtime(source());
    assert!(r.set_from("preview", "message", json!("bad")).is_err());
    for step in [
        json!({"type":"set","target":"message","value":{"kind":"literal","value":"bad"}}),
        json!({"type":"concat","target":"message","parts":[]}),
    ] {
        let mut s = source();
        s["root"]["children"][2]["actions"] = json!({"bad":{"steps":[step]}});
        assert!(v02::parse(&s.to_string()).unwrap_err().contains("只读"));
    }
    let mut s = source();
    s["root"]["children"][1]["children"][0]["children"][0]["value"] =
        json!({"kind":"prop","key":"currentMessage"});
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("只读"));
    let mut s = source();
    s["root"]["children"][1]["props"]["bad"] = json!({"kind":"state","key":"draft"});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][1]["props"]["draft"] = json!({"kind":"literal","value":"conflict"});
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("同名"));
}

#[test]
fn nested_props_paths_literal_values_and_multiple_instances_stay_independent() {
    let mut s = source();
    s["root"]["state"]["settings"] = json!({"data":{"message":"parent"}});
    s["root"]["children"][2]["props"] = json!({"message":{"kind":"state","key":"settings","path":"/data"},"caption":{"kind":"literal","value":{"text":"标题"}}});
    s["root"]["children"][2]["children"] = json!([
        {"type":"module","id":"nested","props":{"text":{"kind":"prop","key":"message","path":"/message"},"caption":{"kind":"prop","key":"caption","path":"/text"}},"children":[
            {"type":"text","id":"nestedText","text":{"kind":"prop","key":"text"}},
            {"type":"text","id":"nestedCaption","text":{"kind":"prop","key":"caption"}}
        ]}
    ]);
    let mut r = runtime(s);
    r.set_from("page", "settings", json!({"data":{"message":"changed"}}))
        .unwrap();
    let resolved = r.tree.resolve(&r.state).unwrap();
    let text = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "nestedText")
        .unwrap();
    let caption = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "nestedCaption")
        .unwrap();
    assert_eq!(resolved[text].text, "changed");
    assert_eq!(resolved[caption].text, "标题");
    assert!(r.set_from("nested", "text", json!("bad")).is_err());
}

#[test]
fn declared_payload_type_and_event_pointer_are_validated() {
    let mut s = source();
    s["root"]["children"][1]["emits"]["messageChanged"] = json!(42);
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("类型"));
    let mut s = source();
    s["root"]["children"][1]["actions"]["publish"]["steps"][0]["event"] = json!("unknown");
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("未声明"));
    let mut s = source();
    s["root"]["children"][1]["on"]["unknown"] = json!("receiveMessage");
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["actions"]["receiveMessage"]["steps"][0]["value"] =
        json!({"kind":"event","path":"/missing"});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][1]["on"]["messageChanged"] = json!("publish");
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][0]["text"] = json!({"kind":"event"});
    assert!(v02::parse(&s.to_string()).is_err());
}

#[test]
fn object_event_payload_updates_parent_and_dynamic_handler_can_switch() {
    let mut s = source();
    s["root"]["state"]["handler"] = json!("receiveMessage");
    s["root"]["children"][1]["state"]["draft"] = json!({"message":"object payload"});
    s["root"]["children"][1]["emits"]["messageChanged"] = json!({"message":""});
    s["root"]["children"][1]["children"][0]["children"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    s["root"]["actions"]["receiveMessage"]["steps"][0]["value"] =
        json!({"kind":"event","path":"/message"});
    s["root"]["actions"]["receiveOther"] = json!({"steps":[{"type":"concat","target":"sharedMessage","parts":[{"kind":"literal","value":"other:"},{"kind":"event","path":"/message"}]}]});
    s["root"]["actions"]["receiveOther"]["eventPayload"] = json!({"message":""});
    s["root"]["children"][1]["on"]["messageChanged"] = json!({"kind":"state","key":"handler"});
    let mut r = runtime(s);
    let click = |sequence| UiEvent {
        component_id: "publishButton".into(),
        target_key: None,
        kind: "click".into(),
        payload: Value::Null,
        sequence,
        generation: 1,
    };
    r.commit(vec![click(1)]);
    assert!(r.error.is_none());
    assert_eq!(
        r.state_from("page", "sharedMessage").unwrap(),
        "object payload"
    );
    r.set_from("page", "handler", json!("receiveOther"))
        .unwrap();
    r.commit(vec![click(2)]);
    assert!(r.error.is_none());
    assert_eq!(
        r.state_from("page", "sharedMessage").unwrap(),
        "other:object payload"
    );
    assert!(r.set_from("page", "handler", json!("missing")).is_err());
}

#[test]
fn unhandled_declared_event_is_allowed_and_direct_event_handler_call_fails() {
    let mut s = source();
    s["root"]["children"][1]["on"] = json!({});
    s["root"]["actions"] = json!({});
    let mut r = runtime(s);
    input(&mut r, "local only", 1);
    assert!(r.error.is_none());
    assert_eq!(
        r.state_from("page", "sharedMessage").unwrap(),
        "等待编辑模块发送内容"
    );
    let mut s = source();
    s["root"]["children"][1]["children"][0]["children"][1]["events"]["click"] =
        json!("receiveMessage");
    assert!(v02::parse(&s.to_string()).is_err());
}

#[test]
fn invalid_parent_update_cannot_break_prop_paths_or_event_contracts() {
    let mut s = source();
    s["root"]["state"]["items"] = json!([{"message":"first"}]);
    s["root"]["children"][2]["props"]["message"] =
        json!({"kind":"state","key":"items","path":"/0/message"});
    let mut r = runtime(s);
    let before = r.version;
    assert!(r.set_from("page", "items", json!([])).is_err());
    assert_eq!(r.version, before);
    assert_eq!(
        r.state_from("page", "items").unwrap(),
        &json!([{"message":"first"}])
    );
    let mut s = source();
    s["root"]["actions"]["receiveMessage"]["eventPayload"] = json!({"message":""});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][1]["props"]["job"] =
        json!({"kind":"literal","value":desktop_app_engine::command::initial_result()});
    s["root"]["children"][1]["actions"]["bad"] =
        json!({"steps":[{"type":"command","program":"example.exe","target":"job"}]});
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("只读"));
}

#[test]
fn events_can_be_forwarded_upward_with_bounded_dispatch_depth() {
    fn chain(levels: usize) -> Value {
        let mut child = json!({"type":"button","id":"start","text":{"kind":"literal","value":"start"},"events":{"click":"start"}});
        for i in (0..levels).rev() {
            child = json!({"type":"module","id":format!("module{i}"),"emits":{"changed":""},"on":{"changed":if i==0 {"finish"} else {"relay"}},
                "actions":{
                    "relay":{"eventPayload":"","steps":[{"type":"emit","event":"changed","payload":{"kind":"event"}}]},
                    "start":{"steps":[{"type":"emit","event":"changed","payload":{"kind":"literal","value":"forwarded"}}]}
                },"children":[child]});
        }
        json!({"schemaVersion":"0.2","app":{"name":"chain","window":{"width":640,"height":480}},"state":{"result":""},"actions":{"finish":{"steps":[{"type":"set","target":"result","value":{"kind":"event"}}]}},"root":child})
    }
    let click = || UiEvent {
        component_id: "start".into(),
        target_key: None,
        kind: "click".into(),
        payload: Value::Null,
        sequence: 1,
        generation: 1,
    };
    let mut r = runtime(chain(3));
    r.commit(vec![click()]);
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.state["result"], "forwarded");
    let mut r = runtime(chain(34));
    r.commit(vec![click()]);
    assert!(r.error.as_ref().unwrap().contains("32"));
    assert_eq!(r.state["result"], "");
}

#[test]
#[cfg(windows)]
fn event_payload_can_be_passed_to_a_background_command() {
    let mut s = source();
    s["root"]["state"]["job"] = desktop_app_engine::command::initial_result();
    s["root"]["actions"]["receiveMessage"] = json!({"steps":[{"type":"command","program":"powershell.exe","args":["-NoProfile","-NonInteractive","-File","command-demo.ps1",{"kind":"event"}],"target":"job","timeoutMs":10000}]});
    let mut r = runtime(s);
    r.command_base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    input(&mut r, "中文 空格 & $()", 1);
    assert!(r.error.is_none(), "{:?}", r.error);
    let start = std::time::Instant::now();
    while r.commands_running() {
        assert!(start.elapsed() < std::time::Duration::from_secs(15));
        r.poll_commands();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let job = r.state_from("page", "job").unwrap();
    assert_eq!(job["status"], "succeeded");
    assert_eq!(job["stdout"].as_str().unwrap().trim(), "中文 空格 & $()");
}

#[test]
fn same_prop_name_in_sibling_modules_can_bind_different_parent_sources() {
    let mut s = source();
    s["root"]["state"]["otherMessage"] = json!("editor only");
    s["root"]["children"][1]["props"] = json!({"message":{"kind":"state","key":"otherMessage"}});
    s["root"]["children"][1]["children"][0]["children"][2]["text"] =
        json!({"kind":"prop","key":"message"});
    let mut r = runtime(s);
    r.set_from("page", "sharedMessage", json!("preview only"))
        .unwrap();
    let resolved = r.tree.resolve(&r.state).unwrap();
    let editor = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "editorEcho")
        .unwrap();
    let preview = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "previewText")
        .unwrap();
    assert_eq!(resolved[editor].text, "editor only");
    assert_eq!(resolved[preview].text, "preview only");
}
