//! Installed JavaScript components communicate over bounded JSON-line RPC.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contract {
    pub kind: String,
    #[serde(default)]
    pub properties: BTreeMap<String, Contract>,
    #[serde(default)]
    pub required: Vec<String>,
    pub items: Option<Box<Contract>>,
    #[serde(default)]
    pub additional_properties: bool,
}
impl Contract {
    pub fn validate(&self, v: &Value) -> Result<(), String> {
        let valid = match self.kind.as_str() {
            "any" => true,
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "number" => v.is_number(),
            "boolean" => v.is_boolean(),
            "null" => v.is_null(),
            _ => return Err(format!("未知插件契约类型 {:?}", self.kind)),
        };
        if !valid {
            return Err(format!("插件值需要 {}", self.kind));
        }
        if let Some(o) = v.as_object() {
            if !self.additional_properties
                && self.kind == "object"
                && o.keys().any(|k| !self.properties.contains_key(k))
            {
                return Err("插件包含未声明属性".into());
            }
            for k in &self.required {
                if !o.contains_key(k) {
                    return Err(format!("插件缺少属性 {k}"));
                }
            }
            for (k, c) in &self.properties {
                if let Some(v) = o.get(k) {
                    c.validate(v).map_err(|e| format!("{k}: {e}"))?;
                }
            }
        }
        if let (Some(a), Some(c)) = (v.as_array(), &self.items) {
            for v in a {
                c.validate(v)?;
            }
        }
        Ok(())
    }
    fn check(&self) -> Result<(), String> {
        if ![
            "any", "object", "array", "string", "number", "boolean", "null",
        ]
        .contains(&self.kind.as_str())
        {
            return Err("非法契约 kind".into());
        }
        if self
            .required
            .iter()
            .any(|k| !self.properties.contains_key(k))
        {
            return Err("required 必须声明在 properties 中".into());
        }
        for c in self.properties.values() {
            c.check()?;
        }
        if let Some(c) = &self.items {
            c.check()?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    pub args: Contract,
    pub result: Contract,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub props: Contract,
    #[serde(default)]
    pub events: BTreeMap<String, Contract>,
    #[serde(default)]
    pub methods: BTreeMap<String, Method>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub api_version: u32,
    pub name: String,
    pub version: String,
    pub entry: String,
    pub components: BTreeMap<String, Component>,
}
#[derive(Debug, Clone)]
pub struct Registration {
    pub definition: Component,
    pub entry: PathBuf,
    pub node: PathBuf,
    pub component: String,
}
pub fn discover(base: &Path, names: &[String]) -> Result<BTreeMap<String, Registration>, String> {
    let mut registry = BTreeMap::new();
    if names.is_empty() {
        return Ok(registry);
    }
    let base = if base.as_os_str().is_empty() {
        Path::new(".")
    } else {
        base
    };
    let base = base
        .canonicalize()
        .map_err(|e| format!("无法定位插件目录: {e}"))?;
    for name in names {
        if name.is_empty()
            || !name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err("插件名仅接受字母数字、-、_".into());
        }
        let dir = base
            .join("plugins")
            .join(name)
            .canonicalize()
            .map_err(|e| format!("插件 {name}: {e}"))?;
        let manifest: Manifest = serde_json::from_str(
            &std::fs::read_to_string(dir.join("plugin.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if manifest.api_version != 1
            || manifest.name != *name
            || manifest.version.is_empty()
            || manifest.components.is_empty()
        {
            return Err(format!("插件 {name}: 无效 manifest"));
        }
        let entry = dir
            .join(&manifest.entry)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !entry.starts_with(&dir) || !entry.is_file() {
            return Err("插件 entry 必须位于包内".into());
        }
        let bundled = base.join("plugins/runtime/node.exe");
        let node = if bundled.is_file() {
            bundled
        } else {
            PathBuf::from("node")
        };
        for (component, definition) in manifest.components {
            definition.props.check()?;
            for c in definition.events.values() {
                c.check()?;
            }
            for m in definition.methods.values() {
                m.args.check()?;
                m.result.check()?;
            }
            let key = format!("{name}.{component}");
            if registry
                .insert(
                    key.clone(),
                    Registration {
                        definition,
                        entry: entry.clone(),
                        node: node.clone(),
                        component,
                    },
                )
                .is_some()
            {
                return Err(format!("重复插件组件 {key}"));
            }
        }
    }
    Ok(registry)
}

pub struct Rpc {
    child: Child,
    lines: mpsc::Receiver<Result<String, String>>,
    stderr: std::sync::Arc<std::sync::Mutex<String>>,
}
impl Rpc {
    pub fn start(r: &Registration) -> Result<Self, String> {
        let mut cmd = Command::new(&r.node);
        cmd.arg(r.entry.file_name().ok_or("entry 无文件名")?)
            .current_dir(r.entry.parent().ok_or("entry 无父目录")?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let mut child = cmd.spawn().map_err(|e| format!("插件宿主启动失败: {e}"))?;
        let stderr = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let errors = stderr.clone();
        let mut pipe = child.stderr.take().ok_or("缺少插件 stderr")?;
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buffer = [0; 1024];
            while let Ok(n) = pipe.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                if let Ok(mut text) = errors.lock() {
                    let remaining = 4096usize.saturating_sub(text.len());
                    if remaining > 0 {
                        text.push_str(&String::from_utf8_lossy(&buffer[..n.min(remaining)]));
                    }
                }
            }
        });
        let stdout = child.stdout.take().ok_or("缺少插件 stdout")?;
        let (tx, lines) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut bytes = Vec::new();
                use std::io::Read;
                let read = reader
                    .by_ref()
                    .take(8 * 1024 * 1024 + 1)
                    .read_until(b'\n', &mut bytes);
                match read {
                    Ok(0) => break,
                    Ok(n) if n <= 8 * 1024 * 1024 && bytes.last() == Some(&b'\n') => {
                        if tx
                            .send(String::from_utf8(bytes).map_err(|e| e.to_string()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    _ => {
                        let _ = tx.send(Err("插件输出超限或读取失败".into()));
                        break;
                    }
                }
            }
        });
        Ok(Self {
            child,
            lines,
            stderr,
        })
    }
    pub fn request(&mut self, request: &Value) -> Result<Value, String> {
        let mut bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err("插件请求超限".into());
        }
        bytes.push(b'\n');
        let mut stdin = self.child.stdin.take().ok_or("插件 stdin 已关闭")?;
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let result = stdin.write_all(&bytes).map_err(|e| e.to_string());
            let _ = tx.send((stdin, result));
        });
        let (stdin, result) = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| format!("插件写入超时: {e}"))?;
        self.child.stdin = Some(stdin);
        result?;
        let line = self
            .lines
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| {
                format!(
                    "插件响应超时/关闭: {e}; {}",
                    self.stderr.lock().map(|s| s.clone()).unwrap_or_default()
                )
            })??;
        let value: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if !value.is_object() {
            return Err("插件响应必须为对象".into());
        }
        if let Some(e) = value.get("error") {
            return Err(format!("插件: {e}"));
        }
        Ok(value)
    }
}
impl Drop for Rpc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Image {
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}
pub fn rasterize(svg: &str) -> Result<Image, String> {
    static OPTIONS: std::sync::OnceLock<resvg::usvg::Options<'static>> = std::sync::OnceLock::new();
    let options = OPTIONS.get_or_init(|| {
        let mut options = resvg::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        options
    });
    let tree = resvg::usvg::Tree::from_str(svg, options).map_err(|e| e.to_string())?;
    let size = tree.size().to_int_size();
    if size.width() > 4096 || size.height() > 4096 {
        return Err("插件 SVG 尺寸超过 4096".into());
    }
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(size.width(), size.height()).ok_or("插件图像分配失败")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    // tiny-skia pixels are premultiplied; egui's image constructor must match.
    Ok(Image {
        size: [size.width() as usize, size.height() as usize],
        rgba: pixmap.take(),
    })
}
pub struct Completion {
    pub id: String,
    pub target: Option<String>,
    pub method: Option<String>,
    pub response: Result<Value, String>,
    pub image: Option<Image>,
    epoch: u64,
}
struct Session {
    tx: mpsc::SyncSender<(Value, Option<String>, Option<String>)>,
    signature: Value,
    epoch: u64,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
#[derive(Default)]
pub struct Host {
    sessions: BTreeMap<String, Session>,
    pub completed: Vec<Completion>,
    rx: Option<mpsc::Receiver<Completion>>,
    tx: Option<mpsc::SyncSender<Completion>>,
    epoch: u64,
}
impl Host {
    pub fn sync(
        &mut self,
        id: &str,
        r: &Registration,
        props: Value,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let signature = json!([props, width, height]);
        if !self.sessions.contains_key(id) {
            if self.tx.is_none() {
                let (tx, rx) = mpsc::sync_channel(4);
                self.tx = Some(tx);
                self.rx = Some(rx);
            }
            let (tx, rx) = mpsc::sync_channel::<(Value, Option<String>, Option<String>)>(32);
            self.epoch += 1;
            let epoch = self.epoch;
            let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let worker_cancel = cancel.clone();
            let out = self.tx.as_ref().unwrap().clone();
            let key = id.to_owned();
            let registration = r.clone();
            std::thread::spawn(move || {
                let mut rpc = match Rpc::start(&registration) {
                    Ok(rpc) => rpc,
                    Err(e) => {
                        let _ = out.send(Completion {
                            id: key,
                            target: None,
                            method: None,
                            response: Err(e),
                            image: None,
                            epoch,
                        });
                        return;
                    }
                };
                for (mut request, target, method) in rx {
                    if worker_cancel.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    request["component"] = json!(registration.component);
                    let response = rpc.request(&request);
                    let mut image = None;
                    let response = response.and_then(|v| {
                        if let Some(svg) = v.get("svg").and_then(Value::as_str) {
                            image = Some(rasterize(svg)?);
                        }
                        Ok(v)
                    });
                    let failed = response.is_err();
                    if out
                        .send(Completion {
                            id: key.clone(),
                            target,
                            method,
                            response,
                            image,
                            epoch,
                        })
                        .is_err()
                        || failed
                    {
                        break;
                    }
                }
                let _ = rpc.request(&json!({"op":"unmount","component":registration.component}));
            });
            tx.try_send((
                json!({"op":"mount","props":props,"width":width,"height":height}),
                None,
                None,
            ))
            .map_err(|e| e.to_string())?;
            self.sessions.insert(
                id.to_owned(),
                Session {
                    tx,
                    signature,
                    epoch,
                    cancel,
                },
            );
        } else {
            let session = self.sessions.get_mut(id).unwrap();
            if session.signature != signature {
                let op = if session.signature[0] != signature[0] {
                    "update"
                } else {
                    "resize"
                };
                session
                    .tx
                    .try_send((
                        json!({"op":op,"props":props,"width":width,"height":height}),
                        None,
                        None,
                    ))
                    .map_err(|e| e.to_string())?;
                session.signature = signature;
            }
        }
        Ok(())
    }
    pub fn send(
        &self,
        id: &str,
        request: Value,
        target: Option<String>,
        method: Option<String>,
    ) -> Result<(), String> {
        self.sessions
            .get(id)
            .ok_or("插件组件尚未 mount")?
            .tx
            .try_send((request, target, method))
            .map_err(|e| e.to_string())
    }
    pub fn retain(&mut self, visible: &[String]) {
        self.sessions.retain(|id, _| visible.contains(id));
    }
    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(c) = rx.try_recv() {
                if self.sessions.get(&c.id).is_some_and(|s| s.epoch == c.epoch) {
                    self.completed.push(c);
                }
            }
        }
    }
}
