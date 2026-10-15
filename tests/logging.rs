use desktop_app_engine::{
    command::{self, Request},
    logging::Session,
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

static SERIAL: AtomicU64 = AtomicU64::new(1);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "engine-log-test-{}-{}-{}",
            std::process::id(),
            desktop_app_engine::logging::now_ms(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn request(script: &str) -> Request {
    Request {
        program: "powershell.exe".into(),
        args: vec![
            "-NoProfile".into(),
            "-NonInteractive".into(),
            "-Command".into(),
            script.into(),
        ],
        cwd: std::env::current_dir().unwrap(),
        timeout_ms: 10000,
    }
}
fn read_json(path: PathBuf) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
#[cfg(windows)]
fn full_binary_streams_are_saved_while_state_and_preview_stay_bounded() {
    let scratch = Scratch::new();
    let session = Session::new(&scratch.0).unwrap();
    let output = command::run_logged(
        request(
            "$secret='argument-marker-do-not-log'; [byte[]]$o=0..255; [byte[]]$e=255..0; $out=[Console]::OpenStandardOutput(); $err=[Console]::OpenStandardError(); for($i=0;$i -lt 800;$i++){ $out.Write($o,0,256); $err.Write($e,0,256) }",
        ),
        Arc::new(AtomicBool::new(false)),
        "binary",
        Some(session.clone()),
    );
    assert_eq!(output["status"], "succeeded");
    let task = session.tasks().pop().unwrap();
    assert_eq!(
        fs::read(task.directory.join("stdout.log")).unwrap(),
        (0..=255u8).cycle().take(204800).collect::<Vec<_>>()
    );
    assert_eq!(
        fs::read(task.directory.join("stderr.log")).unwrap(),
        (0..=255u8).rev().cycle().take(204800).collect::<Vec<_>>()
    );
    assert!(output["stdout"].as_str().unwrap().len() <= 16384 * 3);
    assert_eq!(task.byte_count(false), 204800);
    assert!(task.preview(false).len() <= 32768 * 3);
    let result = read_json(task.directory.join("result.json"));
    assert_eq!(result["stdoutBytes"], 204800);
    assert_eq!(result["stderrBytes"], 204800);
    assert_eq!(result["outputComplete"], true);
    let metadata = fs::read_to_string(task.directory.join("task.json")).unwrap();
    assert!(!metadata.contains("argument-marker-do-not-log"));
    assert!(
        !fs::read_to_string(session.directory.join("engine.jsonl"))
            .unwrap()
            .contains("argument-marker-do-not-log")
    );
    assert!(session.error().is_none());
}

#[test]
#[cfg(windows)]
fn running_process_output_is_visible_and_persisted_before_exit() {
    let scratch = Scratch::new();
    let session = Session::new(&scratch.0).unwrap();
    let gate = scratch.0.join("release");
    let script = format!(
        "[Console]::Out.Write('live-before-exit'); [Console]::Error.Write('live-error'); while(!(Test-Path -LiteralPath '{}')){{ Start-Sleep -Milliseconds 20 }}; exit 7",
        gate.display()
    );
    let copy = session.clone();
    let worker = std::thread::spawn(move || {
        command::run_logged(
            request(&script),
            Arc::new(AtomicBool::new(false)),
            "live",
            Some(copy),
        )
    });
    let start = Instant::now();
    let task = loop {
        if let Some(task) = session.tasks().last()
            && task.preview(false) == "live-before-exit"
            && task.preview(true) == "live-error"
        {
            break task.clone();
        }
        assert!(
            start.elapsed() < Duration::from_secs(8),
            "live output never arrived"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(!worker.is_finished());
    assert!(task.finished().is_none());
    assert!(!task.directory.join("result.json").exists());
    assert_eq!(
        fs::read(task.directory.join("stdout.log")).unwrap(),
        b"live-before-exit"
    );
    fs::write(gate, b"go").unwrap();
    assert_eq!(worker.join().unwrap()["exitCode"], 7);
    assert_eq!(
        read_json(task.directory.join("result.json"))["status"],
        "failed"
    );
}

#[test]
#[cfg(windows)]
fn launch_failure_timeout_and_cancel_have_persistent_results() {
    let scratch = Scratch::new();
    let session = Session::new(&scratch.0).unwrap();
    for status in ["failed", "timedOut", "cancelled"] {
        let mut command = request("Start-Sleep -Seconds 30");
        if status == "failed" {
            command.program = "missing-log-test-command-482491.exe".into();
        }
        if status == "timedOut" {
            command.timeout_ms = 50;
        }
        let result = command::run_logged(
            command,
            Arc::new(AtomicBool::new(status == "cancelled")),
            status,
            Some(session.clone()),
        );
        assert_eq!(result["status"], status);
        let task = session.tasks().pop().unwrap();
        let saved = read_json(task.directory.join("result.json"));
        assert_eq!(saved["status"], status);
        assert_eq!(saved["exitCode"], -1);
        assert!(!saved["error"].as_str().unwrap().is_empty());
    }
}

#[test]
#[cfg(windows)]
fn sessions_preserve_history_and_log_failure_does_not_block_commands() {
    let scratch = Scratch::new();
    let first = Session::new(&scratch.0).unwrap();
    first.event(
        "INFO",
        "test.history",
        serde_json::json!({"marker":"retained"}),
    );
    let previous = first.directory.clone();
    drop(first);
    let mut second = Session::new(&scratch.0).unwrap();
    assert_ne!(previous, second.directory);
    assert!(
        fs::read_to_string(previous.join("engine.jsonl"))
            .unwrap()
            .contains("retained")
    );
    let blocked = scratch.0.join("blocked");
    fs::write(&blocked, b"not a directory").unwrap();
    assert!(Session::new(&blocked).is_err());
    Arc::get_mut(&mut second).unwrap().directory = blocked;
    let result = command::run_logged(
        request("[Console]::Out.Write('still works')"),
        Arc::new(AtomicBool::new(false)),
        "unwritable",
        Some(second.clone()),
    );
    assert_eq!(result["status"], "succeeded");
    assert_eq!(result["stdout"], "still works");
    assert!(second.error().unwrap().contains("创建任务日志失败"));
}
