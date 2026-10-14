#![cfg(windows)]
use desktop_app_engine::credentials::{MAX_SECRET, Store};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "dae-credentials-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn dpapi_ciphertext_roundtrip_rotation_and_delete() {
    let fixture = Fixture::new();
    let store = Store::at(&fixture.0, "test-store").unwrap();
    let token = "test-only-secret-凭据-824927";
    store.put("api-token", token).unwrap();
    let path = fixture.0.join("test-store/api-token.dpapi");
    let ciphertext = std::fs::read(&path).unwrap();
    assert!(
        !ciphertext
            .windows(token.len())
            .any(|w| w == token.as_bytes())
    );
    assert_eq!(store.get("api-token").unwrap().as_str(), token);
    assert_eq!(store.get("API-TOKEN").unwrap().as_str(), token);
    let first = ciphertext;
    store.put("api-token", token).unwrap();
    assert_ne!(
        first,
        std::fs::read(&path).unwrap(),
        "DPAPI should randomize encryption"
    );
    store.put("api-token", "rotated-test-value").unwrap();
    assert_eq!(
        store.get("api-token").unwrap().as_str(),
        "rotated-test-value"
    );
    assert_eq!(store.names().unwrap(), vec!["api-token"]);
    store.delete("api-token").unwrap();
    assert!(store.get("api-token").is_err());
    assert!(store.names().unwrap().is_empty());
}
#[test]
fn corrupted_or_swapped_ciphertext_fails_closed() {
    let fixture = Fixture::new();
    let a = Store::at(&fixture.0, "a").unwrap();
    let b = Store::at(&fixture.0, "b").unwrap();
    let secret = "only-test-plaintext-9fd281";
    a.put("token", secret).unwrap();
    b.put("token", "other-test-value").unwrap();
    let bytes = std::fs::read(fixture.0.join("a/token.dpapi")).unwrap();
    std::fs::write(fixture.0.join("b/token.dpapi"), &bytes).unwrap();
    assert!(b.get("token").is_err());
    std::fs::write(fixture.0.join("a/other.dpapi"), &bytes).unwrap();
    assert!(a.get("other").is_err());
    let mut bytes = bytes;
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    std::fs::write(fixture.0.join("a/token.dpapi"), bytes).unwrap();
    let err = a.get("token").unwrap_err();
    assert!(!err.contains(secret));
    assert!(err.contains("无法解密"));
}
#[test]
fn credential_names_input_size_and_plaintext_files_are_rejected() {
    let fixture = Fixture::new();
    let store = Store::at(&fixture.0, "valid").unwrap();
    for name in ["", "../escape", "name/file", "a.b"] {
        assert!(store.put(name, "test").is_err());
    }
    for value in [
        "".to_owned(),
        "line\nbreak".into(),
        "with\0nul".into(),
        "x".repeat(MAX_SECRET + 1),
    ] {
        assert!(store.put("token", &value).is_err());
    }
    store.put("token", "valid-test-value").unwrap();
    std::fs::write(fixture.0.join("valid/token.dpapi"), b"plaintext-test").unwrap();
    assert!(store.get("token").is_err());
}
fn cli(fixture: &Fixture, args: &[&str], input: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_desktop-app-engine"));
    cmd.args(args)
        .env("LOCALAPPDATA", &fixture.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    } else {
        drop(child.stdin.take());
    }
    child.wait_with_output().unwrap()
}
#[test]
fn cli_stdin_encrypts_without_echo_and_offline_checks_do_not_require_credentials() {
    let fixture = Fixture::new();
    let secret = "cli-test-token-027536";
    let output = cli(
        &fixture,
        &[
            "--credential-store",
            "cli-test",
            "--credential-set",
            "api-token",
        ],
        Some(secret),
    );
    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains(secret));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(secret));
    let ciphertext = std::fs::read(
        fixture
            .0
            .join("DesktopAppEngine/credentials/cli-test/api-token.dpapi"),
    )
    .unwrap();
    assert!(
        !ciphertext
            .windows(secret.len())
            .any(|w| w == secret.as_bytes())
    );
    let mut app: serde_json::Value =
        serde_json::from_str(include_str!("../examples/http-auth.json")).unwrap();
    app["credentialStore"] = serde_json::json!("cli-test");
    let file = fixture.0.join("app.json");
    std::fs::write(&file, app.to_string()).unwrap();
    let file = file.to_str().unwrap();
    let checked = cli(&fixture, &["--app", file, "--check-credentials"], None);
    assert!(checked.status.success());
    assert!(!String::from_utf8_lossy(&checked.stdout).contains(secret));
    let names = cli(
        &fixture,
        &["--credential-store", "cli-test", "--credential-list"],
        None,
    );
    assert_eq!(String::from_utf8_lossy(&names.stdout).trim(), "api-token");
    assert!(
        cli(
            &fixture,
            &[
                "--credential-store",
                "cli-test",
                "--credential-delete",
                "api-token"
            ],
            None
        )
        .status
        .success()
    );
    assert!(
        cli(&fixture, &["--app", file, "--check"], None)
            .status
            .success()
    );
    assert!(
        !cli(&fixture, &["--app", file, "--check-credentials"], None)
            .status
            .success()
    );
    let bad = cli(
        &fixture,
        &[
            "--credential-store",
            "cli-test",
            "--credential-set",
            "api-token",
        ],
        Some("a\nb"),
    );
    assert!(!bad.status.success());
    assert!(!String::from_utf8_lossy(&bad.stderr).contains("a\nb"));
}
