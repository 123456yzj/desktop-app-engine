use desktop_app_engine::{http, interaction::Runtime, v02};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
struct Reply {
    auth: Option<String>,
    code: u16,
    body: String,
    delay: u64,
    declared: Option<usize>,
}
impl Reply {
    fn json(value: Value) -> Self {
        Self {
            auth: None,
            code: 200,
            body: value.to_string(),
            delay: 0,
            declared: None,
        }
    }
}
struct Server {
    url: String,
    requests: mpsc::Receiver<String>,
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/metrics", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let (tx, requests) = mpsc::channel();
        let join = thread::spawn(move || {
            let mut count = 0;
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let mut bytes = Vec::new();
                        let mut buffer = [0; 2048];
                        loop {
                            let n = match stream.read(&mut buffer) {
                                Ok(0) | Err(_) => break,
                                Ok(n) => n,
                            };
                            bytes.extend_from_slice(&buffer[..n]);
                            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                                let header = String::from_utf8_lossy(&bytes[..end]);
                                let length = header
                                    .lines()
                                    .find_map(|l| {
                                        l.to_ascii_lowercase()
                                            .strip_prefix("content-length: ")
                                            .and_then(|v| v.parse::<usize>().ok())
                                    })
                                    .unwrap_or(0);
                                if bytes.len() >= end + 4 + length {
                                    break;
                                }
                            }
                        }
                        if bytes.is_empty() {
                            continue;
                        }
                        let _ = tx.send(String::from_utf8_lossy(&bytes).into_owned());
                        let reply = &replies[count.min(replies.len() - 1)];
                        count += 1;
                        thread::sleep(Duration::from_millis(reply.delay));
                        let authenticated = reply.auth.as_ref().is_none_or(|expected| {
                            String::from_utf8_lossy(&bytes).lines().any(|line| {
                                line.strip_prefix("authorization: ")
                                    .or_else(|| line.strip_prefix("Authorization: "))
                                    == Some(expected.as_str())
                            })
                        });
                        let code = if authenticated { reply.code } else { 401 };
                        let header = format!(
                            "HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            code,
                            reply.declared.unwrap_or(reply.body.len())
                        );
                        let _ = stream.write_all(header.as_bytes());
                        let _ = stream.write_all(reply.body.as_bytes());
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            url,
            requests,
            stop,
            join: Some(join),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.join.take().unwrap().join().unwrap();
    }
}
fn document(url: &str) -> Value {
    json!({"schemaVersion":"0.2","app":{"name":"HTTP test","window":{"width":800,"height":600}},"state":{"url":url,"scope":"香港 节点","token":"test-token","enabled":true,"status":http::initial_status(),"count":0,"message":"initial"},"dataSources":{"metrics":{"url":{"kind":"state","key":"url"},"headers":{"X-Test":{"kind":"state","key":"token"}},"query":{"scope":{"kind":"state","key":"scope"}},"statusTarget":"status","responseSample":{"data":{"count":12,"message":"ready"}},"mappings":[{"target":"count","path":"/data/count"},{"target":"message","path":"/data/message"}],"enabled":{"kind":"state","key":"enabled"}}},"root":{"type":"column","id":"page","children":[{"type":"metric","id":"metric","label":{"kind":"literal","value":"requests"},"value":{"kind":"state","key":"count"}},{"type":"text","id":"message","text":{"kind":"state","key":"message"}}]},"actions":{"refresh":{"steps":[{"type":"request","source":"metrics"}]}}})
}
fn runtime(s: Value) -> Runtime {
    Runtime::new(v02::parse(&s.to_string()).unwrap()).unwrap()
}
fn wait(r: &mut Runtime) {
    let end = Instant::now() + Duration::from_secs(5);
    while r.state["status"]["loading"] == true && Instant::now() < end {
        r.poll_http();
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(r.state["status"]["loading"], false, "{:?}", r.error);
    assert!(r.error.is_none(), "{:?}", r.error);
    if r.state["status"]["status"] == "failed" {
        println!("HTTP failed: {}", r.state["status"]);
    }
}
#[test]
fn get_bindings_snapshot_encoded_query_atomic_mapping_and_duplicate_skip() {
    let server = Server::new(vec![Reply {
        delay: 100,
        ..Reply::json(json!({"data":{"count":12,"message":"接口成功"}}))
    }]);
    let mut r = runtime(document(&server.url));
    let started = Instant::now();
    r.invoke("refresh").unwrap();
    assert!(started.elapsed() < Duration::from_millis(80));
    assert_eq!(r.state["status"]["status"], "loading");
    assert!(!r.request_http("metrics").unwrap());
    r.set("scope", json!("changed")).unwrap();
    let version = r.version;
    wait(&mut r);
    assert_eq!(r.state["count"], 12);
    assert_eq!(r.state["message"], "接口成功");
    assert_eq!(r.version, version + 1);
    assert_eq!(r.key_versions["count"], r.key_versions["message"]);
    assert_eq!(r.state["status"]["statusCode"], 200);
    assert!(r.state["status"]["updatedAt"].as_u64().unwrap() > 0);
    let request = server
        .requests
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    assert!(
        request.starts_with("GET /metrics?scope=%E9%A6%99%E6%B8%AF+%E8%8A%82%E7%82%B9 HTTP/1.1"),
        "{request}"
    );
    assert!(request.to_lowercase().contains("x-test: test-token"));
    assert!(server.requests.try_recv().is_err());
    let resolved = r.tree.resolve(&r.state).unwrap();
    assert_eq!(resolved[2].text, "接口成功");
}
#[test]
fn post_put_patch_delete_json_body_and_content_type() {
    for method in ["POST", "PUT", "PATCH", "DELETE"] {
        let server = Server::new(vec![Reply::json(
            json!({"data":{"count":1,"message":"ok"}}),
        )]);
        let mut s = document(&server.url);
        s["state"]["payload"] = json!({"name":"中文"});
        s["dataSources"]["metrics"]["method"] = json!(method);
        s["dataSources"]["metrics"]["body"] = json!({"kind":"state","key":"payload"});
        let mut r = runtime(s);
        r.request_http("metrics").unwrap();
        wait(&mut r);
        let request = server
            .requests
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        assert!(request.starts_with(method), "{request}");
        assert!(
            request
                .to_lowercase()
                .contains("content-type: application/json")
        );
        assert_eq!(
            serde_json::from_str::<Value>(request.split("\r\n\r\n").nth(1).unwrap()).unwrap(),
            json!({"name":"中文"})
        );
    }
}
#[test]
fn failure_http_json_pointer_type_and_component_constraints_preserve_old_data() {
    for reply in [
        Reply {
            code: 503,
            ..Reply::json(json!({}))
        },
        Reply {
            body: "not json".into(),
            ..Reply::json(json!({}))
        },
        Reply::json(json!({"data":{"count":999}})),
        Reply::json(json!({"data":{"count":999,"message":42}})),
    ] {
        let server = Server::new(vec![
            Reply::json(json!({"data":{"count":12,"message":"good"}})),
            reply,
        ]);
        let mut r = runtime(document(&server.url));
        r.request_http("metrics").unwrap();
        wait(&mut r);
        let timestamp = r.state["status"]["updatedAt"].clone();
        r.request_http("metrics").unwrap();
        wait(&mut r);
        assert_eq!(r.state["status"]["status"], "failed");
        assert_eq!(r.state["count"], 12);
        assert_eq!(r.state["message"], "good");
        assert_eq!(r.state["status"]["updatedAt"], timestamp);
    }
    let server = Server::new(vec![Reply::json(
        json!({"data":{"count":12,"message":"ok"}}),
    )]);
    let mut s = document(&server.url);
    s["root"]["children"][0] =
        json!({"type":"progress","id":"progress","value":{"kind":"state","key":"count"}});
    let mut r = runtime(s);
    r.request_http("metrics").unwrap();
    wait(&mut r);
    assert_eq!(r.state["status"]["status"], "failed");
    assert_eq!(r.state["count"], 0);
    assert_eq!(r.state["message"], "initial");
}
#[test]
fn timeout_oversize_and_redirect_fail_without_following() {
    for reply in [
        Reply {
            delay: 250,
            ..Reply::json(json!({}))
        },
        Reply {
            declared: Some(http::MAX_RESPONSE + 1),
            ..Reply::json(json!({}))
        },
        Reply {
            code: 302,
            ..Reply::json(json!({}))
        },
    ] {
        let server = Server::new(vec![reply]);
        let mut s = document(&server.url);
        s["dataSources"]["metrics"]["timeoutMs"] = json!(100);
        let mut r = runtime(s);
        r.request_http("metrics").unwrap();
        wait(&mut r);
        assert_eq!(r.state["status"]["status"], "failed");
        assert_eq!(r.state["count"], 0);
    }
}
#[test]
fn on_load_interval_pause_resume_and_manual_enabled_semantics() {
    let server = Server::new(vec![
        Reply::json(json!({"data":{"count":1,"message":"first"}})),
        Reply::json(json!({"data":{"count":2,"message":"second"}})),
        Reply::json(json!({"data":{"count":3,"message":"third"}})),
    ]);
    let mut s = document(&server.url);
    s["dataSources"]["metrics"]["onLoad"] = json!(true);
    s["dataSources"]["metrics"]["intervalMs"] = json!(100);
    let mut r = runtime(s);
    assert!(server.requests.try_recv().is_err());
    r.poll_http();
    wait(&mut r);
    assert_eq!(r.state["count"], 1);
    let end = Instant::now() + Duration::from_secs(3);
    while r.state["count"] != 2 && Instant::now() < end {
        r.poll_http();
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(r.state["count"], 2);
    r.set("enabled", json!(false)).unwrap();
    assert!(!r.request_http("metrics").unwrap());
    let count = server.requests.try_iter().count();
    assert_eq!(count, 2);
    for _ in 0..30 {
        r.poll_http();
        thread::sleep(Duration::from_millis(5));
    }
    assert!(server.requests.try_recv().is_err());
    r.set("enabled", json!(true)).unwrap();
    let end = Instant::now() + Duration::from_secs(3);
    while r.state["count"] != 3 && Instant::now() < end {
        r.poll_http();
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(r.state["count"], 3);
}
#[test]
fn config_rejects_invalid_contracts_and_global_sources_cannot_access_module_private_state() {
    let original = document("http://127.0.0.1:1/metrics");
    for (key, value) in [
        ("timeoutMs", json!(0)),
        ("intervalMs", json!(1)),
        ("method", json!("TRACE")),
        ("url", json!("file:///tmp/file")),
        ("mappings", json!([])),
        ("statusTarget", json!("message")),
        ("headers", json!({"X-Bad":"a\r\nb"})),
        ("body", json!({"kind":"literal","value":{}})),
        ("enabled", json!({"kind":"literal","value":1})),
    ] {
        let mut s = original.clone();
        s["dataSources"]["metrics"][key] = value;
        assert!(v02::parse(&s.to_string()).is_err(), "{key}");
    }
    let mut s = original.clone();
    s["dataSources"]["metrics"]["mappings"] = json!([{"target":"count","path":"/~2"}]);
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = original.clone();
    s["dataSources"]["metrics"]["mappings"] =
        json!([{"target":"count","path":"/data/count"},{"target":"count","path":"/data/count"}]);
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = original.clone();
    s["actions"]["refresh"]["steps"][0]["source"] = json!("missing");
    assert!(v02::parse(&s.to_string()).is_err());
    let mut s = original.clone();
    s["root"]["state"] = json!({"private":"secret"});
    s["dataSources"]["metrics"]["headers"] = json!({"X-Test":{"kind":"state","key":"private"}});
    assert!(v02::parse(&s.to_string()).is_err());
    let mut r = runtime(original);
    let version = r.version;
    assert!(r.set("url", json!("invalid")).is_err());
    assert_eq!(r.version, version);
}

#[test]
fn actual_http_response_updates_echarts_pixels_and_metrics() {
    let mut s: Value = serde_json::from_str(include_str!("../examples/http-chart.json")).unwrap();
    let first = s["dataSources"]["metrics"]["responseSample"].clone();
    let mut second = first.clone();
    second["data"]["stats"]["requests"] = json!(2400);
    second["data"]["option"]["series"][0]["data"] = json!([80, 20, 70]);
    let server = Server::new(vec![Reply::json(first), Reply::json(second)]);
    s["state"]["endpoint"] = json!(server.url);
    s["dataSources"]["metrics"]["onLoad"] = json!(false);
    s["dataSources"]["metrics"]
        .as_object_mut()
        .unwrap()
        .remove("intervalMs");
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut r = Runtime::new(v02::parse_at(&s.to_string(), &base).unwrap()).unwrap();
    let mut images = Vec::new();
    for _ in 0..2 {
        r.invoke("refresh").unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while r.state["requestStatus"]["loading"] == true && Instant::now() < end {
            r.poll_http();
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(r.state["requestStatus"]["status"], "success");
        r.sync_plugins(vec![("cpuChart".into(), 800, 330)]).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        while !r.plugin_images.contains_key("cpuChart") && r.error.is_none() && Instant::now() < end
        {
            r.poll_plugins();
            thread::sleep(Duration::from_millis(5));
        }
        assert!(r.error.is_none(), "{:?}", r.error);
        images.push(r.plugin_images.remove("cpuChart").unwrap().rgba);
    }
    assert_ne!(images[0], images[1]);
    assert_eq!(r.state["stats"]["requests"], 2400);
}

#[cfg(windows)]
#[test]
fn encrypted_bearer_header_authenticates_rotation_and_missing_secret_preserve_data() {
    struct Root(std::path::PathBuf);
    impl Drop for Root {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let root = Root(std::env::temp_dir().join(format!("dae-http-auth-{}", std::process::id())));
    std::fs::create_dir(&root.0).unwrap();
    let token = "local-test-token-982471";
    let store = desktop_app_engine::credentials::Store::at(&root.0, "auth-test").unwrap();
    store.put("api-token", token).unwrap();
    let server = Server::new(vec![Reply {
        auth: Some(format!("Bearer {token}")),
        ..Reply::json(json!({"data":{"count":12,"message":"authenticated"}}))
    }]);
    let mut s = document(&server.url);
    s["credentialStore"] = json!("auth-test");
    s["dataSources"]["metrics"]["headers"] =
        json!({"Authorization":{"kind":"credential","name":"api-token","prefix":"Bearer "}});
    let mut missing_store = s.clone();
    missing_store
        .as_object_mut()
        .unwrap()
        .remove("credentialStore");
    assert!(v02::parse(&missing_store.to_string()).is_err());
    let mut plaintext = s.clone();
    plaintext["dataSources"]["metrics"]["headers"]["Authorization"] =
        json!("Bearer forbidden-plaintext");
    assert!(
        v02::parse(&plaintext.to_string())
            .unwrap_err()
            .contains("必须引用加密凭据")
    );
    let mut invalid = s.clone();
    invalid["dataSources"]["metrics"]["headers"]["Authorization"]["prefix"] =
        json!("Bearer\r\nX-Leak: ");
    assert!(v02::parse(&invalid.to_string()).is_err());
    let mut misplaced = s.clone();
    misplaced["dataSources"]["metrics"]["query"] =
        json!({"token":{"kind":"credential","name":"api-token"}});
    assert!(v02::parse(&misplaced.to_string()).is_err());
    assert!(!s.to_string().contains(token));
    let mut r = runtime(s);
    r.credential_store = Some(store.clone());
    r.request_http("metrics").unwrap();
    wait(&mut r);
    assert_eq!(r.state["status"]["status"], "success");
    assert_eq!(r.state["count"], 12);
    assert!(
        server
            .requests
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .lines()
            .any(|line| line == format!("authorization: Bearer {token}"))
    );
    assert!(!serde_json::to_string(&r.state).unwrap().contains(token));
    assert!(!format!("{:?}", r.description).contains(token));
    store.put("api-token", "invalid-test-token").unwrap();
    r.request_http("metrics").unwrap();
    wait(&mut r);
    assert_eq!(r.state["status"]["statusCode"], 401);
    assert_eq!(r.state["count"], 12);
    let _ = server
        .requests
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    store.delete("api-token").unwrap();
    r.request_http("metrics").unwrap();
    wait(&mut r);
    assert_eq!(r.state["status"]["statusCode"], 0);
    assert_eq!(r.state["count"], 12);
    assert!(server.requests.try_recv().is_err());
    store.put("api-token", token).unwrap();
    r.request_http("metrics").unwrap();
    wait(&mut r);
    assert_eq!(r.state["status"]["status"], "success");
}
