//! The single export contract: JSON content in, one application EXE out.
use crate::{application, loader};
use serde_json::Value;
use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, File, OpenOptions},
    hash::{Hash, Hasher},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Validate JSON, embed it and installed plugins in the engine, then check the EXE.
/// The output path must not exist. No compiler, script or source tree is required.
pub fn package(json: &str, output: &Path) -> Result<PathBuf, String> {
    if application::is_packaged()? {
        return Err("请使用制作端引擎打包，应用 EXE 不提供打包入口".into());
    }
    if !output
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err("输出路径必须以 .exe 结尾".into());
    }
    let output = std::path::absolute(output).map_err(|e| e.to_string())?;
    match fs::symlink_metadata(&output) {
        Ok(_) => return Err(format!("输出路径已存在：{}", output.display())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("无法检查输出路径：{e}")),
    }
    let engine = std::env::current_exe().map_err(|e| e.to_string())?;
    let base = engine.parent().ok_or("无法定位引擎目录")?;
    // Installed engines use their own plugin catalog; cargo development uses the repo.
    let root = if base.join("plugins").is_dir() {
        base
    } else {
        let development = Path::new(env!("CARGO_MANIFEST_DIR"));
        if development.join("Cargo.toml").is_file() {
            development
        } else {
            base
        }
    };
    let json = json.trim_start_matches('\u{feff}');
    let document = loader::parse_document_at(json, root)?;
    let host = document
        .app()
        .host
        .as_deref()
        .unwrap_or("desktop-app-engine");
    let runtime = if host == "desktop-app-engine" {
        engine.clone()
    } else {
        let template = base.join(format!("{host}.exe"));
        if !template.is_file() {
            return Err(format!(
                "缺少应用声明的原生宿主 {host}：请在制作端引擎旁安装 {}",
                template.display()
            ));
        }
        template
    };
    let definition: Value = serde_json::from_str(json).map_err(|e| format!("JSON：{e}"))?;
    let mut resources = Vec::new();
    if let Some(plugins) = definition.get("plugins").and_then(Value::as_array)
        && !plugins.is_empty()
    {
        for plugin in plugins {
            let name = plugin.as_str().ok_or("插件名必须是字符串")?;
            collect_resources(root, &root.join("plugins").join(name), &mut resources)?;
        }
        if !root.join("plugins/runtime/node.exe").is_file() {
            return Err("引擎插件资源不完整：缺少 plugins/runtime/node.exe".into());
        }
        collect_resources(root, &root.join("plugins/runtime"), &mut resources)?;
    }
    resources.sort();
    resources.dedup();
    let parent = output.parent().ok_or("无效的输出路径")?;
    fs::create_dir_all(parent).map_err(|e| format!("无法创建输出目录：{e}"))?;
    let staging = Staging::new(parent)?;
    let executable = staging
        .0
        .join(output.file_name().ok_or("输出路径缺少文件名")?);
    fs::copy(runtime, &executable).map_err(|e| format!("无法生成应用 EXE：{e}"))?;
    embed(&executable, definition, resources)?;
    check(&executable, parent)?;
    publish(&executable, &output)?;
    Ok(output)
}

fn collect_resources(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let metadata = fs::symlink_metadata(dir)
        .map_err(|e| format!("无法读取插件资源 {}：{e}", dir.display()))?;
    if !metadata.file_type().is_dir() {
        return Err(format!("资源目录不支持符号链接：{}", dir.display()));
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            collect_resources(root, &path, files)?;
        } else if kind.is_file() {
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("插件资源路径必须是 UTF-8")?
                .replace('\\', "/");
            files.push((name, path));
        } else {
            return Err(format!("资源不支持符号链接或特殊文件：{}", path.display()));
        }
    }
    Ok(())
}

