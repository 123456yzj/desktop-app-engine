//! Per-launch diagnostics and byte-exact command streams. Never log arguments or state.
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const PREVIEW_BYTES: usize = 32 * 1024;
static SERIAL: AtomicU64 = AtomicU64::new(1);
static GLOBAL: OnceLock<Result<Arc<Session>, String>> = OnceLock::new();

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn component(name: &str) -> String {
    let name: String = name
        .chars()
        .take(80)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("app-{name}")
}
pub fn global() -> Option<Arc<Session>> {
    GLOBAL
        .get_or_init(|| {
            let app = std::env::current_exe()
                .ok()
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "application".into());
            let base = std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            let root = base.join("DesktopAppEngine/logs").join(component(&app));
            match Session::new(&root) {
                Ok(session) => Ok(session),
                Err(primary) => {
                    let session = Session::new(
                        &std::env::temp_dir()
                            .join("DesktopAppEngine/logs")
                            .join(component(&app)),
                    )?;
                    session.event("WARN", "log.fallback", json!({"reason":primary}));
                    Ok(session)
                }
            }
        })
        .as_ref()
        .ok()
        .cloned()
}
pub fn init_error() -> Option<String> {
    let _ = global();
    GLOBAL.get().and_then(|r| r.as_ref().err()).cloned()
}
pub fn event(level: &str, name: &str, details: Value) {
    if let Some(session) = global() {
        session.event(level, name, details);
    }
}
pub fn open_viewer() {
    if let Some(session) = global() {
        session.viewer_open.store(true, Ordering::Relaxed);
    }
}

