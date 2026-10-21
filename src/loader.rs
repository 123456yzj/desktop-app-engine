//! Select an input document and resolve its resources relative to its own directory.
use crate::{
    compiler::{self, CompiledDescription},
    protocol::{App, v01, v02},
    validation,
};
use std::path::Path;

#[derive(Debug)]
pub enum Document {
    V01(v01::Description),
    V02(Box<CompiledDescription>),
}
impl Document {
    pub fn app(&self) -> &App {
        match self {
            Self::V01(d) => &d.app,
            Self::V02(d) => d.app(),
        }
    }
}

pub fn parse_document(source: &str) -> Result<Document, String> {
    parse_document_at(source, &std::env::current_dir().map_err(|e| e.to_string())?)
}

pub fn parse_document_at(source: &str, base: &Path) -> Result<Document, String> {
    let source = source.trim_start_matches('\u{feff}');
    let raw: serde_json::Value = serde_json::from_str(source).map_err(|e| format!("JSON: {e}"))?;
    if raw.get("schemaVersion").and_then(serde_json::Value::as_str) == Some("0.2") {
        compiler::compile_at(v02::parse(source)?, base).map(|d| Document::V02(Box::new(d)))
    } else {
        let mut deserializer = serde_json::Deserializer::from_str(source);
        let description = serde_path_to_error::deserialize(&mut deserializer)
            .map_err(|e| format!("描述字段 {}: {}", e.path(), e.inner()))?;
        deserializer.end().map_err(|e| format!("JSON 尾部: {e}"))?;
        validation::validate(&description)?;
        Ok(Document::V01(description))
    }
}

pub fn load_document(path: Option<&Path>) -> Result<Document, String> {
    let exe = std::env::current_exe().map_err(|e| format!("无法定位运行时程序: {e}"))?;
    let base = exe.parent().ok_or("无法确定运行时所在目录")?;
    if let Some(value) = crate::application::packaged_definition(path)? {
        return parse_document_at(&value.to_string(), base);
    }
    let path = path
        .map(Path::to_owned)
        .unwrap_or_else(|| base.join("app.json"));
    let source = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!(
                "应用描述 {} 不存在：请使用 --app FILE.json，或在 EXE 同目录提供 app.json",
                path.display()
            )
        } else {
            format!("外部描述 {}: {e}", path.display())
        }
    })?;
    parse_document_at(&source, path.parent().unwrap_or(Path::new(".")))
        .map_err(|e| format!("外部描述 {}: {e}", path.display()))
}
