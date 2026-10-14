use desktop_app_engine::{
    command,
    interaction::{Runtime, UiEvent},
    v02,
};
use serde_json::{Value, json};

fn source() -> Value {
    serde_json::from_str(include_str!("../examples/scoped-state.json")).unwrap()
}
fn runtime(s: Value) -> Runtime {
    Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap()
}
fn click(r: &mut Runtime, id: &str, sequence: u64) {
    r.commit(vec![UiEvent {
        component_id: id.into(),
        target_key: None,
        kind: "click".into(),
        payload: Value::Null,
        sequence,
        generation: 1,
    }]);
    assert!(r.error.is_none(), "{:?}", r.error);
}

#[test]
fn sibling_states_actions_and_reactive_updates_are_isolated() {
    let mut r = runtime(source());
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
    let prior = r.version;
    click(&mut r, "buttonA", 1);
    assert_eq!(r.state_from("buttonA", "button").unwrap()["text"], "提交");
    assert_eq!(r.state_from("buttonB", "button").unwrap()["text"], "下一步");
    assert_eq!(r.tree.binding_version(b, &r.key_versions), prior);
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
        r.state_from("messageA", "message").unwrap(),
        "A：已提交（本地演示）"
    );
    assert_eq!(r.state_from("messageB", "message").unwrap(), "B：尚未更新");
    click(&mut r, "buttonB", 3);
    assert_eq!(
        r.state_from("buttonB", "button").unwrap()["action"],
        "submit"
    );
    assert!(
        !r.state_from("buttonA", "button").unwrap()["enabled"]
            .as_bool()
            .unwrap()
    );
}

#[test]
fn ancestor_inheritance_shadowing_and_action_definition_scope() {
    let mut s = source();
    s["state"]["message"] = json!("global");
    s["root"]["children"][1]["actions"]["parentWrite"] = json!({"steps":[{"type":"set","target":"message","value":{"kind":"literal","value":"parent changed"}}]});
    s["root"]["children"][1]["children"].as_array_mut().unwrap().push(json!({
        "type":"column","id":"nested","state":{"message":"nested","nullable":null},"children":[
            {"type":"text","id":"nestedMessage","text":{"kind":"state","key":"message"}},
            {"type":"text","id":"nestedHeading","text":{"kind":"state","key":"pageTitle"}},
            {"type":"button","id":"nestedCall","text":{"kind":"literal","value":"call parent"},"events":{"click":"parentWrite"}}
        ]
    }));
    let mut r = runtime(s);
    click(&mut r, "nestedCall", 1);
    assert_eq!(
        r.state_from("moduleA", "message").unwrap(),
        "parent changed"
    );
    assert_eq!(r.state_from("nested", "message").unwrap(), "nested");
    assert_eq!(r.state["message"], "global");
    assert_eq!(
        r.state_from("nestedHeading", "pageTitle").unwrap(),
        "两个模块使用同名状态，分别更新"
    );
    assert!(r.state_from("nested", "nullable").unwrap().is_null());
    r.set_from("nested", "message", json!("local changed"))
        .unwrap();
    assert_eq!(
        r.state_from("moduleA", "message").unwrap(),
        "parent changed"
    );
}

#[test]
fn parent_sibling_global_actions_and_forged_names_cannot_access_private_state() {
    let mut s = source();
    s["root"]["children"][0]["text"] = json!({"kind":"state","key":"message"});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][1]["state"]["private"] = json!("secret");
    s["root"]["children"][2]["children"][0]["text"] = json!({"kind":"state","key":"private"});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["actions"]["globalWrite"] = json!({"steps":[{"type":"set","target":"message","value":{"kind":"literal","value":"bad"}}]});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source();
    s["root"]["children"][0]["text"] = json!({"kind":"state","key":"@scope/7:moduleA/message"});
    assert!(v02::parse(&s.to_string()).unwrap_err().contains("保留前缀"));
    let mut r = runtime(source());
    assert!(r.state_from("page", "message").is_err());
    assert!(r.set_from("page", "message", json!("bad")).is_err());
}