pub struct Session {
    pub directory: PathBuf,
    engine: Mutex<File>,
    tasks: Mutex<VecDeque<Arc<Task>>>,
    failure: Mutex<Option<String>>,
    events: Mutex<VecDeque<String>>,
    pub viewer_open: AtomicBool,
}
impl Session {
    pub fn new(root: &Path) -> Result<Arc<Self>, String> {
        fs::create_dir_all(root).map_err(|e| format!("创建日志目录失败: {e}"))?;
        let directory = root.join(format!(
            "session-{}-{}-{}",
            now_ms(),
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).map_err(|e| format!("创建运行日志目录失败: {e}"))?;
        let engine = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(directory.join("engine.jsonl"))
            .map_err(|e| format!("创建运行日志失败: {e}"))?;
        let session = Arc::new(Self {
            directory,
            engine: Mutex::new(engine),
            tasks: Mutex::new(VecDeque::new()),
            failure: Mutex::new(None),
            events: Mutex::new(VecDeque::new()),
            viewer_open: AtomicBool::new(false),
        });
        session.event(
            "INFO",
            "session.start",
            json!({"version":env!("CARGO_PKG_VERSION"),"pid":std::process::id()}),
        );
        Ok(session)
    }
    fn fail(&self, error: impl ToString) {
        *self.failure.lock().unwrap_or_else(|e| e.into_inner()) = Some(error.to_string());
    }
    pub fn error(&self) -> Option<String> {
        self.failure
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn event(&self, level: &str, name: &str, details: Value) {
        let record =
            json!({"timestampUnixMs":now_ms(),"level":level,"event":name,"details":details});
        {
            let mut events = self.events.lock().unwrap_or_else(|e| e.into_inner());
            events.push_back(record.to_string());
            while events.len() > 128 {
                events.pop_front();
            }
        }
        let result = (|| {
            let mut file = self.engine.lock().unwrap_or_else(|e| e.into_inner());
            serde_json::to_writer(&mut *file, &record)?;
            file.write_all(b"\n").map_err(serde_json::Error::io)
        })();
        if let Err(e) = result {
            self.fail(format!("写入运行日志失败: {e}"));
        }
    }
    pub fn tasks(&self) -> Vec<Arc<Task>> {
        self.tasks
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .cloned()
            .collect()
    }
    pub fn events(&self) -> String {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn task(
        self: &Arc<Self>,
        label: &str,
        program: &str,
        cwd: &Path,
        timeout_ms: u64,
    ) -> Result<Arc<Task>, String> {
        let id = format!(
            "task-{}-{}",
            now_ms(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        );
        let result = (|| {
            let directory = self.directory.join(&id);
            fs::create_dir(&directory).map_err(|e| e.to_string())?;
            let task = Arc::new(Task {
                id: id.clone(),
                label: label.into(),
                directory: directory.clone(),
                started_ms: now_ms(),
                started: Instant::now(),
                session: Arc::downgrade(self),
                stdout: Stream::new(&directory.join("stdout.log"))?,
                stderr: Stream::new(&directory.join("stderr.log"))?,
                result: Mutex::new(None),
            });
            let metadata = json!({"id":id,"label":label,"program":Path::new(program).file_name(),"cwd":cwd,"timeoutMs":timeout_ms,"startedUnixMs":task.started_ms});
            fs::write(
                directory.join("task.json"),
                serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let mut tasks = self.tasks.lock().unwrap_or_else(|e| e.into_inner());
            tasks.push_back(task.clone());
            while tasks.len() > 64 {
                if let Some(index) = tasks.iter().position(|t| t.finished().is_some()) {
                    tasks.remove(index);
                } else {
                    break;
                }
            }
            self.event("INFO", "command.start", metadata);
            Ok(task)
        })();
        if let Err(e) = &result {
            self.fail(format!("创建任务日志失败: {e}"));
        }
        result
    }
}
struct Stream {
    file: Mutex<File>,
    tail: Mutex<VecDeque<u8>>,
    bytes: AtomicU64,
}
impl Stream {
    fn new(path: &Path) -> Result<Self, String> {
        Ok(Self {
            file: Mutex::new(
                OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .map_err(|e| e.to_string())?,
            ),
            tail: Mutex::new(VecDeque::new()),
            bytes: AtomicU64::new(0),
        })
    }
    fn write(&self, bytes: &[u8]) -> std::io::Result<()> {
        self.bytes.fetch_add(bytes.len() as u64, Ordering::Relaxed);
        {
            let mut tail = self.tail.lock().unwrap_or_else(|e| e.into_inner());
            tail.extend(bytes);
            let excess = tail.len().saturating_sub(PREVIEW_BYTES);
            tail.drain(..excess);
        }
        self.file
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .write_all(bytes)
    }
    fn preview(&self) -> String {
        let bytes: Vec<u8> = self
            .tail
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .copied()
            .collect();
        // A rolling tail may begin in the middle of a UTF-8 code point.
        let start = bytes
            .iter()
            .position(|b| b & 0xc0 != 0x80)
            .unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[start..]).into_owned()
    }
}
pub struct Task {
    pub id: String,
    pub label: String,
    pub directory: PathBuf,
    pub started_ms: u64,
    started: Instant,
    session: Weak<Session>,
    stdout: Stream,
    stderr: Stream,
    result: Mutex<Option<Value>>,
}
impl Task {
    pub fn read_failed(&self, stderr: bool, error: &str) {
        if let Some(session) = self.session.upgrade() {
            session.fail(format!("读取任务输出失败: {error}"));
            session.event(
                "ERROR",
                "command.readFailed",
                json!({"id":self.id,"stream":if stderr {"stderr"} else {"stdout"},"error":error}),
            );
        }
    }
    pub fn write(&self, stderr: bool, bytes: &[u8]) {
        let result = if stderr { &self.stderr } else { &self.stdout }.write(bytes);
        if let Err(e) = result
            && let Some(session) = self.session.upgrade()
        {
            session.fail(format!("写入任务日志失败: {e}"));
        }
    }
    pub fn preview(&self, stderr: bool) -> String {
        if stderr { &self.stderr } else { &self.stdout }.preview()
    }
    pub fn byte_count(&self, stderr: bool) -> u64 {
        if stderr { &self.stderr } else { &self.stdout }
            .bytes
            .load(Ordering::Relaxed)
    }
    pub fn finished(&self) -> Option<Value> {
        self.result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn finish(&self, status: &str, code: i32, error: &str, output_complete: bool) {
        let record = json!({"id":self.id,"status":status,"exitCode":code,"error":error,"endedUnixMs":now_ms(),"durationMs":self.started.elapsed().as_millis() as u64,"stdoutBytes":self.byte_count(false),"stderrBytes":self.byte_count(true),"outputComplete":output_complete});
        *self.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(record.clone());
        if let Some(session) = self.session.upgrade() {
            if let Err(e) = fs::write(
                self.directory.join("result.json"),
                serde_json::to_vec_pretty(&record).unwrap_or_default(),
            ) {
                session.fail(format!("保存任务结果失败: {e}"));
            }
            session.event(
                if status == "succeeded" {
                    "INFO"
                } else {
                    "ERROR"
                },
                "command.finish",
                record,
            );
        }
    }
}

/// Called only by explicit user interaction. Pass the path as one OS argument.
pub fn open_directory(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err("日志目录不存在".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map_err(|e| format!("打开日志目录失败: {e}"))?;
    Ok(())
}
