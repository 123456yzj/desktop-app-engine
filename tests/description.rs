use desktop_app_engine::{EXAMPLE, loader};
use serde_json::{Value, json};

fn invalid(change: impl FnOnce(&mut Value), expected: &str) {
    let mut description: Value = serde_json::from_str(EXAMPLE).unwrap();
    change(&mut description);
    let error = loader::parse(&description.to_string()).unwrap_err();
    assert!(
        error.contains(expected),
        "expected {expected:?}, got {error}"
    );
}

#[test]
fn valid_embedded_description() {
    assert_eq!(loader::parse(EXAMPLE).unwrap().schema_version, "0.1");
}

#[test]
fn rejects_versions_fields_types_and_missing_fields() {
    invalid(|v| v["schemaVersion"] = json!("2"), "schemaVersion");
    invalid(|v| v["extra"] = json!(true), "unknown field");
    invalid(
        |v| v["app"]["window"]["width"] = json!("640"),
        "app.window.width",
    );
    invalid(
        |v| {
            v["app"]
                .as_object_mut()
                .unwrap()
                .remove("name")
                .map(|_| ())
                .unwrap()
        },
        "app",
    );
    invalid(|v| v["root"]["type"] = json!("table"), "unknown variant");
    invalid(|v| v["root"]["bogus"] = json!(1), "unknown field");
    invalid(|v| v["state"]["name"] = json!(42), "state.name");
    invalid(
        |v| v["root"]["children"][0]["text"]["value"] = json!(false),
        "root",
    );
    invalid(
        |v| {
            v["root"]["children"][0]
                .as_object_mut()
                .unwrap()
                .remove("text")
                .map(|_| ())
                .unwrap()
        },
        "text",
    );
    invalid(
        |v| v["actions"]["greet"]["steps"][0]["extra"] = json!(true),
        "actions",
    );
    invalid(
        |v| v["root"]["children"][0]["text"]["extra"] = json!(true),
        "unknown field",
    );
}

#[test]
fn rejects_duplicate_ids_and_invalid_references_with_context() {
    invalid(
        |v| v["root"]["children"][0]["id"] = json!("page"),
        "重复组件 ID",
    );
    invalid(
        |v| v["root"]["children"][2]["text"]["key"] = json!("missing"),
        "live-name",
    );
    invalid(
        |v| v["root"]["children"][1]["children"][2]["onClick"] = json!("missing"),
        "confirm",
    );
    invalid(
        |v| v["actions"]["greet"]["steps"][0]["target"] = json!("missing"),
        "steps[0].target",
    );
    invalid(
        |v| v["actions"]["greet"]["steps"][0]["parts"][1]["key"] = json!("missing"),
        "parts[1]",
    );
    invalid(
        |v| v["app"]["window"]["width"] = json!(-1),
        "app.window.width",
    );
    invalid(|v| v["root"]["id"] = json!(""), ".id");
    invalid(|v| v["actions"]["greet"]["steps"] = json!([]), "steps");
}

#[test]
fn invalid_external_path_does_not_fall_back() {
    let error =
        loader::load(Some(std::path::Path::new("this-file-does-not-exist.json"))).unwrap_err();
    assert!(error.contains("this-file-does-not-exist.json"));
}

#[test]
fn malformed_and_trailing_json_are_rejected() {
    assert!(loader::parse("{").is_err());
    assert!(
        loader::parse(&format!("{EXAMPLE} {{}}"))
            .unwrap_err()
            .contains("尾部")
    );
}
