use super::*;
use crate::protocol;
use serde_json::json;

fn document(root: Value, state: Value) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"Inputs","window":{"width":600,"height":500}},"root":root,"state":state,"actions":{}})
}
fn runtime(document: Value) -> Runtime {
    Runtime::new(protocol::parse(&document.to_string()).unwrap()).unwrap()
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
fn boolean_changes_write_before_actions_and_reject_invalid_payloads() {
    for kind in ["checkbox", "switch"] {
        let mut d = document(
            json!({"type":kind,"id":"control","value":{"kind":"state","key":"value"},"events":{"change":"capture"}}),
            json!({"value":false,"captured":false}),
        );
        d["actions"] = json!({"capture":{"steps":[{"type":"set","target":"captured","value":{"kind":"state","key":"value"}}]}});
        let mut r = runtime(d);
        r.commit(vec![event("control", "change", json!(true), 1)]);
        assert_eq!(r.state()["captured"], json!(true));
        r.commit(vec![event("control", "change", json!("false"), 2)]);
        assert_eq!(r.state()["value"], json!(true));
        assert!(r.error().is_some());
    }
}

#[test]
fn radio_enforces_dynamic_options_and_disabled_choices() {
    let mut r = runtime(document(
        json!({"type":"radio","id":"control","value":{"kind":"state","key":"value"},"options":{"kind":"state","key":"options"}}),
        json!({"value":"a","options":[{"label":"A","value":"a","disabled":false},{"label":"B","value":"b","disabled":true}]}),
    ));
    r.commit(vec![event("control", "change", json!("b"), 1)]);
    assert_eq!(r.state()["value"], json!("a"));
    r.commit(vec![event("control", "change", json!("other"), 2)]);
    assert_eq!(r.state()["value"], json!("a"));
    r.set("options",json!([{"label":"A","value":"a","disabled":false},{"label":"B","value":"b","disabled":false}])).unwrap();
    r.commit(vec![event("control", "change", json!("b"), 3)]);
    assert_eq!(r.state()["value"], json!("b"));
    assert!(
        r.set(
            "options",
            json!([{"label":"A","value":"a","disabled":false}])
        )
        .is_err()
    );
}

#[test]
fn numeric_values_remain_typed_and_updates_are_atomic() {
    for kind in ["inputNumber", "slider"] {
        let mut r = runtime(document(
            json!({"type":kind,"id":"control","value":{"kind":"state","key":"value"},"min":-2,"max":5,"step":0.1}),
            json!({"value":0.0,"other":0}),
        ));
        for payload in [json!("1"), json!(true), json!(-3), json!(6)] {
            r.commit(vec![event(
                "control",
                "change",
                payload,
                r.last_sequence.unwrap_or(0) + 1,
            )]);
            assert_eq!(r.state()["value"], json!(0.0));
            assert!(r.error().is_some());
        }
        r.commit(vec![event("control", "change", json!(1.5), 10)]);
        assert_eq!(r.state()["value"], json!(1.5));
        assert!(
            r.set_many(BTreeMap::from([
                ("value".into(), json!(9)),
                ("other".into(), json!(1))
            ]))
            .is_err()
        );
        assert_eq!(r.state()["other"], json!(0));
    }
}

fn form() -> Value {
    let mut d = document(
        json!({"type":"form","id":"form","events":{"submit":"save","invalid":"invalid","reset":"reset"},"children":[
            {"type":"formItem","id":"nameItem","label":{"kind":"literal","value":"名称"},"rules":[{"required":true},{"minLength":2,"message":"至少两个字符"}],"children":[{"type":"input","id":"nameInput","value":{"kind":"state","key":"name"}}]},
            {"type":"formItem","id":"ageItem","rules":[{"min":18,"max":60}],"children":[{"type":"inputNumber","id":"ageInput","value":{"kind":"state","key":"age"},"min":0,"max":100}]},
            {"type":"formItem","id":"consentItem","rules":[{"required":true}],"children":[{"type":"checkbox","id":"consentInput","value":{"kind":"state","key":"consent"}}]},
            {"type":"column","id":"hiddenContainer","visible":{"kind":"state","key":"show"},"children":[{"type":"formItem","id":"hiddenItem","rules":[{"required":true}],"children":[{"type":"input","id":"hiddenInput","value":{"kind":"state","key":"hidden"}}]}]},
            {"type":"formItem","id":"disabledItem","enabled":{"kind":"literal","value":false},"rules":[{"required":true}],"children":[{"type":"input","id":"disabledInput","value":{"kind":"state","key":"disabled"}}]}
        ]}),
        json!({"name":"","age":10,"consent":false,"hidden":"","disabled":"","show":false,"status":"idle"}),
    );
    d["actions"] = json!({"save":{"steps":[{"type":"set","target":"status","value":{"kind":"state","key":"name"}}]},"invalid":{"steps":[{"type":"set","target":"status","value":{"kind":"literal","value":"invalid"}}]},"reset":{"steps":[{"type":"set","target":"status","value":{"kind":"state","key":"name"}}]}});
    d
}

#[test]
fn form_submit_validates_then_actions_read_latest_state_and_reset_restores_initials() {
    let mut r = runtime(form());
    assert!(r.form_error("nameItem").is_none());
    r.commit(vec![event("form", "submit", Value::Null, 1)]);
    assert_eq!(r.state()["status"], json!("invalid"));
    assert!(r.form_error("nameItem").is_some());
    assert!(r.form_error("ageItem").is_some());
    assert!(r.form_error("consentItem").is_some());
    assert!(r.form_error("hiddenItem").is_none());
    assert!(r.form_error("disabledItem").is_none());
    r.commit(vec![event("nameInput", "change", json!("中"), 2)]);
    assert_eq!(r.form_error("nameItem"), Some("至少两个字符"));
    r.commit(vec![
        event("nameInput", "change", json!("中文"), 3),
        event("ageInput", "change", json!(20), 4),
        event("consentInput", "change", json!(true), 5),
        event("form", "submit", Value::Null, 6),
    ]);
    assert_eq!(r.state()["status"], json!("中文"));
    assert!(r.form_errors.is_empty());
    r.set("show", json!(true)).unwrap();
    assert!(r.form_error("hiddenItem").is_some());
    r.commit(vec![event("form", "reset", Value::Null, 7)]);
    assert_eq!(r.state()["name"], json!(""));
    assert_eq!(r.state()["age"], json!(10));
    assert_eq!(r.state()["consent"], json!(false));
    assert_eq!(r.state()["status"], json!(""));
    assert!(r.form_errors.is_empty());
}

#[test]
fn forms_keep_local_state_and_reset_independent() {
    let field = |id: &str, value: i32| json!({"type":"form","id":id,"state":{"count":value},"children":[{"type":"formItem","id":format!("{id}Item"),"children":[{"type":"inputNumber","id":format!("{id}Input"),"value":{"kind":"state","key":"count"}}]}]});
    let mut r = runtime(document(
        json!({"type":"column","id":"root","children":[field("a",1),field("b",2)]}),
        json!({}),
    ));
    r.commit(vec![
        event("aInput", "change", json!(10), 1),
        event("bInput", "change", json!(20), 2),
        event("a", "reset", Value::Null, 3),
    ]);
    assert_eq!(r.state_from("aInput", "count").unwrap(), &json!(1));
    assert_eq!(r.state_from("bInput", "count").unwrap(), &json!(20));
}

#[test]
fn malformed_controls_and_forms_are_rejected() {
    let value = json!({"kind":"literal","value":1});
    let cases = [
        json!({"type":"checkbox","id":"x","value":value}),
        json!({"type":"switch","id":"x","value":value,"options":{"kind":"literal","value":[]}}),
        json!({"type":"inputNumber","id":"x","value":value,"step":0}),
        json!({"type":"inputNumber","id":"x","value":value,"precision":13}),
        json!({"type":"slider","id":"x","value":value,"min":2,"max":1}),
        json!({"type":"radio","id":"x","value":{"kind":"literal","value":"a"},"options":{"kind":"literal","value":["a","a"]}}),
        json!({"type":"slider","id":"x","value":{"kind":"state","key":"nested","path":"/n"}}),
        json!({"type":"form","id":"x","children":[{"type":"form","id":"nested"}]}),
        json!({"type":"formItem","id":"x","children":[]}),
        json!({"type":"button","id":"x","text":{"kind":"literal","value":"Save"},"htmlType":"submit"}),
        json!({"type":"form","id":"x","children":[{"type":"formItem","id":"item","children":[{"type":"inputNumber","id":"n","value":value}]}]}),
        json!({"type":"form","id":"x","children":[{"type":"formItem","id":"item","rules":[{"minLength":2}],"children":[{"type":"inputNumber","id":"n","value":{"kind":"state","key":"n"}}]}]}),
        json!({"type":"module","id":"x","props":{"n":{"kind":"literal","value":1}},"children":[{"type":"inputNumber","id":"n","value":{"kind":"prop","key":"n"}}]}),
    ];
    for root in cases {
        let d = document(root.clone(), json!({"nested":{"n":1},"n":1}));
        let result = protocol::parse(&d.to_string()).and_then(Runtime::new);
        assert!(result.is_err(), "unexpectedly accepted {root}");
    }
    let mut duplicate = form();
    duplicate["root"]["children"][1]["children"][0] =
        json!({"type":"input","id":"duplicate","value":{"kind":"state","key":"name"}});
    assert!(
        protocol::parse(&duplicate.to_string())
            .and_then(Runtime::new)
            .is_err()
    );
}

#[test]
fn numeric_normalization_uses_slider_minimum_and_precision() {
    let r = runtime(document(
        json!({"type":"slider","id":"slider","value":{"kind":"literal","value":0.15},"min":0.15,"max":1.0,"step":0.2}),
        json!({}),
    ));
    let n = &r.tree.nodes[0].spec;
    assert!((n.normalize_number(0.36) - 0.35).abs() < 1e-12);
    assert_eq!(n.normalize_number(100.0), 1.0);
    let r = runtime(document(
        json!({"type":"inputNumber","id":"n","value":{"kind":"literal","value":0.0},"min":0,"max":1,"step":0.01,"precision":2}),
        json!({}),
    ));
    assert_eq!(r.tree.nodes[0].spec.normalize_number(0.1234), 0.12);
}
