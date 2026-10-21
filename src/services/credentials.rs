//! Per-user Windows DPAPI storage. JSON/state contain references, never decrypted values.
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use zeroize::{Zeroize, Zeroizing};
const MAGIC: &[u8] = b"DAE-DPAPI-1\n";
pub const MAX_SECRET: usize = 8192;
pub fn valid_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    {
        return Err("凭据库与凭据名称必须为 1..64 位字母、数字、-、_".into());
    }
    let lower = name.to_ascii_lowercase();
    if [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ]
    .contains(&lower.as_str())
    {
        return Err("凭据名称不能使用 Windows 保留设备名".into());
    }
    Ok(())
}
#[derive(Debug, Clone)]
pub struct Store {
    directory: PathBuf,
    namespace: String,
}
impl Store {
    pub fn new(namespace: &str) -> Result<Self, String> {
        let root = std::env::var_os("LOCALAPPDATA").ok_or("无法定位当前用户 LOCALAPPDATA")?;
        Self::at(
            &PathBuf::from(root).join("DesktopAppEngine/credentials"),
            namespace,
        )
    }
    /// Host override, also used to isolate tests from the user's actual credential directory.
    pub fn at(root: &Path, namespace: &str) -> Result<Self, String> {
        valid_name(namespace)?;
        Ok(Self {
            directory: root.join(namespace.to_ascii_lowercase()),
            namespace: namespace.to_ascii_lowercase(),
        })
    }
    fn path(&self, name: &str) -> Result<PathBuf, String> {
        valid_name(name)?;
        Ok(self
            .directory
            .join(format!("{}.dpapi", name.to_ascii_lowercase())))
    }
    fn entropy(&self, name: &str) -> Vec<u8> {
        format!(
            "desktop-app-engine/credential/1/{}/{}",
            self.namespace,
            name.to_ascii_lowercase()
        )
        .into_bytes()
    }
    pub fn put(&self, name: &str, secret: &str) -> Result<(), String> {
        let path = self.path(name)?;
        if secret.is_empty() || secret.len() > MAX_SECRET || secret.contains(['\0', '\r', '\n']) {
            return Err("凭据必须为非空单行 UTF-8，最大 8192 字节".into());
        }
        let plain = Zeroizing::new(secret.as_bytes().to_vec());
        let encrypted = protect(&plain, &self.entropy(name), false)?;
        std::fs::create_dir_all(&self.directory).map_err(|_| "无法创建凭据目录")?;
        static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let temp = self.directory.join(format!(
            ".{}-{}.tmp",
            std::process::id(),
            SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .map_err(|_| "无法创建凭据密文文件")?;
            file.write_all(MAGIC)
                .and_then(|_| file.write_all(&encrypted))
                .and_then(|_| file.sync_all())
                .map_err(|_| "写入凭据密文失败")?;
            drop(file);
            replace(&temp, &path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temp);
        }
        result
    }
    pub fn get(&self, name: &str) -> Result<Zeroizing<String>, String> {
        let path = self.path(name)?;
        let file =
            std::fs::File::open(path).map_err(|_| format!("凭据 {name:?} 不存在或无法读取"))?;
        use std::io::Read;
        let mut bytes = Vec::new();
        file.take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| "读取凭据密文失败")?;
        if bytes.len() > 65536 || !bytes.starts_with(MAGIC) {
            return Err("凭据密文格式无效".into());
        }
        let plain = protect(&bytes[MAGIC.len()..], &self.entropy(name), true)?;
        let value = std::str::from_utf8(&plain).map_err(|_| "凭据内容不是 UTF-8")?;
        if value.is_empty() || value.len() > MAX_SECRET || value.contains(['\0', '\r', '\n']) {
            return Err("凭据内容无效".into());
        }
        Ok(Zeroizing::new(value.to_owned()))
    }
    pub fn delete(&self, name: &str) -> Result<(), String> {
        std::fs::remove_file(self.path(name)?).map_err(|_| "凭据不存在或删除失败".into())
    }
    pub fn names(&self) -> Result<Vec<String>, String> {
        let entries = match std::fs::read_dir(&self.directory) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err("无法读取凭据目录".into()),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| "无法读取凭据目录项")?;
            if entry.path().extension().is_some_and(|e| e == "dpapi")
                && entry.file_type().is_ok_and(|t| t.is_file())
                && let Some(name) = entry.path().file_stem().and_then(|s| s.to_str())
                && valid_name(name).is_ok()
            {
                names.push(name.to_owned());
            }
        }
        names.sort();
        Ok(names)
    }
}
#[cfg(windows)]
fn protect(data: &[u8], entropy: &[u8], decrypt: bool) -> Result<Zeroizing<Vec<u8>>, String> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    };
    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len().try_into().map_err(|_| "凭据数据超限")?,
        pbData: data.as_ptr().cast_mut(),
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_ptr().cast_mut(),
    };
    let mut out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    // DPAPI owns output allocation; all input slices stay live throughout the synchronous call.
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                &entropy,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        } else {
            CryptProtectData(
                &input,
                std::ptr::null(),
                &entropy,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        }
    };
    if ok == 0 {
        return Err(if decrypt {
            "凭据无法解密：用户不匹配、密文损坏或凭据绑定不匹配"
        } else {
            "Windows DPAPI 加密失败"
        }
        .into());
    }
    if out.pbData.is_null() {
        return Err("DPAPI 未返回数据".into());
    }
    // Copy into a zeroizing buffer, clear the DPAPI buffer, then release with LocalFree.
    let result = unsafe {
        let buffer = std::slice::from_raw_parts_mut(out.pbData, out.cbData as usize);
        let value = Zeroizing::new(buffer.to_vec());
        buffer.zeroize();
        LocalFree(out.pbData.cast());
        value
    };
    Ok(result)
}
#[cfg(not(windows))]
fn protect(_: &[u8], _: &[u8], _: bool) -> Result<Zeroizing<Vec<u8>>, String> {
    Err("凭据加密目前仅支持 Windows DPAPI".into())
}
#[cfg(windows)]
fn replace(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // NUL-terminated path buffers live until the synchronous replacement completes.
    if unsafe {
        MoveFileExW(
            from.as_ptr(),
            to.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        return Err("凭据密文替换失败".into());
    }
    Ok(())
}
#[cfg(not(windows))]
fn replace(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::rename(from, to).map_err(|_| "凭据密文替换失败".into())
}