#[test]
fn sibling_action_reference_and_invalid_local_update_are_rejected() {
    let mut s = source();
    s["root"]["children"][2]["actions"]["onlyB"] =
        json!({"steps":[{"type":"set","target":"message","value":{"kind":"literal","value":"B"}}]});
    s["root"]["children"][1]["state"]["button"]["action"] = json!("onlyB");
    assert!(v02::parse(&s.to_string()).is_err());
    let mut r = runtime(source());
    let before = r.state_from("buttonA", "button").unwrap().clone();
    let mut invalid = before.clone();
    invalid["action"] = json!("missing");
    let version = r.version;
    assert!(r.set_from("buttonA", "button", invalid).is_err());
    assert_eq!(r.state_from("buttonA", "button").unwrap(), &before);
    assert_eq!(r.version, version);
    let mut s = source();
    s["root"]["children"][0]["state"] = json!({"bad":1});
    assert!(v02::parse(&s.to_string()).is_err());
}

#[test]
fn local_input_updates_only_its_module() {
    let mut s = source();
    for (index, id) in [(1, "inputA"), (2, "inputB")] {
        s["root"]["children"][index]["children"]
            .as_array_mut()
            .unwrap()
            .push(json!({"type":"input","id":id,"value":{"kind":"state","key":"message"}}));
    }
    let mut r = runtime(s);
    r.commit(vec![UiEvent {
        component_id: "inputA".into(),
        target_key: None,
        kind: "change".into(),
        payload: json!("changed input"),
        sequence: 1,
        generation: 1,
    }]);
    assert!(r.error.is_none());
    assert_eq!(r.state_from("inputA", "message").unwrap(), "changed input");
    assert_eq!(r.state_from("inputB", "message").unwrap(), "B：尚未更新");
}

#[test]
fn dashboard_metrics_table_and_chart_bindings_work_in_page_scope() {
    let mut s: Value = serde_json::from_str(include_str!("../examples/dashboard.json")).unwrap();
    s["root"]["state"] = std::mem::replace(&mut s["state"], json!({}));
    s["root"]["actions"] = std::mem::replace(&mut s["actions"], json!({}));
    let mut r = runtime(s);
    click(&mut r, "refresh", 1);
    let resolved = r.tree.resolve(&r.state).unwrap();
    let cpu = r
        .tree
        .nodes
        .iter()
        .position(|n| n.spec.id == "cpu")
        .unwrap();
    assert_eq!(resolved[cpu].text, "57.0%");
    let table = &r
        .tree
        .nodes
        .iter()
        .find(|n| n.spec.id == "server-table")
        .unwrap()
        .spec;
    assert_eq!(
        table.rows.as_ref().unwrap().resolve(&r.state).unwrap(),
        r.state_from("server-table", "servers").unwrap()
    );
    let chart = &r
        .tree
        .nodes
        .iter()
        .find(|n| n.spec.id == "trend-chart")
        .unwrap()
        .spec;
    assert_eq!(
        chart.series.as_ref().unwrap()[0]
            .data
            .resolve(&r.state)
            .unwrap(),
        r.state_from("trend-chart", "trend").unwrap()
    );
    assert!(!r.state.contains_key("stats"));
}

#[test]
#[cfg(windows)]
fn command_arguments_and_results_with_same_names_are_module_local() {
    let mut s = source();
    for (index, button) in [(1, "buttonA"), (2, "buttonB")] {
        s["root"]["children"][index]["state"]["job"] = command::initial_result();
        s["root"]["children"][index]["actions"]["run"] = json!({"steps":[{"type":"command","program":"powershell.exe","args":["-NoProfile","-NonInteractive","-File","command-demo.ps1",{"kind":"state","key":"message"}],"target":"job","timeoutMs":10000}]});
        s["root"]["children"][index]["children"][1]["events"]["click"] = json!("run");
        assert_eq!(s["root"]["children"][index]["children"][1]["id"], button);
    }
    let mut r = runtime(s);
    r.command_base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    click(&mut r, "buttonA", 1);
    click(&mut r, "buttonB", 2);
    let start = std::time::Instant::now();
    while r.commands_running() {
        assert!(start.elapsed() < std::time::Duration::from_secs(15));
        r.poll_commands();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    for (component, expected) in [("moduleA", "A：尚未更新"), ("moduleB", "B：尚未更新")]
    {
        let job = r.state_from(component, "job").unwrap();
        assert_eq!(job["status"], "succeeded");
        assert_eq!(job["stdout"].as_str().unwrap().trim(), expected);
    }
}
