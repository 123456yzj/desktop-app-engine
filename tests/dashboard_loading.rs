use desktop_app_engine::{loader, v02};
use serde_json::{Value, json};
use std::{fs, process::Command};

#[test]
fn dashboard_cli_strict_errors_and_standalone_loading() {
    let source = include_str!("../examples/dashboard.json");
    let directory = std::env::temp_dir().join(format!("dashboard-loading-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let executable = directory.join("Dashboard.exe");
    fs::copy(env!("CARGO_BIN_EXE_desktop-app-engine"), &executable).unwrap();
    let check = || {
        Command::new(&executable)
            .arg("--check")
            .current_dir(directory.parent().unwrap())
            .output()
            .unwrap()
    };
    fs::write(directory.join("app.json"), source).unwrap();
    assert!(check().status.success());
    for (change, expected) in [
        (0, "unknown field"),
        (1, "value 必须为数字"),
        (2, "未定义状态"),
        (3, "null"),
    ] {
        let mut d: Value = serde_json::from_str(source).unwrap();
        match change {
            0 => d["root"]["span"] = json!(2),
            1 => {
                d["root"]["children"][2]["children"][0]["children"][0]["value"] =
                    json!({"kind":"literal","value":"42"})
            }
            2 => {
                d["root"]["children"][2]["children"][0]["children"][0]["value"]["key"] =
                    json!("missing")
            }
            _ => d["root"]["layout"]["width"] = json!(null),
        }
        fs::write(directory.join("app.json"), d.to_string()).unwrap();
        let output = check();
        assert_eq!(output.status.code(), Some(2));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("app.json"));
    }
    // Existing sibling failure must still lose to an explicit valid path.
    let explicit = directory.join("explicit.json");
    fs::write(&explicit, source).unwrap();
    assert!(
        Command::new(&executable)
            .args(["--app", explicit.to_str().unwrap(), "--check"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(matches!(
        loader::parse_document(source).unwrap(),
        loader::Document::V02(_)
    ));
    assert!(v02::parse(source).is_ok());
    fs::remove_dir_all(directory).unwrap();
}
