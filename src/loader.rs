//! Select an input document and resolve its resources relative to its own directory.
use crate::{
    compiler::{self, CompiledDescription},
    protocol::v02,
};
use std::path::Path;
#[cfg(test)]
mod tests;

pub fn parse_document(source: &str) -> Result<CompiledDescription, String> {
    parse_document_at(source, &std::env::current_dir().map_err(|e| e.to_string())?)
}

pub fn parse_document_at(source: &str, base: &Path) -> Result<CompiledDescription, String> {
    compiler::compile_at(v02::parse(source)?, base)
}

pub fn load_document(path: Option<&Path>) -> Result<CompiledDescription, String> {
    let exe = std::env::current_exe().map_err(|e| format!("无法定位运行时程序: {e}"))?;
    let base = exe.parent().ok_or("无法确定运行时所在目录")?;
    if let Some(value) = crate::application::packaged_definition(path)? {
        return parse_document_at(
            &value.to_string(),
            &crate::application::resource_base(base)?,
        );
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
