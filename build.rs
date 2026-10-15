use serde_json::Value;
use std::{fmt::Write, path::PathBuf};

// Compile the input into Rust value constructors; no external description is shipped.
fn expression(value: &Value, code: &mut String) {
    match value {
        Value::Null => code.push_str("serde_json::Value::Null"),
        Value::Bool(v) => write!(code, "serde_json::Value::Bool({v})").unwrap(),
        Value::Number(v) => {
            code.push_str("serde_json::Value::Number(");
            if let Some(n) = v.as_u64() {
                write!(code, "serde_json::Number::from({n}u64)").unwrap();
            } else if let Some(n) = v.as_i64() {
                write!(code, "serde_json::Number::from({n}i64)").unwrap();
            } else {
                write!(
                    code,
                    "serde_json::Number::from_f64({:?}f64).unwrap()",
                    v.as_f64().unwrap()
                )
                .unwrap();
            }
            code.push(')');
        }
        Value::String(v) => write!(code, "serde_json::Value::String({v:?}.to_owned())").unwrap(),
        Value::Array(values) => {
            code.push_str("serde_json::Value::Array(vec![");
            for value in values {
                expression(value, code);
                code.push(',');
            }
            code.push_str("])");
        }
        Value::Object(values) => {
            code.push_str("serde_json::Value::Object(serde_json::Map::from_iter([");
            for (key, value) in values {
                write!(code, "({key:?}.to_owned(),").unwrap();
                expression(value, code);
                code.push_str("),");
            }
            code.push_str("]))");
        }
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=DAE_APP_DESCRIPTION");
    let mut code = String::new();
    if let Some(path) = std::env::var_os("DAE_APP_DESCRIPTION") {
        let path = std::fs::canonicalize(path).expect("无法定位打包输入描述");
        println!("cargo:rerun-if-changed={}", path.display());
        let source = std::fs::read_to_string(&path).expect("无法读取打包输入描述");
        let value: Value = serde_json::from_str(source.trim_start_matches('\u{feff}'))
            .expect("打包输入不是有效 JSON");
        code.push_str("pub const IS_PACKAGED: bool = true;\npub fn definition() -> Option<serde_json::Value> { Some(");
        expression(&value, &mut code);
        code.push_str(") }\n");
    } else {
        code.push_str("pub const IS_PACKAGED: bool = false;\npub fn definition() -> Option<serde_json::Value> { None }\n");
    }
    std::fs::write(
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("app_definition.rs"),
        code,
    )
    .unwrap();
}
