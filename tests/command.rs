use desktop_app_engine::{
    command,
    interaction::{Runtime, UiEvent},
    v02,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn source(script: &str) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"command test","window":{"width":640,"height":480}},
        "state":{"job":command::initial_result(),"argument":"原始值"},
        "root":{"type":"button","id":"run","text":{"kind":"literal","value":"run"},"events":{"click":"run"}},
        "actions":{"run":{"steps":[{"type":"command","program":"powershell.exe",
            "args":["-NoProfile","-NonInteractive","-Command",script],"target":"job","timeoutMs":10000}]}}})
}
fn runtime(source: Value) -> Runtime {
    Runtime::new(v02::parse(&source.to_string()).unwrap()).unwrap()
}
fn wait(r: &mut Runtime) {
    let start = Instant::now();
    while r.commands_running() {
        assert!(
            start.elapsed() < Duration::from_secs(15),
            "worker did not finish"
        );
        r.poll_commands();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn command_contract_rejects_bad_fields_types_and_references() {
    for (field, value) in [
        ("program", json!("")),
        ("timeoutMs", json!(0)),
        ("args", json!([1])),
        ("cwd", json!("")),
        ("target", json!("missing")),
        ("unknown", json!(true)),
    ] {
        let mut s = source("exit 0");
        s["actions"]["run"]["steps"][0][field] = value;
        assert!(v02::parse(&s.to_string()).is_err(), "{field}");
    }
    let mut s = source("exit 0");
    s["state"]["job"]["exitCode"] = json!(null);
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = source("exit 0");
    s["actions"]["run"]["steps"][0]["args"] = json!([{"kind":"state","key":"missing"}]);
    assert!(v02::parse(&s.to_string()).is_err());
}

#[test]
#[cfg(windows)]
fn click_starts_background_command_captures_both_streams_and_exit_code() {
    let mut r = runtime(source(
        "[Console]::Out.Write('hello'); [Console]::Error.Write('problem'); exit 7",
    ));
    r.commit(vec![UiEvent {
        component_id: "run".into(),
        target_key: None,
        kind: "click".into(),
        payload: Value::Null,
        sequence: 1,
        generation: 1,
    }]);
    assert_eq!(r.state["job"]["status"], "running");
    assert!(r.state["job"]["running"].as_bool().unwrap());
    assert!(r.invoke("run").unwrap_err().contains("已在运行"));
    wait(&mut r);
    assert_eq!(r.state["job"]["status"], "failed");
    assert_eq!(r.state["job"]["exitCode"], 7);
    assert_eq!(r.state["job"]["stdout"], "hello");
    assert_eq!(r.state["job"]["stderr"], "problem");
    assert_eq!(r.state["job"]["running"], false);
    r.invoke("run").unwrap();
    wait(&mut r);
    assert_eq!(r.state["job"]["exitCode"], 7);
}

#[test]
#[cfg(windows)]
fn arguments_use_click_snapshot_and_description_directory() {
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut s = source("");
    s["actions"]["run"]["steps"][0]["args"] = json!(["-NoProfile","-NonInteractive","-File","command-demo.ps1",{"kind":"state","key":"argument"}]);
    s["state"]["argument"] = json!("空 格 & ; $() \"中文\"");
    let mut r = runtime(s);
    r.command_base = base;
    r.invoke("run").unwrap();
    r.set("argument", json!("已修改")).unwrap();
    wait(&mut r);
    assert_eq!(r.state["job"]["status"], "succeeded");
    assert_eq!(
        r.state["job"]["stdout"].as_str().unwrap().trim(),
        "空 格 & ; $() \"中文\""
    );
}

#[test]
#[cfg(windows)]
fn timeout_launch_failure_and_large_output_complete_without_blocking() {
    let mut s = source("Start-Sleep -Seconds 30");
    s["actions"]["run"]["steps"][0]["timeoutMs"] = json!(100);
    let mut r = runtime(s);
    r.invoke("run").unwrap();
    wait(&mut r);
    assert_eq!(r.state["job"]["status"], "timedOut");
    assert_eq!(r.state["job"]["exitCode"], -1);
    let mut s = source("");
    s["actions"]["run"]["steps"][0]["program"] = json!("nonexistent-command-954103.exe");
    let mut r = runtime(s);
    r.invoke("run").unwrap();
    wait(&mut r);
    assert_eq!(r.state["job"]["status"], "failed");
    assert!(!r.state["job"]["error"].as_str().unwrap().is_empty());
    let mut r = runtime(source(
        "[Console]::Out.Write(('x' * 200000)); [Console]::Error.Write(('y' * 200000))",
    ));
    r.invoke("run").unwrap();
    wait(&mut r);
    assert_eq!(r.state["job"]["status"], "succeeded");
    assert_eq!(r.state["job"]["stdout"].as_str().unwrap().len(), 16384);
    assert_eq!(r.state["job"]["stderr"].as_str().unwrap().len(), 16384);
}
