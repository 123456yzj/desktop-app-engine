use desktop_app_engine::{EXAMPLE, loader};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

/// Keep deterministic fixtures in the workspace for subsequent native GUI verification.
#[test]
fn external_descriptions_report_errors_and_generate_stress_fixture() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("evidence/fixtures");
    fs::create_dir_all(&directory).unwrap();
    let invalid = directory.join("invalid.json");
    fs::write(&invalid, r#"{"schemaVersion":"2"}"#).unwrap();
    let error = loader::load(Some(&invalid)).unwrap_err();
    assert!(error.contains("invalid.json"));
    assert!(error.contains("missing field"));

    let mut description: Value = serde_json::from_str(EXAMPLE).unwrap();
    description["app"]["name"] = json!("外部描述 · 长文本与失败验证");
    description["state"] = json!({"name":"", "result":"未执行", "chunk":"中文".repeat(6000)});
    description["root"] = json!({
        "type":"column", "id":"stress-page", "children":[
            {"type":"row","id":"stress-row","children":[
                {"type":"input","id":"different-input","value":{"kind":"state","key":"name"}},
                {"type":"button","id":"different-button","text":{"kind":"literal","value":"触发失败"},"onClick":"overflow"}
            ]},
            {"type":"text","id":"different-result","text":{"kind":"state","key":"result"}},
            {"type":"text","id":"long-text","text":{"kind":"literal","value":"这是用于验证窗口缩放、自动换行和垂直滚动的长中文文本。".repeat(100)}}
        ]
    });
    description["actions"] = json!({"overflow":{"steps":[
        {"type":"set","target":"result","value":{"kind":"literal","value":"前一步已执行"}},
        {"type":"concat","target":"name","parts":[{"kind":"state","key":"chunk"},{"kind":"state","key":"chunk"}]},
        {"type":"set","target":"result","value":{"kind":"literal","value":"不应执行"}}
    ]}});
    let path = directory.join("长文本与失败.json");
    fs::write(&path, serde_json::to_string_pretty(&description).unwrap()).unwrap();
    assert_eq!(
        loader::load(Some(&path)).unwrap().app.name,
        "外部描述 · 长文本与失败验证"
    );
}
