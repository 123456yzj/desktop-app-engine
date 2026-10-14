use crate::{EXAMPLE, description::Description, validation};
use std::path::Path;

#[derive(Debug)]
pub enum Document {
    V01(Description),
    V02(Box<crate::v02::Description>),
}
impl Document {
    pub fn app(&self) -> &crate::description::App {
        match self {
            Self::V01(d) => &d.app,
            Self::V02(d) => &d.app,
        }
    }
}
pub fn parse_document(source: &str) -> Result<Document, String> {
    let raw: serde_json::Value = serde_json::from_str(source).map_err(|e| format!("JSON: {e}"))?;
    match raw.get("schemaVersion").and_then(serde_json::Value::as_str) {
        Some("0.2") => crate::v02::parse(source).map(|d| Document::V02(Box::new(d))),
        // Keep the original parser's diagnostics and semantics for all other inputs.
        _ => parse(source).map(Document::V01),
    }
}
pub fn load_document(path: Option<&Path>) -> Result<Document, String> {
    let file = if let Some(path) = path {
        Some(path.to_owned())
    } else {
        let exe = std::env::current_exe().map_err(|e| format!("无法定位运行时程序: {e}"))?;
        let candidate = exe
            .parent()
            .ok_or("无法确定运行时所在目录")?
            .join("app.json");
        match std::fs::metadata(&candidate) {
            Ok(_) => Some(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(format!("外部描述 {}: {e}", candidate.display())),
        }
    };
    match file {
        Some(path) => {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("外部描述 {}: {e}", path.display()))?;
            let raw: serde_json::Value = serde_json::from_str(&source)
                .map_err(|e| format!("外部描述 {}: JSON: {e}", path.display()))?;
            if raw.get("schemaVersion").and_then(serde_json::Value::as_str) == Some("0.2") {
                crate::v02::parse_at(&source, path.parent().unwrap_or(Path::new(".")))
                    .map(|d| Document::V02(Box::new(d)))
            } else {
                parse_document(&source)
            }
            .map_err(|e| format!("外部描述 {}: {e}", path.display()))
        }
        None => parse_document(EXAMPLE),
    }
}

pub fn parse(source: &str) -> Result<Description, String> {
    let mut deserializer = serde_json::Deserializer::from_str(source);
    let description = serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|e| format!("描述字段 {}: {}", e.path(), e.inner()))?;
    deserializer.end().map_err(|e| format!("JSON 尾部: {e}"))?;
    validation::validate(&description)?;
    Ok(description)
}

pub fn load(path: Option<&Path>) -> Result<Description, String> {
    if let Some(path) = path {
        return load_file(path);
    }
    let executable = std::env::current_exe().map_err(|e| format!("无法定位运行时程序: {e}"))?;
    let directory = executable.parent().ok_or("无法确定运行时所在目录")?;
    load_from_directory(directory)
}

pub fn load_from_directory(directory: &Path) -> Result<Description, String> {
    let path = directory.join("app.json");
    match std::fs::metadata(&path) {
        Ok(_) => load_file(&path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => parse(EXAMPLE).map_err(|e| {
            format!(
                "未提供 --app，{} 不存在，嵌入示例不可用: {e}",
                path.display()
            )
        }),
        Err(error) => Err(format!("外部描述 {}: {error}", path.display())),
    }
}

fn load_file(path: &Path) -> Result<Description, String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("外部描述 {}: {e}", path.display()))?;
    parse(&source).map_err(|e| format!("外部描述 {}: {e}", path.display()))
}