fn embed(
    executable: &Path,
    definition: Value,
    resources: Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(executable)
        .map_err(|e| e.to_string())?;
    // The producer is a console command; exported applications use the GUI subsystem.
    gui_subsystem(&mut file).map_err(|e| format!("无法生成 Windows 应用：{e}"))?;
    let payload_start = file.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    let mut hash = DefaultHasher::new();
    let mut buffer = [0u8; 64 * 1024];
    for (path, source) in resources {
        path.hash(&mut hash);
        let offset = file.stream_position().map_err(|e| e.to_string())?;
        let mut source = File::open(source).map_err(|e| e.to_string())?;
        loop {
            let count = source.read(&mut buffer).map_err(|e| e.to_string())?;
            if count == 0 {
                break;
            }
            hash.write(&buffer[..count]);
            file.write_all(&buffer[..count])
                .map_err(|e| e.to_string())?;
        }
        let length = file.stream_position().map_err(|e| e.to_string())? - offset;
        entries.push(application::Resource {
            path,
            offset,
            length,
        });
    }
    let bundle = application::Bundle {
        definition,
        payload_start,
        resources: entries,
        resource_id: format!("{:016x}", hash.finish()),
    };
    let metadata = serde_json::to_vec(&bundle).map_err(|e| e.to_string())?;
    file.write_all(&metadata)
        .and_then(|_| file.write_all(&(metadata.len() as u64).to_le_bytes()))
        .and_then(|_| file.write_all(application::MAGIC))
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("无法写入应用定义：{e}"))
}

fn gui_subsystem(file: &mut File) -> io::Result<()> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, "引擎不是有效的 PE 可执行文件");
    let mut dos = [0u8; 64];
    file.read_exact(&mut dos)?;
    if &dos[..2] != b"MZ" {
        return Err(invalid());
    }
    let pe = u32::from_le_bytes(dos[60..64].try_into().unwrap()) as u64;
    file.seek(SeekFrom::Start(pe))?;
    let mut header = [0u8; 26];
    file.read_exact(&mut header)?;
    if &header[..4] != b"PE\0\0"
        || u16::from_le_bytes(header[20..22].try_into().unwrap()) < 70
        || !matches!(
            u16::from_le_bytes(header[24..26].try_into().unwrap()),
            0x10b | 0x20b
        )
    {
        return Err(invalid());
    }
    file.seek(SeekFrom::Start(pe + 24 + 64))?;
    file.write_all(&0u32.to_le_bytes())?; // Clear the optional PE checksum after editing.
    file.write_all(&2u16.to_le_bytes()) // IMAGE_SUBSYSTEM_WINDOWS_GUI
}

struct Staging(PathBuf);

impl Staging {
    fn new(parent: &Path) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let path = parent.join(format!(".dae-package-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).map_err(|e| format!("无法创建打包暂存目录：{e}"))?;
        Ok(Self(path))
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check(executable: &Path, cwd: &Path) -> Result<(), String> {
    let parent = executable.parent().ok_or("无法定位暂存目录")?;
    let stdout = parent.join("check.out");
    let stderr = parent.join("check.err");
    let mut child = Command::new(executable)
        .arg("--check-plugins")
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(File::create(&stdout).map_err(|e| e.to_string())?)
        .stderr(File::create(&stderr).map_err(|e| e.to_string())?)
        .spawn()
        .map_err(|e| format!("无法启动应用校验：{e}"))?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            result => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(match result {
                    Err(e) => format!("无法等待应用校验：{e}"),
                    _ => "应用校验超过 30 秒".into(),
                });
            }
        }
    };
    let status = status?;
    if !status.success() {
        let stdout = fs::read(&stdout).unwrap_or_default();
        let stderr = fs::read(&stderr).unwrap_or_default();
        return Err(format!(
            "应用 EXE 校验失败（{status}）：\n{}{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr)
        ));
    }
    Ok(())
}

fn publish(source: &Path, output: &Path) -> Result<(), String> {
    let mut source = File::open(source).map_err(|e| e.to_string())?;
    let mut destination = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("无法创建输出 EXE（已有路径拒绝覆盖）：{e}"))?;
    let result = io::copy(&mut source, &mut destination).and_then(|_| destination.sync_all());
    drop(destination);
    if let Err(error) = result {
        let _ = fs::remove_file(output);
        return Err(format!("无法写入输出 EXE：{error}"));
    }
    Ok(())
}
