//! Mutable connection settings are separate from the compiled application definition.
use super::backend::Connection;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

const MAGIC: &[u8] = b"DAE-CONNECTION-1\0";
const MAX_BYTES: usize = 32768;

pub fn user_path() -> Result<PathBuf, String> {
    let local = std::env::var_os("LOCALAPPDATA").ok_or("无法定位 LOCALAPPDATA")?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let name = exe.file_stem().ok_or("无法定位应用名称")?;
    Ok(PathBuf::from(local)
        .join("DesktopAppEngine/apps")
        .join(name)
        .join("connection.dat"))
}

pub fn load(path: &Path) -> Result<Option<Connection>, String> {
    use std::io::Read;
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("读取连接设置失败: {e}")),
    };
    let mut data = Vec::new();
    file.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > MAX_BYTES || !data.starts_with(MAGIC) {
        return Err("连接设置格式无效".into());
    }
    let mut data = &data[MAGIC.len()..];
    fn string(data: &mut &[u8]) -> Result<String, String> {
        let length = data.get(..4).ok_or("连接设置不完整")?;
        let length = u32::from_le_bytes(length.try_into().unwrap()) as usize;
        *data = &data[4..];
        let text = std::str::from_utf8(data.get(..length).ok_or("连接设置不完整")?)
            .map_err(|_| "连接设置编码无效")?
            .to_owned();
        *data = &data[length..];
        Ok(text)
    }
    let connection = Connection {
        host: string(&mut data)?,
        root: string(&mut data)?,
    };
    if !data.is_empty() {
        return Err("连接设置存在多余内容".into());
    }
    connection.validate()?;
    Ok(Some(connection))
}

pub fn save(path: &Path, connection: &Connection) -> Result<(), String> {
    connection.validate()?;
    let mut data = MAGIC.to_vec();
    for value in [&connection.host, &connection.root] {
        if value.len() > MAX_BYTES {
            return Err("连接设置过长".into());
        }
        data.extend_from_slice(&(value.len() as u32).to_le_bytes());
        data.extend_from_slice(value.as_bytes());
    }
    if data.len() > MAX_BYTES {
        return Err("连接设置过长".into());
    }
    let directory = path.parent().ok_or("连接设置路径无效")?;
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let pending = path.with_extension(format!(
        "{}.{}.pending",
        std::process::id(),
        SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&pending)
            .map_err(|e| e.to_string())?;
        file.write_all(&data)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        std::fs::rename(&pending, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&pending);
    }
    result
}
