use super::*;
use crate::runtime::{Runtime, UiEvent};
use serde_json::{Value, json};

fn document() -> Value {
    json!({
        "schemaVersion":"0.2",
        "app":{"name":"Basic application","window":{"width":640,"height":480}},
        "state":{"name":"初始值","result":""},
        "root":{"type":"column","id":"page","children":[
            {"type":"row","id":"fields","layout":{"wrap":true},"children":[
                {"type":"input","id":"name","value":{"kind":"state","key":"name"},
                 "placeholder":{"kind":"literal","value":"请输入姓名"}},
                {"type":"button","id":"submit","text":{"kind":"literal","value":"提交"},
                 "events":{"click":"greet"}}
            ]},
            {"type":"text","id":"result","text":{"kind":"state","key":"result"}}
        ]},
        "actions":{"greet":{"steps":[
            {"type":"set","target":"name","value":{"kind":"literal","value":"动作改写"}},
            {"type":"concat","target":"result","parts":[
                {"kind":"literal","value":"你好，"},{"kind":"state","key":"name"}
            ]}
        ]}}
    })
}

fn event(id: &str, kind: &str, payload: Value, sequence: u64) -> UiEvent {
    UiEvent {
        component_id: id.into(),
        target_key: None,
        kind: kind.into(),
        payload,
        sequence,
        generation: 1,
    }
}

#[test]
fn basic_components_use_ordered_input_and_action_snapshots() {
    let source = format!("\u{feff}{}", document());
    let compiled = parse_document(&source).unwrap();
    assert_eq!(compiled.app().name, "Basic application");
    let mut runtime = Runtime::from_compiled(compiled).unwrap();
    runtime.commit(vec![
        event("submit", "click", Value::Null, 2),
        event("name", "change", json!("用户输入"), 1),
    ]);
    assert!(runtime.error().is_none(), "{:?}", runtime.error());
    assert_eq!(runtime.state()["name"], json!("动作改写"));
    assert_eq!(runtime.state()["result"], json!("你好，用户输入"));

    runtime.set("result", json!("保留结果")).unwrap();
    runtime.commit(vec![event("submit", "click", Value::Null, 2)]);
    runtime.commit(vec![]);
    assert_eq!(runtime.state()["result"], json!("保留结果"));
}

#[test]
fn unsupported_and_missing_versions_are_rejected_before_component_parsing() {
    for version in [json!("0.1"), json!("0.3"), json!(2), Value::Null] {
        let source = json!({"schemaVersion":version,"root":{
            "type":"button","id":"old","onClick":"action"
        }})
        .to_string();
        for error in [
            parse_document(&source).unwrap_err(),
            v02::parse(&source).unwrap_err(),
        ] {
            assert!(error.starts_with("schemaVersion:"), "{error}");
            assert!(error.contains("仅支持 \"0.2\""), "{error}");
        }
    }
    assert!(
        parse_document("{}")
            .unwrap_err()
            .starts_with("schemaVersion:")
    );
    let mut source = document();
    source["root"]["children"][0]["children"][1]["onClick"] = json!("greet");
    assert!(parse_document(&source.to_string()).is_err());
}

#[test]
fn failed_actions_preserve_prior_writes_and_stop_later_steps() {
    let mut source = document();
    source["actions"]["greet"]["steps"]
        .as_array_mut()
        .unwrap()
        .extend([
            json!({"type":"parseJson","target":"result","value":{"kind":"literal","value":"invalid JSON"}}),
            json!({"type":"set","target":"result","value":{"kind":"literal","value":"不应执行"}}),
        ]);
    let mut runtime = Runtime::from_compiled(parse_document(&source.to_string()).unwrap()).unwrap();
    let error = runtime.invoke("greet").unwrap_err();
    assert!(error.contains("actions.greet.steps[2]"), "{error}");
    assert_eq!(runtime.state()["name"], json!("动作改写"));
    assert_eq!(runtime.state()["result"], json!("你好，初始值"));
}

#[test]
fn string_limits_remain_in_force_after_protocol_cleanup() {
    let mut source = document();
    let limit = crate::validation::MAX_STRING_BYTES;
    source["state"]["largeText"] = json!(format!("{}x", "界".repeat(limit / 3)));
    let compiled = parse_document(&source.to_string()).unwrap();
    let mut runtime = Runtime::from_compiled(compiled).unwrap();
    let before = runtime.state()["largeText"].clone();
    let oversized = format!("{}x", before.as_str().unwrap());
    assert!(runtime.set("largeText", json!(oversized)).is_err());
    assert_eq!(runtime.state()["largeText"], before);
    source["state"]["largeText"] = json!(oversized);
    assert!(parse_document(&source.to_string()).is_err());
}
