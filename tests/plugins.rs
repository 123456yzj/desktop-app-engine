use desktop_app_engine::{
    interaction::{Runtime, UiEvent},
    plugin::{self, Rpc},
    v02,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples")
}
fn source() -> Value {
    serde_json::from_str(include_str!("../examples/plugin-chart.json")).unwrap()
}
fn parse(s: &Value) -> Result<v02::Description, String> {
    v02::parse_at(&s.to_string(), &base())
}
fn wait(r: &mut Runtime, done: impl Fn(&Runtime) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !done(r) && r.error.is_none() && Instant::now() < deadline {
        r.poll_plugins();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(r.error.is_none(), "{:?}", r.error);
    assert!(done(r), "plugin completion timed out");
}
#[test]
fn installed_manifest_rejects_unknown_components_props_events_and_methods() {
    parse(&source()).unwrap();
    for (field, value) in [
        ("component", json!("echarts.missing")),
        ("props", json!({"unknown":{"kind":"literal","value":1}})),
        ("events", json!({"missing":"chartClick"})),
    ] {
        let mut s = source();
        s["root"]["children"][0][field] = value;
        assert!(parse(&s).is_err(), "{field}");
    }
    let mut s = source();
    s["actions"]["highlightBar"]["steps"][0]["method"] = json!("unknown");
    assert!(parse(&s).is_err());
    let mut s = source();
    s["actions"]["highlightBar"]["steps"][0]["args"] = json!({"kind":"literal","value":{}});
    assert!(parse(&s).is_err());
    let mut s = source();
    s["plugins"] = json!(["../escape"]);
    assert!(parse(&s).is_err());
}
#[test]
fn echarts_real_rpc_mount_update_resize_method_hit_test_and_dispose() {
    let registry = plugin::discover(&base(), &["echarts".into()]).unwrap();
    let mut rpc = Rpc::start(&registry["echarts.chart"]).unwrap();
    let option = source()["state"]["chartOption"].clone();
    let mount = rpc.request(&json!({"op":"mount","component":"chart","props":{"option":option},"width":800,"height":400})).unwrap();
    let svg = mount["svg"].as_str().unwrap();
    assert!(svg.contains("ECharts plugin"));
    let image = plugin::rasterize(svg).unwrap();
    assert_eq!(image.size, [800, 400]);
    assert!(image.rgba.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    let option_result = rpc
        .request(&json!({"op":"call","component":"chart","method":"getOption","args":{}}))
        .unwrap();
    assert_eq!(
        option_result["result"]["series"][0]["data"],
        json!([12, 24, 18])
    );
    let mut clicked = None;
    for x in (100..750).step_by(50) {
        for y in (100..340).step_by(40) {
            let event = rpc
                .request(
                    &json!({"op":"input","component":"chart","event":{"kind":"click","x":x,"y":y}}),
                )
                .unwrap();
            if let Some(v) = event["events"].as_array().and_then(|a| a.first()) {
                clicked = Some(v["payload"].clone());
                break;
            }
        }
        if clicked.is_some() {
            break;
        }
    }
    assert!(
        clicked.is_some(),
        "ECharts display-list hit test found no bar"
    );
    registry["echarts.chart"].definition.events["click"]
        .validate(&clicked.unwrap())
        .unwrap();
    let resized = rpc
        .request(&json!({"op":"resize","component":"chart","width":600,"height":300}))
        .unwrap();
    assert_eq!(
        plugin::rasterize(resized["svg"].as_str().unwrap())
            .unwrap()
            .size,
        [600, 300]
    );
    let mut option = source()["state"]["chartOption"].clone();
    option["series"][0]["data"] = json!([4, 8, 16]);
    rpc.request(&json!({"op":"update","component":"chart","props":{"option":option},"width":600,"height":300})).unwrap();
    let updated = rpc
        .request(&json!({"op":"call","component":"chart","method":"getOption","args":{}}))
        .unwrap();
    assert_eq!(updated["result"]["series"][0]["data"], json!([4, 8, 16]));
    rpc.request(&json!({"op":"unmount","component":"chart"}))
        .unwrap();
}
#[test]
fn runtime_background_render_reactive_update_method_result_and_event_payload() {
    let mut r = Runtime::new(parse(&source()).unwrap()).unwrap();
    r.sync_plugins(vec![("cpuChart".into(), 800, 400)]).unwrap();
    wait(&mut r, |r| r.plugin_images.contains_key("cpuChart"));
    let initial = r.plugin_images.remove("cpuChart").unwrap().rgba;
    let mut option = r.state["chartOption"].clone();
    option["series"][0]["data"] = json!([5, 10, 20]);
    r.set("chartOption", option).unwrap();
    r.sync_plugins(vec![("cpuChart".into(), 800, 400)]).unwrap();
    wait(&mut r, |r| r.plugin_images.contains_key("cpuChart"));
    assert_ne!(initial, r.plugin_images.remove("cpuChart").unwrap().rgba);
    r.invoke("highlightBar").unwrap();
    wait(&mut r, |r| r.state["dispatchResult"] == true);
    r.commit(vec![UiEvent {
        component_id: "cpuChart".into(),
        target_key: None,
        kind: "click".into(),
        payload: json!({"seriesIndex":0,"dataIndex":1,"name":"B","value":10}),
        sequence: 1,
        generation: 1,
    }]);
    assert_eq!(r.state["selected"], "B");
    assert!(r.error.is_none());
    let version = r.version;
    assert!(r.set("chartOption", json!(42)).is_err());
    assert_eq!(r.version, version);
    r.sync_plugins(vec![]).unwrap();
    assert!(r.invoke("highlightBar").is_err());
}
#[test]
fn plugin_events_validate_payload_and_respect_disabled_ancestors() {
    let mut s = source();
    s["root"]["enabled"] = json!({"kind":"literal","value":false});
    let mut r = Runtime::new(parse(&s).unwrap()).unwrap();
    r.commit(vec![UiEvent {
        component_id: "cpuChart".into(),
        target_key: None,
        kind: "click".into(),
        payload: json!({"seriesIndex":0,"dataIndex":0,"name":"A","value":12}),
        sequence: 1,
        generation: 1,
    }]);
    assert_eq!(r.state["selected"], "Click a bar");
    let mut r = Runtime::new(parse(&source()).unwrap()).unwrap();
    r.commit(vec![UiEvent {
        component_id: "cpuChart".into(),
        target_key: None,
        kind: "click".into(),
        payload: json!({"name":"wrong"}),
        sequence: 1,
        generation: 1,
    }]);
    assert!(r.error.is_some());
    assert_eq!(r.state["selected"], "Click a bar");
}

fn probe() -> Runtime {
    let s = json!({"schemaVersion":"0.2","plugins":["probe"],"app":{"name":"probe","window":{"width":400,"height":300}},"state":{"text":"start","result":false,"enabled":true},"root":{"type":"column","id":"root","enabled":{"kind":"state","key":"enabled"},"children":[{"type":"plugin","id":"probe","component":"probe.view","props":{"text":{"kind":"state","key":"text"}},"events":{"changed":"receive"}}]},"actions":{"receive":{"eventPayload":"","steps":[{"type":"set","target":"text","value":{"kind":"event"}}]},"bad":{"steps":[{"type":"pluginCall","target":"probe","method":"badResult","args":{"kind":"literal","value":{}},"resultTarget":"result"}]}}});
    Runtime::new(
        v02::parse_at(
            &s.to_string(),
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn generic_plugin_events_use_real_worker_and_bad_results_preserve_state() {
    let mut r = probe();
    r.sync_plugins(vec![("probe".into(), 100, 80)]).unwrap();
    wait(&mut r, |r| r.plugin_images.contains_key("probe"));
    r.plugins
        .send(
            "probe",
            json!({"op":"input","event":{"text":"worker event"}}),
            None,
            None,
        )
        .unwrap();
    wait(&mut r, |r| r.state["text"] == "worker event");
    r.invoke("bad").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while r.error.is_none() && Instant::now() < deadline {
        r.poll_plugins();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(r.error.is_some());
    assert_eq!(r.state["result"], false);
}
#[test]
fn late_worker_events_cannot_mutate_disabled_or_unmounted_instances() {
    let mut r = probe();
    r.sync_plugins(vec![("probe".into(), 100, 80)]).unwrap();
    wait(&mut r, |r| r.plugin_images.contains_key("probe"));
    r.plugins
        .send(
            "probe",
            json!({"op":"input","event":{"text":"discarded"}}),
            None,
            None,
        )
        .unwrap();
    r.set("enabled", json!(false)).unwrap();
    for _ in 0..30 {
        r.poll_plugins();
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(r.state["text"], "start");
    r.set("enabled", json!(true)).unwrap();
    r.plugins
        .send(
            "probe",
            json!({"op":"input","event":{"text":"old instance"}}),
            None,
            None,
        )
        .unwrap();
    r.sync_plugins(vec![]).unwrap();
    r.plugin_images.clear();
    r.sync_plugins(vec![("probe".into(), 120, 90)]).unwrap();
    wait(&mut r, |r| {
        r.plugin_images
            .get("probe")
            .is_some_and(|i| i.size == [120, 90])
    });
    assert_eq!(r.state["text"], "start");
}
#[test]
fn unresponsive_plugin_rpc_times_out() {
    let registry = plugin::discover(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        &["probe".into()],
    )
    .unwrap();
    let mut rpc = Rpc::start(&registry["probe.view"]).unwrap();
    assert!(
        rpc.request(&json!({"op":"call","method":"invalidReply","args":{}}))
            .unwrap_err()
            .contains("响应必须为对象")
    );
    let start = Instant::now();
    assert!(
        rpc.request(&json!({"op":"call","method":"stall","args":{}}))
            .unwrap_err()
            .contains("超时")
    );
    assert!(start.elapsed() >= Duration::from_secs(5) && start.elapsed() < Duration::from_secs(8));
}

#[test]
fn module_props_dynamic_plugin_events_and_local_method_results_share_lexical_scope() {
    let mut s = source();
    s["state"]["handler"] = json!("chartClick");
    let mut chart = s["root"]["children"][0].clone();
    chart["props"]["option"] = json!({"kind":"prop","key":"option"});
    chart["events"]["click"] = json!({"kind":"state","key":"handler"});
    let action = s["actions"]
        .as_object_mut()
        .unwrap()
        .remove("highlightBar")
        .unwrap();
    s["root"]["children"][0] = json!({"type":"module","id":"chartModule","props":{"option":{"kind":"state","key":"chartOption"}},"state":{"dispatchResult":false},"actions":{"highlightBar":action},"children":[chart]});
    // The external button invokes the global action, so keep it out of this scope test.
    s["root"]["children"].as_array_mut().unwrap().pop();
    let mut r = Runtime::new(parse(&s).unwrap()).unwrap();
    r.sync_plugins(vec![("cpuChart".into(), 800, 400)]).unwrap();
    wait(&mut r, |r| r.plugin_images.contains_key("cpuChart"));
    let action = r
        .tree
        .nodes
        .iter()
        .find(|n| n.spec.id == "chartModule")
        .unwrap()
        .spec
        .action_names["highlightBar"]
        .clone();
    r.invoke(&action).unwrap();
    wait(&mut r, |r| {
        r.state_from("cpuChart", "dispatchResult").unwrap() == true
    });
    assert_eq!(r.state["dispatchResult"], false);
    r.commit(vec![UiEvent {
        component_id: "cpuChart".into(),
        target_key: None,
        kind: "click".into(),
        payload: json!({"seriesIndex":0,"dataIndex":1,"name":"B","value":24}),
        sequence: 1,
        generation: 1,
    }]);
    assert_eq!(r.state["selected"], "B");
    assert!(r.error.is_none());
}
