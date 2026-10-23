use super::*;
use crate::v02;
use serde_json::json;

fn document(root: Value, state: Value) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"Widget contracts","window":{"width":640,"height":480}},"root":root,"state":state,"actions":{}})
}
fn runtime(value: Value) -> Runtime {
    Runtime::new(v02::parse(&value.to_string()).unwrap()).unwrap()
}
fn event(component: &str, kind: &str, payload: Value, sequence: u64) -> UiEvent {
    UiEvent {
        component_id: component.into(),
        target_key: None,
        kind: kind.into(),
        payload,
        sequence,
        generation: 1,
    }
}

#[test]
fn widget_primary_and_secondary_bindings_write_before_callbacks() {
    let mut d = document(
        json!({"type":"pagination","id":"pages","current":{"kind":"state","key":"page"},"pageSize":{"kind":"state","key":"size"},"total":200,"events":{"change":"capturePage","showSizeChange":"captureSize"}}),
        json!({"page":1,"size":10,"capturedPage":1,"capturedSize":10}),
    );
    d["actions"] = json!({"capturePage":{"steps":[{"type":"set","target":"capturedPage","value":{"kind":"state","key":"page"}}]},"captureSize":{"eventPayload":20,"steps":[{"type":"set","target":"capturedSize","value":{"kind":"event"}}]}});
    let mut r = runtime(d);
    r.commit(vec![
        event("pages", "change", json!(2), 1),
        event("pages", "showSizeChange", json!(20), 2),
    ]);
    assert_eq!(r.state()["capturedPage"], 2);
    assert_eq!(r.state()["size"], 20);
    assert_eq!(r.state()["capturedSize"], 20);
    for payload in [json!(0), json!("20"), json!(true)] {
        r.commit(vec![event(
            "pages",
            "showSizeChange",
            payload,
            r.last_sequence.unwrap() + 1,
        )]);
        assert!(r.error().is_some());
        assert_eq!(r.state()["size"], 20);
    }
}

#[test]
fn overlay_events_update_open_and_reject_wrong_payloads_without_actions() {
    for kind in [
        "modal",
        "drawer",
        "popover",
        "tooltip",
        "popconfirm",
        "dropdown",
        "tour",
        "message",
        "notification",
    ] {
        let mut r = runtime(document(
            json!({"type":kind,"id":"overlay","open":{"kind":"state","key":"open"}}),
            json!({"open":true}),
        ));
        r.commit(vec![event("overlay", "openChange", json!("false"), 1)]);
        assert_eq!(r.state()["open"], true);
        assert!(r.error().is_some());
        r.commit(vec![event("overlay", "openChange", json!(false), 2)]);
        assert_eq!(r.state()["open"], false);
        assert!(r.error().is_none());
    }
}

#[test]
fn select_and_checkbox_groups_validate_dynamic_options_and_updates_atomically() {
    for root in [
        json!({"type":"select","id":"control","mode":"multiple","value":{"kind":"state","key":"selected"},"options":{"kind":"state","key":"choices"}}),
        json!({"type":"checkbox","id":"control","value":{"kind":"state","key":"selected"},"options":{"kind":"state","key":"choices"}}),
    ] {
        let mut r = runtime(document(
            root,
            json!({"selected":[],"choices":["a","b"],"other":0}),
        ));
        r.commit(vec![event("control", "change", json!(["a"]), 1)]);
        assert_eq!(r.state()["selected"], json!(["a"]));
        for payload in [
            json!(["a", "a"]),
            json!(["missing"]),
            json!("a"),
            json!([1]),
        ] {
            r.commit(vec![event(
                "control",
                "change",
                payload,
                r.last_sequence.unwrap() + 1,
            )]);
            assert_eq!(r.state()["selected"], json!(["a"]));
            assert!(r.error().is_some());
        }
        assert!(
            r.set_many(BTreeMap::from([
                ("choices".into(), json!(["b"])),
                ("other".into(), json!(1))
            ]))
            .is_err()
        );
        assert_eq!(r.state()["other"], 0);
    }
}

#[test]
fn range_slider_and_required_array_fields_keep_typed_values() {
    let mut r = runtime(document(
        json!({"type":"slider","id":"range","range":true,"min":0,"max":100,"value":{"kind":"state","key":"range"}}),
        json!({"range":[20,80]}),
    ));
    for payload in [json!([80, 20]), json!([-1, 80]), json!([20]), json!(20)] {
        r.commit(vec![event(
            "range",
            "change",
            payload,
            r.last_sequence.unwrap_or(0) + 1,
        )]);
        assert_eq!(r.state()["range"], json!([20, 80]));
        assert!(r.error().is_some());
    }
    r.commit(vec![event("range", "change", json!([30, 70]), 100)]);
    assert_eq!(r.state()["range"], json!([30, 70]));
    let mut r = runtime(document(
        json!({"type":"form","id":"form","children":[{"type":"formItem","id":"item","rules":[{"required":true}],"children":[{"type":"select","id":"control","mode":"multiple","value":{"kind":"state","key":"selected"},"options":{"kind":"literal","value":["a"]}}]}]}),
        json!({"selected":[]}),
    ));
    r.commit(vec![event("form", "submit", Value::Null, 1)]);
    assert!(r.form_error("item").is_some());
    r.commit(vec![event("control", "change", json!(["a"]), 2)]);
    assert!(r.form_error("item").is_none());
}

