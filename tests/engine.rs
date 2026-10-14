use desktop_app_engine::{
    EXAMPLE, description::Property, engine::Engine, loader, validation::MAX_STRING_BYTES,
};
use serde_json::{Value, json};

fn engine() -> Engine {
    Engine::new(loader::parse(EXAMPLE).unwrap()).unwrap()
}
fn custom(steps: Value) -> Engine {
    let mut value: Value = serde_json::from_str(EXAMPLE).unwrap();
    value["actions"]["greet"]["steps"] = steps;
    Engine::new(loader::parse(&value.to_string()).unwrap()).unwrap()
}

#[test]
fn binding_and_repeated_click_use_latest_input() {
    let mut engine = engine();
    let property = Property::State { key: "name".into() };
    for name in ["小明", "小红"] {
        engine.set_state("name", name.into()).unwrap();
        assert_eq!(engine.resolve(&property).unwrap(), name);
        engine.trigger("greet").unwrap();
        engine.process_pending();
        assert_eq!(engine.state()["greeting"], format!("你好，{name}"));
    }
}

#[test]
fn captures_inputs_at_trigger_and_consumes_event_once() {
    let mut engine = engine();
    engine.set_state("name", "点击时".into()).unwrap();
    engine.trigger("greet").unwrap();
    engine.set_state("name", "点击后".into()).unwrap();
    engine.process_pending();
    assert_eq!(engine.state()["greeting"], "你好，点击时");
    engine.set_state("greeting", "保留".into()).unwrap();
    for _ in 0..20 {
        engine.process_pending();
    }
    assert_eq!(engine.state()["greeting"], "保留");
}

#[test]
fn sequential_writes_do_not_change_snapshot_reads() {
    let mut engine = custom(json!([
        {"type":"set","target":"name","value":{"kind":"literal","value":"覆盖"}},
        {"type":"set","target":"greeting","value":{"kind":"state","key":"name"}},
        {"type":"concat","target":"name","parts":[{"kind":"literal","value":"最后"}]}
    ]));
    engine.set_state("name", "原始".into()).unwrap();
    engine.trigger("greet").unwrap();
    engine.process_pending();
    assert_eq!(engine.state()["greeting"], "原始");
    assert_eq!(engine.state()["name"], "最后");
}

#[test]
fn failure_stops_later_steps_and_keeps_previous_success() {
    let mut engine = custom(json!([
        {"type":"set","target":"greeting","value":{"kind":"literal","value":"前一步"}},
        {"type":"concat","target":"name","parts":[{"kind":"state","key":"name"},{"kind":"state","key":"name"}]},
        {"type":"set","target":"greeting","value":{"kind":"literal","value":"不执行"}}
    ]));
    engine
        .set_state("name", "a".repeat(MAX_STRING_BYTES))
        .unwrap();
    engine.trigger("greet").unwrap();
    engine.process_pending();
    assert_eq!(engine.state()["greeting"], "前一步");
    assert!(engine.error().unwrap().contains("steps[1]"));
    engine.set_state("name", "短".into()).unwrap();
    engine.trigger("greet").unwrap();
    engine.process_pending();
    assert!(engine.error().is_none());
    assert_eq!(engine.state()["greeting"], "不执行");
}

#[test]
fn invalid_runtime_action_and_state_report_errors() {
    let mut engine = engine();
    assert!(engine.trigger("missing").unwrap_err().contains("missing"));
    assert!(engine.error().unwrap().contains("missing"));
    assert!(engine.set_state("missing", "x".into()).is_err());
    assert!(
        engine
            .set_state("name", "a".repeat(MAX_STRING_BYTES + 1))
            .is_err()
    );
}

#[test]
fn queued_invocations_keep_individual_snapshots() {
    let mut engine = engine();
    engine.set_state("name", "一".into()).unwrap();
    engine.trigger("greet").unwrap();
    engine.set_state("name", "二".into()).unwrap();
    engine.trigger("greet").unwrap();
    engine.set_state("name", "三".into()).unwrap();
    engine.process_pending();
    assert_eq!(engine.state()["greeting"], "你好，二");
}
