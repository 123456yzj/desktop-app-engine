use desktop_app_engine::{EXAMPLE, loader};
use std::{fs, process::Command};

#[test]
fn packaged_loading_priority_and_errors() {
    let directory = std::env::temp_dir().join(format!("packaging-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    {
        assert_eq!(
            loader::load_from_directory(&directory)
                .unwrap()
                .schema_version,
            "0.1"
        );
        let mut value: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
        value["app"]["name"] = "同目录应用".into();
        fs::write(directory.join("app.json"), value.to_string()).unwrap();
        let executable = directory.join("运行时.exe");
        fs::copy(env!("CARGO_BIN_EXE_desktop-app-engine"), &executable).unwrap();
        let check = |args: &[&str]| {
            Command::new(&executable)
                .args(args)
                .current_dir(directory.parent().unwrap())
                .output()
                .unwrap()
        };
        let output = check(&["--check"]);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("同目录应用"));
        assert_eq!(
            loader::load_from_directory(&directory).unwrap().app.name,
            "同目录应用"
        );
        fs::write(directory.join("app.json"), "{").unwrap();
        assert!(loader::load_from_directory(&directory).is_err());
        let output = check(&["--check"]);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("外部描述"));
        let explicit = directory.join("explicit.json");
        fs::write(&explicit, EXAMPLE).unwrap();
        assert!(
            check(&["--app", explicit.to_str().unwrap(), "--check"])
                .status
                .success()
        );
        assert_eq!(
            check(&["--app", "missing.json", "--check"]).status.code(),
            Some(2)
        );
        fs::remove_file(directory.join("app.json")).unwrap();
        assert!(check(&["--check"]).status.success());
    }
    fs::remove_dir_all(&directory).unwrap();
}
