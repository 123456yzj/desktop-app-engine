use desktop_app_engine::{
    interaction::{Runtime, UiEvent},
    v02,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn event(r: &mut Runtime, id: &str, kind: &str, payload: Value, sequence: u64) {
    r.commit(vec![UiEvent {
        component_id: id.into(),
        target_key: None,
        kind: kind.into(),
        payload,
        sequence,
        generation: 1,
    }]);
}
fn wait(r: &mut Runtime) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while r.commands_running() {
        assert!(Instant::now() < deadline);
        r.poll_commands();
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn response(s: &Value, branch: &str, success: bool) -> Value {
    let mut v = s["state"]["view"].clone();
    v["currentBranch"] = json!(branch);
    v["branches"] = json!("dev\nfeature/tms");
    v["ready"] = json!(true);
    v["hasBranches"] = json!(true);
    v["success"] = json!(success);
    v["failure"] = json!(!success);
    v
}
fn mock(s: &mut Value, action: &str, response: Value, code: i32) {
    let step = s["actions"][action]["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["type"] == "command")
        .unwrap();
    let output = response.to_string().replace('\'', "''");
    step["program"] = json!("powershell.exe");
    step["args"] = json!([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        format!(
            "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); [Console]::Out.Write('{output}'); exit {code}"
        )
    ]);
}
#[test]
fn read_select_switch_build_and_project_change_use_real_callback_chain() {
    let mut s: Value =
        serde_json::from_str(include_str!("../examples/vben-package-legacy.json")).unwrap();
    let refreshed = response(&s, "dev", true);
    mock(&mut s, "refresh", refreshed, 0);
    let switched = response(&s, "feature/tms", true);
    mock(&mut s, "switch", switched.clone(), 0);
    let mut built = switched;
    built["buildResult"] = json!("打包成功");
    built["buildBranch"] = json!("feature/tms");
    mock(&mut s, "build", built, 0);
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    event(&mut r, "buildButton", "click", Value::Null, 1);
    assert!(!r.commands_running());
    event(&mut r, "refreshButton", "click", Value::Null, 2);
    assert_eq!(r.state["idle"], false);
    assert_eq!(r.state["busy"], true);
    event(&mut r, "buildButton", "click", Value::Null, 3);
    wait(&mut r);
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.state["view"]["currentBranch"], "dev");
    event(&mut r, "branch", "change", json!("feature/tms"), 4);
    assert_eq!(r.state["view"]["currentBranch"], "dev");
    event(&mut r, "switchButton", "click", Value::Null, 5);
    wait(&mut r);
    assert_eq!(r.state["view"]["currentBranch"], "feature/tms");
    event(&mut r, "buildButton", "click", Value::Null, 6);
    wait(&mut r);
    assert_eq!(r.state["view"]["buildResult"], "打包成功");
    assert_eq!(r.state["view"]["buildBranch"], "feature/tms");
    event(&mut r, "sshTarget", "change", json!("other-host"), 7);
    assert_eq!(r.state["view"]["ready"], false);
    assert_eq!(r.state["view"]["currentBranch"], "尚未读取");
    assert_eq!(r.state["branch"], "");
    assert_eq!(r.state["lastResponse"], "{}");
}

#[test]
fn failed_switch_keeps_reported_branch_and_launch_failure_unlocks_ui() {
    let mut s: Value =
        serde_json::from_str(include_str!("../examples/vben-package-legacy.json")).unwrap();
    let initial = response(&s, "dev", true);
    s["state"]["view"] = initial;
    s["state"]["branch"] = json!("feature/tms");
    s["state"]["switchAvailable"] = json!(true);
    let failed = response(&s, "dev", false);
    mock(&mut s, "switch", failed, 1);
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    event(&mut r, "switchButton", "click", Value::Null, 1);
    wait(&mut r);
    assert_eq!(r.state["branch"], "feature/tms");
    assert_eq!(r.state["view"]["currentBranch"], "dev");
    assert_eq!(r.state["view"]["failure"], true);
    assert_eq!(r.state["idle"], true);
    let step = s["actions"]["build"]["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["type"] == "command")
        .unwrap();
    step["program"] = json!("missing-vben-2811.exe");
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    event(&mut r, "buildButton", "click", Value::Null, 1);
    wait(&mut r);
    assert_eq!(r.state["idle"], true);
    assert_eq!(r.state["busy"], false);
    assert_eq!(r.state["view"]["ready"], false);
    assert_eq!(r.state["view"]["failure"], true);
}
