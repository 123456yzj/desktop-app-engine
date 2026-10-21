//! Background child processes. Arguments are passed directly, without shell interpolation.
use crate::binding::{Binding, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Text(String),
    Binding(Binding),
}
impl Argument {
    pub fn resolve(&self, state: &State) -> Result<String, String> {
        self.resolve_event(state, None)
    }
    pub fn resolve_event(&self, state: &State, payload: Option<&Value>) -> Result<String, String> {
        let text = match self {
            Self::Text(s) => s.as_str(),
            Self::Binding(b) => b
                .resolve_event(state, payload)?
                .as_str()
                .ok_or("命令参数绑定必须为字符串")?,
        };
        if text.contains('\0') {
            return Err("命令参数不得包含 NUL".into());
        }
        Ok(text.to_owned())
    }
}
pub fn default_timeout() -> u64 {
    60_000
}
pub fn result(
    status: &str,
    running: bool,
    code: i32,
    stdout: String,
    stderr: String,
    error: String,
) -> Value {
    json!({"status":status,"running":running,"exitCode":code,"stdout":stdout,"stderr":stderr,"error":error})
}
pub fn initial_result() -> Value {
    result(
        "idle",
        false,
        -1,
        String::new(),
        String::new(),
        String::new(),
    )
}
pub fn validate(
    program: &str,
    args: &[Argument],
    cwd: &Option<String>,
    target: &str,
    timeout: u64,
    state: &State,
) -> Result<(), String> {
    validate_event(program, args, cwd, target, timeout, state, None)
}
#[allow(clippy::too_many_arguments)]
pub fn validate_event(
    program: &str,
    args: &[Argument],
    cwd: &Option<String>,
    target: &str,
    timeout: u64,
    state: &State,
    payload: Option<&Value>,
) -> Result<(), String> {
    if program.trim().is_empty()
        || program.contains('\0')
        || cwd
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.contains('\0'))
    {
        return Err("program/cwd 不能为空或包含 NUL".into());
    }
    if !(1..=3_600_000).contains(&timeout) {
        return Err("timeoutMs 必须在 1..3600000".into());
    }
    for arg in args {
        arg.resolve_event(state, payload)?;
    }
    let value = state
        .get(target)
        .ok_or_else(|| format!("未定义状态 {target:?}"))?;
    if crate::binding::ValueType::of(value)? != crate::binding::ValueType::of(&initial_result())? {
        return Err("命令 target 必须含 status/stdout/stderr/error 字符串、running 布尔和 exitCode 数字，且无额外字段".into());
    }
    Ok(())
}

type Capture = Arc<Mutex<Vec<u8>>>;
fn reader(
    mut pipe: impl Read + Send + 'static,
    task: Option<Arc<crate::logging::Task>>,
    stderr: bool,
) -> (Capture, mpsc::Receiver<bool>) {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let copy = bytes.clone();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut chunk = [0; 4096];
        let complete = loop {
            let n = match pipe.read(&mut chunk) {
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    if let Some(task) = &task {
                        task.read_failed(stderr, &e.to_string());
                    }
                    break false;
                }
            };
            if n == 0 {
                break true;
            }
            if let Some(task) = &task {
                task.write(stderr, &chunk[..n]);
            }
            let mut output = copy.lock().unwrap();
            let keep = n.min(16_384usize.saturating_sub(output.len()));
            output.extend_from_slice(&chunk[..keep]);
        };
        let _ = tx.send(complete);
    });
    (bytes, rx)
}
pub struct Request {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub timeout_ms: u64,
}
pub fn working_directory(base: &Path, cwd: &Option<String>) -> PathBuf {
    cwd.as_ref()
        .map_or_else(|| base.to_owned(), |p| base.join(p))
}
pub fn run(request: Request, cancel: Arc<AtomicBool>) -> Value {
    run_logged(request, cancel, "command", crate::logging::global())
}
pub fn run_logged(
    request: Request,
    cancel: Arc<AtomicBool>,
    label: &str,
    session: Option<Arc<crate::logging::Session>>,
) -> Value {
    let task = session.and_then(|s| {
        s.task(label, &request.program, &request.cwd, request.timeout_ms)
            .ok()
    });
    run_with_task(request, cancel, task)
}
/// Run against a task created by a native host, allowing immediate process-log selection.
pub fn run_with_task(
    request: Request,
    cancel: Arc<AtomicBool>,
    task: Option<Arc<crate::logging::Task>>,
) -> Value {
    let mut command = Command::new(&request.program);
    command
        .args(&request.args)
        .current_dir(&request.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            if let Some(task) = &task {
                task.finish("failed", -1, &e.to_string(), true);
            }
            return result(
                "failed",
                false,
                -1,
                String::new(),
                String::new(),
                e.to_string(),
            );
        }
    };
    let (out, out_done) = reader(child.stdout.take().unwrap(), task.clone(), false);
    let (err, err_done) = reader(child.stderr.take().unwrap(), task.clone(), true);
    let start = Instant::now();
    let (status, code, error) = loop {
        match child.try_wait() {
            Ok(Some(exit)) => {
                break (
                    if exit.success() {
                        "succeeded"
                    } else {
                        "failed"
                    },
                    exit.code().unwrap_or(-1),
                    String::new(),
                );
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                break ("failed", -1, e.to_string());
            }
            Ok(None) => {}
        }
        if cancel.load(Ordering::Relaxed)
            || start.elapsed() >= Duration::from_millis(request.timeout_ms)
        {
            let cancelled = cancel.load(Ordering::Relaxed);
            let _ = child.kill();
            let _ = child.wait();
            break (
                if cancelled { "cancelled" } else { "timedOut" },
                -1,
                if cancelled {
                    "应用已关闭"
                } else {
                    "命令执行超时"
                }
                .into(),
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Descendants may inherit pipes. Never wait indefinitely for their EOF.
    let out_complete = out_done
        .recv_timeout(Duration::from_millis(100))
        .unwrap_or(false);
    let err_complete = err_done
        .recv_timeout(Duration::from_millis(100))
        .unwrap_or(false);
    let stdout = String::from_utf8_lossy(&out.lock().unwrap()).into_owned();
    let stderr = String::from_utf8_lossy(&err.lock().unwrap()).into_owned();
    if let Some(task) = &task {
        task.finish(status, code, &error, out_complete && err_complete);
    }
    result(status, false, code, stdout, stderr, error)
}
