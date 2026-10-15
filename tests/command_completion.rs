use desktop_app_engine::{command, interaction::Runtime, v02};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn source(script: &str) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"completion","window":{"width":600,"height":400}},
        "state":{"job":command::initial_result(),"value":{"branch":"old"},"unlocked":false},
        "root":{"type":"text","id":"label","text":{"kind":"state","key":"value","path":"/branch"}},
        "actions":{"run":{"steps":[{"type":"command","program":"powershell.exe","args":["-NoProfile","-NonInteractive","-Command",script],"target":"job","onComplete":"finish","timeoutMs":1000}]},
        "finish":{"steps":[{"type":"set","target":"unlocked","value":{"kind":"literal","value":true}},{"type":"parseJson","target":"value","value":{"kind":"state","key":"job","path":"/stdout"}}]}}})
}
fn wait(r: &mut Runtime) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while r.commands_running() {
        assert!(Instant::now() < deadline);
        r.poll_commands();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn callback_reads_committed_result_on_success_and_failure() {
    for code in [0, 7] {
        let mut r = Runtime::new(
            v02::parse(
                &source(&format!(
                    "[Console]::Out.Write('{{\"branch\":\"dev\"}}'); exit {code}"
                ))
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        r.invoke("run").unwrap();
        wait(&mut r);
        assert!(r.error.is_none(), "{:?}", r.error);
        assert_eq!(r.state["unlocked"], true);
        assert_eq!(r.state["value"]["branch"], "dev");
        assert_eq!(r.state["job"]["exitCode"], code);
    }
}

#[test]
fn timeout_launch_and_bad_json_unlock_and_preserve_typed_state() {
    for script in [
        "[Console]::Out.Write('bad')",
        "[Console]::Out.Write('{\"branch\":42}')",
        "Start-Sleep -Seconds 20",
        "exit 0",
    ] {
        let mut s = source(script);
        if script == "exit 0" {
            s["actions"]["run"]["steps"][0]["program"] = json!("missing-vben-9261.exe");
        }
        let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
        r.invoke("run").unwrap();
        wait(&mut r);
        assert_eq!(r.state["unlocked"], true);
        assert_eq!(r.state["value"]["branch"], "old");
        assert!(r.error.is_some());
    }
}

#[test]
fn callbacks_and_parse_targets_follow_module_scope() {
    let mut s = source("[Console]::Out.Write('{\"branch\":\"dev\"}')");
    let state = s["state"].take();
    let actions = s["actions"].take();
    let child = s["root"].take();
    s["state"] = json!({});
    s["actions"] = json!({});
    s["root"] =
        json!({"type":"module","id":"module","state":state,"actions":actions,"children":[child]});
    let mut r = Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap();
    let id = r.tree.nodes[0].spec.action_names["run"].clone();
    r.invoke(&id).unwrap();
    wait(&mut r);
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.state_from("label", "value").unwrap()["branch"], "dev");
    for value in [json!("missing"), json!("")] {
        let mut s = source("exit 0");
        s["actions"]["run"]["steps"][0]["onComplete"] = value;
        assert!(v02::parse(&s.to_string()).is_err());
    }
}