#[test]
fn nested_configuration_disabling_and_readonly_reject_changes() {
    let mut d = document(
        json!({"type":"configProvider","id":"config","componentDisabled":{"kind":"state","key":"disabled"},"children":[{"type":"column","id":"inner","children":[{"type":"switch","id":"control","value":{"kind":"state","key":"value"}}]}]}),
        json!({"disabled":true,"value":false}),
    );
    let mut r = runtime(d.clone());
    assert!(!r.tree.resolve(r.state()).unwrap()[2].enabled);
    r.commit(vec![event("control", "change", json!(true), 1)]);
    assert_eq!(r.state()["value"], false);
    r.set("disabled", json!(false)).unwrap();
    r.commit(vec![event("control", "change", json!(true), 2)]);
    assert_eq!(r.state()["value"], true);
    d["root"] = json!({"type":"input","id":"control","readOnly":true,"value":{"kind":"state","key":"text"}});
    d["state"] = json!({"text":"original"});
    let mut r = runtime(d);
    r.commit(vec![event("control", "change", json!("changed"), 1)]);
    assert_eq!(r.state()["text"], "original");
}

#[test]
fn widget_contracts_reject_malformed_items_bindings_dates_and_properties() {
    for root in [
        json!({"type":"menu","id":"control","items":[{"key":"a"},{"key":"a"}]}),
        json!({"type":"menu","id":"control","items":[{"key":"a"}],"selectedKeys":["missing"]}),
        json!({"type":"pagination","id":"control","pageSize":0}),
        json!({"type":"pagination","id":"control","pageSizeOptions":[0,10]}),
        json!({"type":"modal","id":"control","open":{"kind":"state","key":"nested","path":"/open"}}),
        json!({"type":"datePicker","id":"control","value":"2026-02-30"}),
        json!({"type":"datePicker","id":"control","value":"2026-10-22","minDate":"2026-11-01"}),
        json!({"type":"cascader","id":"control","value":[],"options":[{"value":"a","children":[{"value":"a"}]}]}),
        json!({"type":"tabs","id":"control","items":[{"key":"a"}],"children":[{"type":"column","id":"b"}]}),
        json!({"type":"modal","id":"control","api":{}}),
        json!({"type":"rate","id":"control","value":2.5}),
        json!({"type":"rate","id":"control","value":2,"unknownProperty":true}),
    ] {
        let d = document(root.clone(), json!({"nested":{"open":false}}));
        assert!(
            v02::parse(&d.to_string()).and_then(Runtime::new).is_err(),
            "accepted {root}"
        );
    }
}

#[test]
fn table_page_and_tree_secondary_states_commit_before_callbacks_and_validate_membership() {
    let mut d = document(
        json!({"type":"table","id":"table","pageSize":2,"current":{"kind":"state","key":"page"},"rows":{"kind":"literal","value":[{"id":"a","name":"A"},{"id":"b","name":"B"},{"id":"c","name":"C"}]},"rowKey":"id","columns":[{"id":"name","field":"name","title":"Name","width":{"fr":1}}],"events":{"pageChange":"capture"}}),
        json!({"page":1,"captured":1}),
    );
    d["actions"] = json!({"capture":{"steps":[{"type":"set","target":"captured","value":{"kind":"state","key":"page"}}]}});
    let mut r = runtime(d);
    r.commit(vec![event("table", "pageChange", json!(2), 1)]);
    assert_eq!(r.state()["page"], 2);
    assert_eq!(r.state()["captured"], 2);
    r.commit(vec![event("table", "pageChange", json!(0), 2)]);
    assert!(r.error().is_some());
    assert_eq!(r.state()["page"], 2);
    let mut r = runtime(document(
        json!({"type":"tree","id":"tree","checkable":true,"treeData":{"kind":"state","key":"items"},"checkedKeys":{"kind":"state","key":"checked"}}),
        json!({"items":[{"key":"a","title":"A"}],"checked":[]}),
    ));
    r.commit(vec![event("tree", "check", json!(["missing"]), 1)]);
    assert!(r.error().is_some());
    assert_eq!(r.state()["checked"], json!([]));
    r.commit(vec![event("tree", "check", json!(["a"]), 2)]);
    assert!(r.error().is_none());
    assert_eq!(r.state()["checked"], json!(["a"]));
    assert!(r.set("items", json!([])).is_err());
    assert_eq!(r.state()["items"], json!([{"key":"a","title":"A"}]));
}

#[test]
fn schema_and_native_contracts_have_matching_extension_fields_and_event_names() {
    let schema: Value =
        serde_json::from_str(include_str!("../../docs/description-0.2.schema.json")).unwrap();
    let definitions = &schema["$defs"];
    for name in definitions["nativeWidget"]["properties"]["type"]["enum"]
        .as_array()
        .unwrap()
    {
        let kind: v02::Kind = serde_json::from_value(name.clone()).unwrap();
        let api = v02::util::component_api(kind);
        for property in api["properties"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|key| !["disabled", "size"].contains(&key.as_str()))
        {
            let found = definitions["componentApi"]["allOf"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| {
                    let condition = &entry["if"]["properties"]["type"];
                    (condition["const"] == *name
                        || condition["enum"]
                            .as_array()
                            .is_some_and(|a| a.contains(name)))
                        && entry["then"]["properties"].get(property).is_some()
                });
            assert!(found, "{name}.{property} missing from Schema");
        }
        let entry = definitions["nativeEvents"]["allOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| {
                let condition = &entry["if"]["properties"]["type"];
                condition["const"] == *name
                    || condition["enum"]
                        .as_array()
                        .is_some_and(|a| a.contains(name))
            })
            .unwrap_or_else(|| panic!("{name} events missing from Schema"));
        if api["events"].as_array().unwrap().is_empty() {
            assert_eq!(entry["then"]["$ref"], "#/$defs/noEvents");
        } else {
            assert_eq!(
                entry["then"]["properties"]["events"]["propertyNames"]["enum"], api["events"],
                "{name}"
            );
        }
        assert_eq!(api["container"], kind.is_container());
    }
}
