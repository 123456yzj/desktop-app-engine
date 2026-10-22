//! Embedded application payload shared by executable hosts.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) const MAGIC: &[u8; 8] = b"DAEPKG01";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Resource {
    pub path: String,
    pub offset: u64,
    pub length: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bundle {
    pub definition: serde_json::Value,
    pub payload_start: u64,
    pub resources: Vec<Resource>,
    pub resource_id: String,
}

fn bundle() -> Result<Option<&'static Bundle>, String> {
    static BUNDLE: OnceLock<Result<Option<Bundle>, String>> = OnceLock::new();
    BUNDLE
        .get_or_init(read_bundle)
        .as_ref()
        .map(Option::as_ref)
        .map_err(Clone::clone)
}

fn read_bundle() -> Result<Option<Bundle>, String> {
    let mut file = File::open(std::env::current_exe().map_err(|e| e.to_string())?)
        .map_err(|e| format!("无法读取应用 EXE：{e}"))?;
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    if length < 16 {
        return Ok(None);
    }
    file.seek(SeekFrom::End(-16)).map_err(|e| e.to_string())?;
    let mut footer = [0u8; 16];
    file.read_exact(&mut footer).map_err(|e| e.to_string())?;
    if &footer[8..] != MAGIC {
        return Ok(None);
    }
    let size = u64::from_le_bytes(footer[..8].try_into().unwrap());
    let start = length
        .checked_sub(16)
        .and_then(|end| end.checked_sub(size))
        .ok_or("应用 EXE 的内置定义长度无效")?;
    file.seek(SeekFrom::Start(start))
        .map_err(|e| e.to_string())?;
    let bundle: Bundle = serde_json::from_reader(file.take(size))
        .map_err(|e| format!("应用 EXE 的内置定义损坏：{e}"))?;
    if bundle.payload_start > start
        || bundle.resource_id.len() != 16
        || !bundle.resource_id.bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err("应用 EXE 的资源索引无效".into());
    }
    let mut paths = std::collections::BTreeSet::new();
    for resource in &bundle.resources {
        if !resource.path.starts_with("plugins/")
            || !Path::new(&resource.path)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            || !paths.insert(&resource.path)
            || resource.offset < bundle.payload_start
            || resource
                .offset
                .checked_add(resource.length)
                .is_none_or(|end| end > start)
        {
            return Err("应用 EXE 的资源路径或范围无效".into());
        }
    }
    Ok(Some(bundle))
}

pub fn is_packaged() -> Result<bool, String> {
    Ok(bundle()?.is_some())
}

pub fn packaged_definition(
    override_path: Option<&Path>,
) -> Result<Option<serde_json::Value>, String> {
    let Some(bundle) = bundle()? else {
        return Ok(None);
    };
    if override_path.is_some() {
        return Err("打包应用的定义已内置到 EXE，不接受 --app；请修改 JSON 后重新打包".into());
    }
    Ok(Some(bundle.definition.clone()))
}

/// Plugins are extracted to the user cache, so only one EXE needs to be delivered.
pub fn resource_base(exe_base: &Path) -> Result<PathBuf, String> {
    let Some(bundle) = bundle()? else {
        return Ok(exe_base.to_owned());
    };
    if bundle.resources.is_empty() {
        return Ok(exe_base.to_owned());
    }
    static BASE: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    BASE.get_or_init(|| extract_resources(bundle)).clone()
}

fn extract_resources(bundle: &Bundle) -> Result<PathBuf, String> {
    let local = std::env::var_os("LOCALAPPDATA").ok_or("无法定位用户资源缓存目录")?;
    let root = PathBuf::from(local).join("DesktopAppEngine/resources");
    let destination = root.join(&bundle.resource_id);
    if destination.join(".complete").is_file() {
        return Ok(destination);
    }
    fs::create_dir_all(&root).map_err(|e| format!("无法创建资源缓存：{e}"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let staging = root.join(format!(".extract-{}-{nonce}", std::process::id()));
    fs::create_dir(&staging).map_err(|e| format!("无法创建资源暂存目录：{e}"))?;
    let result = (|| {
        let mut source = File::open(std::env::current_exe().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for resource in &bundle.resources {
            let path = staging.join(&resource.path);
            fs::create_dir_all(path.parent().ok_or("无效的资源路径")?)
                .map_err(|e| e.to_string())?;
            source
                .seek(SeekFrom::Start(resource.offset))
                .map_err(|e| e.to_string())?;
            let mut file = File::create(&path).map_err(|e| e.to_string())?;
            let copied = std::io::copy(&mut (&mut source).take(resource.length), &mut file)
                .map_err(|e| format!("无法释放资源 {}：{e}", resource.path))?;
            if copied != resource.length {
                return Err(format!("内置资源不完整：{}", resource.path));
            }
        }
        fs::write(staging.join(".complete"), &bundle.resource_id).map_err(|e| e.to_string())?;
        match fs::rename(&staging, &destination) {
            Ok(()) => Ok(destination.clone()),
            Err(_) if destination.join(".complete").is_file() => Ok(destination.clone()),
            Err(e) => Err(format!("无法发布资源缓存：{e}")),
        }
    })();
    if staging.exists() {
        let _ = fs::remove_dir_all(staging);
    }
    result
}
