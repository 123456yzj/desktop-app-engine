//! Build-time application definition shared by both executable hosts.
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/app_definition.rs"));

pub fn packaged_definition(
    override_path: Option<&Path>,
) -> Result<Option<serde_json::Value>, String> {
    if IS_PACKAGED && override_path.is_some() {
        return Err("打包应用的定义已编入 EXE，不接受 --app；请修改源描述后重新打包".into());
    }
    Ok(definition())
}
