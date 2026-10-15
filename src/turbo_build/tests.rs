use super::*;
#[path = "render_tests.rs"]
mod render_tests;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

fn git(root: &Path, args: &[&str]) -> String {
    let result = Command::new("git")
        .args(args)
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().into()
}
fn fixture() -> (PathBuf, Controller, Value) {
    static SERIAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "turbo-native-{}-{}-{}",
        std::process::id(),
        logging::now_ms(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("apps")).unwrap();
    fs::write(root.join(".bash_logout"), "false\n").unwrap();
    for id in ["web-admin", "web-bms", "web-copilot", "web-tms"] {
        let cwd = root.join("apps").join(id);
        fs::create_dir(&cwd).unwrap();
        fs::write(
            cwd.join("package.json"),
            json!({"name":format!("@repo/{}",id.strip_prefix("web-").unwrap())}).to_string(),
        )
        .unwrap();
        git(&cwd, &["init", "-b", "dev"]);
        git(&cwd, &["add", "package.json"]);
        git(&cwd, &["commit", "-m", "fixture"]);
        git(&cwd, &["branch", "feature/native"]);
    }
    fs::create_dir(root.join("apps/not-a-repo")).unwrap();
    fs::write(root.join("package.json"),json!({"scripts":{"build:dev":"turbo build","build:dev:admin":"turbo admin","build:dev:bms":"turbo bms","build:dev:copilot":"turbo copilot","build:dev:tms":"turbo tms","build:dev:multichannel":"turbo absent"}}).to_string()).unwrap();
    fs::write(
        root.join(".gitignore"),
        "apps/\nlogs/\n.turbo/\n.*-invocations\n.fail-*\n.bash_logout\napp.json\n",
    )
    .unwrap();
    git(&root, &["init", "-b", "main"]);
    git(&root, &["add", "package.json", ".gitignore"]);
    git(&root, &["commit", "-m", "root fixture"]);
    git(&root, &["branch", "root/next"]);
    fs::create_dir(root.join(".turbo")).unwrap();
    fs::write(root.join(".turbo/stale"), "old").unwrap();
    let windows = root.to_string_lossy().replace('\\', "/");
    let linux = format!("/{}{}", windows[..1].to_lowercase(), &windows[2..]);
    let backend = Backend {
        base: root.clone(),
        session: logging::Session::new(&root.join("logs")).unwrap(),
        program: "python.exe".into(),
        prefix_args: vec![
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/turbo_ssh.py")
                .to_string_lossy()
                .into_owned(),
        ],
    };
    let model = Controller::new(
        Connection {
            host: "fixture-host".into(),
            root: linux,
        },
        backend,
        None,
    );
    let template = serde_json::from_str(include_str!("../../examples/vben-package.json")).unwrap();
    (root, model, template)
}
fn click(model: &mut Controller, id: &str) {
    send(model, id, "click", Value::Null);
}
fn send(model: &mut Controller, id: &str, kind: &str, payload: Value) {
    model.handle(UiEvent {
        component_id: id.into(),
        target_key: None,
        kind: kind.into(),
        payload,
        sequence: 1,
        generation: 1,
    });
}
fn wait(model: &mut Controller) {
    let deadline = Instant::now() + Duration::from_secs(25);
    while !model.idle() {
        assert!(Instant::now() < deadline, "{}", model.note);
        model.poll();
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn select(model: &mut Controller, ids: &[&str]) {
    click(model, "clear");
    for id in ids {
        click(model, &format!("choose:{id}"));
    }
}
fn build(model: &mut Controller) {
    click(model, "build");
    assert!(model.pending.is_some(), "{}", model.error);
    click(model, "confirmBuild");
    assert!(!model.idle(), "{}", model.error);
}
fn count(root: &Path, file: &str) -> usize {
    fs::read_to_string(root.join(file))
        .unwrap_or_default()
        .lines()
        .count()
}

#[test]
fn root_branch_checkout_fetch_and_pull_refresh_workspace_scripts() {
    let (root, mut model, template) = fixture();
    let original = fs::read_to_string(root.join("package.json")).unwrap();
    git(&root, &["checkout", "root/next"]);
    let mut next: Value = serde_json::from_str(&original).unwrap();
    next["scripts"]
        .as_object_mut()
        .unwrap()
        .remove("build:dev:bms");
    next["scripts"].as_object_mut().unwrap().remove("build:dev");
    fs::write(root.join("package.json"), next.to_string()).unwrap();
    git(&root, &["add", "package.json"]);
    git(&root, &["commit", "-m", "different root scripts"]);
    git(&root, &["checkout", "main"]);
    model.scan();
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    let repo = model.root_repo.as_ref().unwrap();
    assert_eq!(repo.project.branch, "main");
    assert!(repo.branches.contains(&"root/next".into()));
    let mut doc = view::document_json(&template, &model).unwrap();
    assert_eq!(doc["state"]["target_@root"], "main");
    assert!(view::node_mut(&mut doc["root"], "build:@root").is_none());
    assert!(view::node_mut(&mut doc["root"], "choose:@root").is_none());
    click(&mut model, "all");
    assert_eq!(model.selected.len(), 4);
    assert_eq!(model.plan(&model.selected).unwrap().projects.len(), 4);
    let tasks = model.session_tasks.len();
    send(&mut model, "target:@root", "open", Value::Null);
    send(&mut model, "target:@root", "change", json!("main"));
    send(&mut model, "target:@root", "change", json!("unknown"));
    assert_eq!(tasks, model.session_tasks.len());
    // A real checkout conflict must preserve the local file and restore actual branch display.
    fs::write(root.join("package.json"), format!("{original}\n")).unwrap();
    send(&mut model, "target:@root", "change", json!("root/next"));
    assert_eq!(
        view::document_json(&template, &model).unwrap()["state"]["target_@root"],
        "root/next"
    );
    wait(&mut model);
    assert!(!model.error.is_empty());
    assert_eq!(model.root_repo.as_ref().unwrap().project.branch, "main");
    assert!(model.root_repo.as_ref().unwrap().target.is_empty());
    assert_eq!(
        fs::read_to_string(root.join("package.json")).unwrap(),
        format!("{original}\n")
    );
    git(&root, &["checkout", "--", "package.json"]);
    send(&mut model, "target:@root", "change", json!("root/next"));
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_eq!(git(&root, &["branch", "--show-current"]), "root/next");
    assert_eq!(
        model.root_repo.as_ref().unwrap().project.branch,
        "root/next"
    );
    assert!(model.workspace_ready);
    assert!(!model.all_script);
    assert!(model.repos[1].project.script.is_none());
    assert!(!model.selected.contains("web-bms"));
    assert!(model.repos.iter().all(|r| r.project.branch == "dev"));
    send(&mut model, "target:@root", "change", json!("main"));
    wait(&mut model);
    assert!(model.all_script);
    assert!(model.repos.iter().all(|r| r.project.available()));
    git(&root, &["branch", "root/added"]);
    click(&mut model, "fetch:@root");
    wait(&mut model);
    assert!(
        model
            .root_repo
            .as_ref()
            .unwrap()
            .branches
            .contains(&"root/added".into())
    );
    let cached = model.root_repo.as_ref().unwrap().branches.clone();
    git(
        &root,
        &[
            "remote",
            "add",
            "broken",
            root.join("absent.git").to_str().unwrap(),
        ],
    );
    git(
        &root.join("apps/web-admin"),
        &["branch", "despite-root-fetch-error"],
    );
    model.scan();
    wait(&mut model);
    assert!(model.error.contains("Turbo 根仓库"));
    assert!(model.root_repo.as_ref().unwrap().refresh_error);
    assert_eq!(model.root_repo.as_ref().unwrap().branches, cached);
    assert!(
        model.repos[0]
            .branches
            .contains(&"despite-root-fetch-error".into())
    );
    git(&root, &["remote", "remove", "broken"]);
    // Pull against a disposable bare remote, then re-read the changed root scripts.
    let upstream = root.join("root-upstream.git");
    fs::create_dir(&upstream).unwrap();
    git(&upstream, &["init", "--bare", "--initial-branch=main"]);
    git(
        &root,
        &["remote", "add", "origin", upstream.to_str().unwrap()],
    );
    git(&root, &["push", "-u", "origin", "main"]);
    let author = root.join("root-author");
    git(
        &root,
        &[
            "clone",
            upstream.to_str().unwrap(),
            author.to_str().unwrap(),
        ],
    );
    let mut pulled: Value = serde_json::from_str(&original).unwrap();
    pulled["scripts"]
        .as_object_mut()
        .unwrap()
        .remove("build:dev:admin");
    fs::write(author.join("package.json"), pulled.to_string()).unwrap();
    git(&author, &["add", "package.json"]);
    git(&author, &["commit", "-m", "root upstream scripts"]);
    git(&author, &["push"]);
    let before = model.root_repo.as_ref().unwrap().project.head.clone();
    click(&mut model, "pull:@root");
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_ne!(model.root_repo.as_ref().unwrap().project.head, before);
    assert!(model.repos[0].project.script.is_none());
    assert_eq!(model.repos.len(), 4);
    view::document(&template, &model).unwrap();
}

#[test]
fn root_snapshot_preflight_blocks_external_branch_and_commit_changes() {
    let (root, mut model, _) = fixture();
    model.scan();
    wait(&mut model);
    select(&mut model, &["web-bms"]);
    assert_eq!(
        model.plan(&model.selected).unwrap().root.unwrap().branch,
        "main"
    );
    git(&root, &["checkout", "root/next"]);
    build(&mut model);
    wait(&mut model);
    assert!(
        model.error.contains("Turbo 根仓库 分支或提交已变化"),
        "{}",
        model.error
    );
    assert_eq!(count(&root, ".clean-invocations"), 0);
    assert_eq!(count(&root, ".build-invocations"), 0);
    let stale = model.plan(&model.selected).unwrap();
    let operation = Operation::Build(stale.projects, "build:dev:bms".into(), stale.root);
    let request = model
        .backend
        .request(&model.connection, &operation)
        .unwrap();
    let result = desktop_app_engine::command::run(request, Arc::new(AtomicBool::new(false)));
    assert_ne!(
        result["exitCode"], 0,
        "each build process must verify the root snapshot too"
    );
    assert_eq!(count(&root, ".build-invocations"), 0);
    // Fetch also notices an externally changed root branch and refreshes workspace metadata.
    click(&mut model, "fetch:@root");
    wait(&mut model);
    assert_eq!(
        model.root_repo.as_ref().unwrap().project.branch,
        "root/next"
    );
    assert!(model.workspace_ready);
    git(
        &root,
        &["commit", "--allow-empty", "-m", "external root commit"],
    );
    build(&mut model);
    wait(&mut model);
    assert!(model.error.contains("Turbo 根仓库 分支或提交已变化"));
    assert_eq!(count(&root, ".clean-invocations"), 0);
    assert_eq!(count(&root, ".build-invocations"), 0);
    // A failed workspace rescan must disable building from stale scripts/snapshots.
    fs::rename(root.join("package.json"), root.join("package-saved.json")).unwrap();
    model.scan_read_only();
    wait(&mut model);
    assert!(!model.workspace_ready);
    assert!(model.plan(&model.selected).is_err());
}

#[test]
fn workspace_without_root_git_retains_business_build_support() {
    let (root, mut model, template) = fixture();
    fs::rename(root.join(".git"), root.join("root-git-saved")).unwrap();
    model.scan();
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert!(model.root_repo.is_none());
    assert_eq!(model.repos.len(), 4);
    let mut doc = view::document_json(&template, &model).unwrap();
    assert_eq!(
        view::node_mut(&mut doc["root"], "fetch:@root").unwrap()["enabled"]["value"],
        false
    );
    select(&mut model, &["web-bms"]);
    assert!(model.plan(&model.selected).unwrap().root.is_none());
    build(&mut model);
    wait(&mut model);
    assert_eq!(model.runs[0].status, Status::Success);
}

#[test]
fn branch_selection_checks_out_immediately_and_restores_actual_branch_on_failure() {
    let (root, mut model, template) = fixture();
    let cwd = root.join("apps/web-admin");
    fs::write(cwd.join("guard.txt"), "dev\n").unwrap();
    git(&cwd, &["add", "guard.txt"]);
    git(&cwd, &["commit", "-m", "dev guard"]);
    git(&cwd, &["checkout", "feature/native"]);
    fs::write(cwd.join("guard.txt"), "feature\n").unwrap();
    git(&cwd, &["add", "guard.txt"]);
    git(&cwd, &["commit", "-m", "feature guard"]);
    git(&cwd, &["checkout", "dev"]);
    fs::write(cwd.join("guard.txt"), "local edit\n").unwrap();
    model.scan();
    wait(&mut model);
    let mut initial = view::document_json(&template, &model).unwrap();
    assert_eq!(initial["state"]["target_web-admin"], "dev");
    assert!(view::node_mut(&mut initial["root"], "switch:web-admin").is_none());
    assert!(view::node_mut(&mut initial["root"], "current:web-admin").is_none());
    assert_eq!(
        view::node_mut(&mut initial["root"], "tableHeader").unwrap()["columns"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let tasks = model.session_tasks.len();
    send(&mut model, "target:web-admin", "change", json!("dev"));
    send(
        &mut model,
        "target:web-admin",
        "change",
        json!("missing-branch"),
    );
    assert!(model.idle());
    assert_eq!(model.session_tasks.len(), tasks);
    send(
        &mut model,
        "target:web-admin",
        "change",
        json!("feature/native"),
    );
    assert!(matches!(model.active, Some(Active::Change(_))));
    let mut switching = view::document_json(&template, &model).unwrap();
    assert_eq!(switching["state"]["target_web-admin"], "feature/native");
    assert_eq!(
        view::node_mut(&mut switching["root"], "target:web-admin").unwrap()["enabled"]["value"],
        false
    );
    send(&mut model, "target:web-admin", "change", json!("dev"));
    assert_eq!(model.session_tasks.len(), tasks + 1);
    wait(&mut model);
    assert!(!model.error.is_empty());
    assert_eq!(model.repos[0].project.branch, "dev");
    assert!(model.repos[0].target.is_empty());
    assert_eq!(
        view::document_json(&template, &model).unwrap()["state"]["target_web-admin"],
        "dev"
    );
    assert_eq!(
        fs::read_to_string(cwd.join("guard.txt")).unwrap(),
        "local edit\n"
    );
    git(&cwd, &["checkout", "--", "guard.txt"]);
    send(
        &mut model,
        "target:web-admin",
        "change",
        json!("feature/native"),
    );
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_eq!(git(&cwd, &["branch", "--show-current"]), "feature/native");
    assert_eq!(
        view::document_json(&template, &model).unwrap()["state"]["target_web-admin"],
        "feature/native"
    );
    let tasks = model.session_tasks.len();
    send(
        &mut model,
        "target:web-admin",
        "change",
        json!("feature/native"),
    );
    assert!(model.idle());
    assert_eq!(model.session_tasks.len(), tasks);
}

#[test]
fn log_scrolling_pauses_follow_and_never_shows_full_text_tooltip() {
    use eframe::egui;
    let (_, mut model, template) = fixture();
    let task = model
        .backend
        .session
        .task("日志滚动测试", "ssh.exe", &model.backend.base, 120000)
        .unwrap();
    let lines = (0..200)
        .map(|n| format!("log-line-{n:03}\n"))
        .collect::<String>();
    task.write(false, lines.as_bytes());
    model.session_tasks.push(task.clone());
    let mut app = crate::App::new(model, template.clone()).unwrap();
    let ctx = egui::Context::default();
    desktop_app_engine::gui::install_chinese_font(&ctx).unwrap();
    let mut time = 0.0;
    let mut frame = |app: &mut crate::App, events| {
        time += 0.1;
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1440.0, 900.0),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| app.draw_main(ctx),
        )
    };
    let log_shape = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .find_map(|s| {
                let egui::epaint::Shape::Text(t) = &s.shape else {
                    return None;
                };
                t.galley
                    .job
                    .text
                    .contains("log-line-000")
                    .then_some((t.pos, s.clip_rect))
            })
            .expect("log text is painted")
    };
    frame(&mut app, vec![]);
    let output = frame(&mut app, vec![]);
    let (bottom, clip) = log_shape(&output);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(clip.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 400.0),
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert!(!app.model.auto_scroll, "wheel input pauses following");
    for _ in 0..30 {
        frame(&mut app, vec![]);
    }
    let output = frame(&mut app, vec![]);
    let (scrolled, _) = log_shape(&output);
    assert!(scrolled.y > bottom.y + 100.0, "wheel reveals older logs");
    assert_eq!(output.shapes.iter().filter(|s| matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text.contains("log-line-000"))).count(),1,"hovering logs must not create a duplicate full-text tooltip");
    task.write(false, b"new-log-line\n");
    app.main
        .replace_description(view::document(&template, &app.model).unwrap())
        .unwrap();
    let output = frame(&mut app, vec![]);
    assert!(
        (log_shape(&output).0.y - scrolled.y).abs() < 1.0,
        "new output and view replacement preserve manual position"
    );
    click(&mut app.model, "autoScroll");
    assert!(app.model.auto_scroll);
    let output = frame(&mut app, vec![]);
    assert!(
        log_shape(&output).0.y < scrolled.y - 100.0,
        "enabling follow returns to the bottom"
    );
    let track = egui::pos2(clip.right() + 5.0, clip.top() + 40.0);
    for pressed in [true, false] {
        frame(
            &mut app,
            vec![
                egui::Event::PointerMoved(track),
                egui::Event::PointerButton {
                    pos: track,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    assert!(!app.model.auto_scroll, "scrollbar input pauses following");
    let output = frame(&mut app, vec![]);
    assert!(
        log_shape(&output).0.y > bottom.y + 100.0,
        "scrollbar can reveal older logs"
    );
}

#[test]
fn scan_refreshes_all_branch_caches_and_preserves_failed_repositories() {
    let (root, mut model, _) = fixture();
    model.scan();
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert!(
        model
            .repos
            .iter()
            .all(|r| r.branches.contains(&"feature/native".into()))
    );
    let admin = root.join("apps/web-admin");
    let bms = root.join("apps/web-bms");
    git(&admin, &["branch", "from-rescan"]);
    click(&mut model, "scan");
    wait(&mut model);
    assert!(model.repos[0].branches.contains(&"from-rescan".into()));
    send(
        &mut model,
        "target:web-bms",
        "change",
        json!("feature/native"),
    );
    wait(&mut model);
    let old_branches = model.repos[1].branches.clone();
    git(
        &bms,
        &[
            "remote",
            "add",
            "broken",
            &root.join("missing.git").to_string_lossy(),
        ],
    );
    git(
        &root.join("apps/web-copilot"),
        &["branch", "other-repo-still-refreshes"],
    );
    click(&mut model, "scan");
    wait(&mut model);
    assert!(model.error.contains("web-bms"), "{}", model.error);
    assert!(model.repos[1].refresh_error);
    assert_eq!(model.repos[1].branches, old_branches);
    assert_eq!(model.repos[1].project.branch, "feature/native");
    assert!(
        model.repos[2]
            .branches
            .contains(&"other-repo-still-refreshes".into())
    );
    git(&bms, &["remote", "remove", "broken"]);
    git(&bms, &["branch", "from-saved-connection"]);
    click(&mut model, "settings");
    click(&mut model, "saveSettings");
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert!(!model.repos[1].refresh_error);
    assert!(
        model.repos[1]
            .branches
            .contains(&"from-saved-connection".into())
    );
    assert_eq!(model.repos[1].project.branch, "feature/native");
    git(&admin, &["branch", "read-only-does-not-fetch"]);
    model.scan_read_only();
    wait(&mut model);
    assert!(
        !model.repos[0]
            .branches
            .contains(&"read-only-does-not-fetch".into())
    );
    let log = model.log_text();
    assert!(log.contains("[fetch] web-admin"));
    assert!(log.contains("[fetch] web-bms · 失败"));
}

#[test]
fn actual_bash_node_git_workflow_state_and_process_logs() {
    let (root, mut model, template) = fixture();
    model.scan();
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_eq!(model.repos.len(), 4);
    assert!(model.selected.is_empty());
    click(&mut model, "fetch:web-admin");
    assert!(matches!(model.active, Some(Active::Branches(_))));
    let loading = view::document_json(&template, &model).unwrap();
    assert_eq!(
        view::node_mut(&mut loading.clone()["root"], "build").unwrap()["enabled"]["value"],
        false
    );
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    send(
        &mut model,
        "target:web-admin",
        "change",
        json!("feature/native"),
    );
    assert_eq!(model.repos[0].project.branch, "dev");
    assert!(matches!(model.active, Some(Active::Change(_))));
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_eq!(model.repos[0].project.branch, "feature/native");
    assert_eq!(model.repos[1].project.branch, "dev");
    select(&mut model, &["web-bms", "web-tms"]);
    build(&mut model);
    assert_eq!(model.runs[0].jobs.len(), 2);
    let busy = view::document_json(&template, &model).unwrap();
    assert_eq!(
        view::node_mut(&mut busy.clone()["root"], "logs:web-tms").unwrap()["enabled"]["value"],
        true
    );
    click(&mut model, "logs:web-tms");
    let pinned = model.log_view;
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert!(model.log_view == pinned);
    assert!(model.runs[0].status == Status::Success);
    assert_eq!(count(&root, ".clean-invocations"), 1);
    assert_eq!(count(&root, ".build-invocations"), 2);
    assert!(!root.join(".turbo/stale").exists());
    assert!(root.join(".turbo/generated").exists());
    assert!(model.log_text().contains("OUTPUT-END build:dev:tms"));
    assert!(!model.log_text().contains("OUTPUT-END build:dev:bms"));
    click(&mut model, "clearLogs");
    assert!(model.log_text().is_empty());
    model.open_logs(pinned);
    assert!(model.log_text().contains("OUTPUT-END build:dev:tms"));
    let first_task = model.runs[0].jobs[1].task.as_ref().unwrap().id.clone();
    fs::write(root.join(".fail-build"), "build:dev:bms").unwrap();
    select(&mut model, &["web-admin", "web-bms", "web-tms"]);
    build(&mut model);
    wait(&mut model);
    assert_eq!(
        model.runs[1]
            .jobs
            .iter()
            .map(|j| j.status)
            .collect::<Vec<_>>(),
        vec![Status::Success, Status::Failed, Status::Skipped]
    );
    assert!(model.runs[1].jobs[2].task.is_none());
    assert_eq!(
        model.retry_ids(),
        BTreeSet::from(["web-bms".into(), "web-tms".into()])
    );
    let fail = model.runs[1].jobs[1].task.as_ref().unwrap();
    assert_eq!(fail.finished().unwrap()["exitCode"], 9);
    assert!(
        fs::read_to_string(fail.directory.join("stderr.log"))
            .unwrap()
            .contains("BUILD-ERROR")
    );
    fs::remove_file(root.join(".fail-build")).unwrap();
    click(&mut model, "retry");
    assert_eq!(model.pending.as_ref().unwrap().projects.len(), 2);
    click(&mut model, "confirmBuild");
    wait(&mut model);
    assert!(model.runs[2].status == Status::Success);
    assert_ne!(model.runs[2].jobs[1].task.as_ref().unwrap().id, first_task);
    model.open_logs(LogView::Job(1, 1));
    assert!(model.log_text().contains("BUILD-ERROR build:dev:bms"));
    click(&mut model, "all");
    build(&mut model);
    wait(&mut model);
    assert!(model.runs[3].plan.all);
    assert_eq!(model.runs[3].jobs.len(), 1);
    assert_eq!(model.runs[3].jobs[0].command, "build:dev");
    let shared = model.repos[0].latest;
    assert!(model.repos.iter().all(|r| r.latest == shared));
    let tms = root.join("apps/web-tms");
    let upstream = root.join("upstream.git");
    fs::create_dir(&upstream).unwrap();
    git(&upstream, &["init", "--bare", "--initial-branch=dev"]);
    git(
        &tms,
        &["remote", "add", "origin", upstream.to_str().unwrap()],
    );
    git(&tms, &["push", "-u", "origin", "dev"]);
    let author = root.join("author");
    git(
        &root,
        &[
            "clone",
            upstream.to_str().unwrap(),
            author.to_str().unwrap(),
        ],
    );
    fs::write(author.join("pulled.txt"), "new upstream commit").unwrap();
    git(&author, &["add", "pulled.txt"]);
    git(&author, &["commit", "-m", "upstream advance"]);
    git(&author, &["push"]);
    let previous_head = model.repos[3].project.head.clone();
    click(&mut model, "pull:web-tms");
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_ne!(model.repos[3].project.head, previous_head);
    assert!(model.repos[3].status.is_empty());
    assert!(model.repos[3].latest == shared);
    assert!(tms.join("pulled.txt").exists());
    fs::write(root.join(".fail-clean"), "").unwrap();
    let before = count(&root, ".build-invocations");
    build(&mut model);
    wait(&mut model);
    assert_eq!(count(&root, ".build-invocations"), before);
    assert!(
        model.runs[4]
            .jobs
            .iter()
            .all(|j| j.status == Status::Skipped && j.task.is_none())
    );
    assert_eq!(
        model.runs[4].prepare.as_ref().unwrap().finished().unwrap()["exitCode"],
        13
    );
    fs::remove_file(root.join(".fail-clean")).unwrap();
    let cleans = count(&root, ".clean-invocations");
    git(
        &root.join("apps/web-tms"),
        &["checkout", "-b", "changed-elsewhere"],
    );
    build(&mut model);
    wait(&mut model);
    assert_eq!(count(&root, ".clean-invocations"), cleans);
    assert_eq!(count(&root, ".build-invocations"), before);
    assert!(model.error.contains("分支或提交已变化"), "{}", model.error);
    view::document(&template, &model).unwrap();
}

#[test]
fn large_branch_response_refresh_failure_and_native_layout() {
    let (root, mut model, template) = fixture();
    model.scan();
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    let cwd = root.join("apps/web-bms");
    let head = git(&cwd, &["rev-parse", "HEAD"]);
    let mut child = Command::new("git")
        .args(["update-ref", "--stdin"])
        .current_dir(&cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    {
        let mut input = child.stdin.take().unwrap();
        for n in 0..650 {
            writeln!(input,"create refs/heads/feature/a-very-long-branch-name-for-stream-validation-{n:04} {head}").unwrap();
        }
    }
    assert!(child.wait().unwrap().success());
    click(&mut model, "fetch:web-bms");
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_eq!(model.repos[1].branches.len(), 652);
    let tasks = model.session_tasks.len();
    git(&cwd, &["branch", "feature/added-after-cache"]);
    for _ in 0..3 {
        send(&mut model, "target:web-bms", "open", Value::Null);
        assert!(model.idle());
    }
    assert_eq!(model.session_tasks.len(), tasks);
    assert!(
        !model.repos[1]
            .branches
            .contains(&"feature/added-after-cache".into())
    );
    click(&mut model, "fetch:web-bms");
    wait(&mut model);
    assert!(
        model.repos[1]
            .branches
            .contains(&"feature/added-after-cache".into())
    );
    let task = model.session_tasks.last().unwrap();
    assert!(task.byte_count(false) > 16_384);
    send(
        &mut model,
        "target:web-bms",
        "change",
        json!("feature/native"),
    );
    wait(&mut model);
    // Break only the disposable repository's Git metadata; preserve the current branch on fetch failure.
    fs::rename(cwd.join(".git"), cwd.join(".git-saved")).unwrap();
    click(&mut model, "fetch:web-bms");
    wait(&mut model);
    assert!(model.repos[1].refresh_error);
    assert_eq!(model.repos[1].project.branch, "feature/native");
    assert_eq!(model.repos[1].branches.len(), 653);
    let mut failed = view::document_json(&template, &model).unwrap();
    assert!(
        view::node_mut(&mut failed["root"], "target:web-bms").unwrap()["options"]["value"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["value"] == "feature/native")
    );
    fs::rename(cwd.join(".git-saved"), cwd.join(".git")).unwrap();
    click(&mut model, "fetch:web-bms");
    wait(&mut model);
    assert!(!model.repos[1].refresh_error);
    assert_eq!(model.repos[1].project.branch, "feature/native");
    for width in [1120.0, 1440.0] {
        let doc = view::document(&template, &model).unwrap();
        let tree = desktop_app_engine::component::Tree::new(&doc.root);
        let resolved = tree.resolve(&doc.state).unwrap();
        let mut scroll = vec![desktop_app_engine::layout::Size::default(); tree.nodes.len()];
        struct Measure;
        impl desktop_app_engine::layout::TextMeasure for Measure {
            fn text(&self, text: &str, _: f32, font: f32) -> desktop_app_engine::layout::Size {
                desktop_app_engine::layout::Size {
                    w: text.chars().count() as f32 * font * 0.6,
                    h: font * 1.5,
                }
            }
        }
        let boxes = desktop_app_engine::layout::arrange(
            &tree,
            &resolved,
            desktop_app_engine::layout::Rect {
                x: 0.0,
                y: 0.0,
                w: width,
                h: 900.0,
            },
            &mut scroll,
            &Measure,
            1.0,
        );
        let find =
            |id: &str| boxes.boxes[tree.nodes.iter().position(|n| n.spec.id == id).unwrap()].rect;
        let row = find("repo:web-bms");
        let pack = find("build:web-bms");
        let logs = find("logs:web-bms");
        assert!(logs.x >= pack.x + pack.w);
        assert!((pack.y - logs.y).abs() < 1.0);
        assert!(row.h <= 54.0);
        assert!((find("repoCard").h - find("logCard").h).abs() < 1.0);
        assert!(
            (find("repoCard").y + find("repoCard").h - (find("status").y - 16.0)).abs() < 1.0,
            "panels must fill available height: {:?}, {:?}",
            find("repoCard"),
            find("status")
        );
        assert!(find("repoCard").x < find("logCard").x);
    }
    for n in 0..20 {
        git(
            &root.join("apps/web-admin"),
            &["branch", &format!("visible-{n:02}")],
        );
    }
    model.repos[0].branches.clear();
    let context = eframe::egui::Context::default();
    desktop_app_engine::gui::install_chinese_font(&context).unwrap();
    let mut app = desktop_app_engine::gui_v02::DashboardApp::embedded(
        view::document(&template, &model).unwrap(),
    )
    .unwrap();
    let output = context.run(
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(1440.0, 900.0),
            )),
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    );
    assert!(!output.shapes.is_empty());
    for shape in &output.shapes {
        if let eframe::egui::epaint::Shape::Text(t) = &shape.shape
            && [
                "切换",
                "Fetch",
                "Pull",
                "打包",
                "查看日志",
                "全选",
                "清空选择",
                "复制",
                "清空显示",
                "日志目录",
            ]
            .contains(&t.galley.job.text.as_str())
        {
            assert_eq!(
                t.galley.rows.len(),
                1,
                "{} must remain on one line",
                t.galley.job.text
            );
        }
    }
    let pos = text_position(&output, "dev").expect("selector displays the actual current branch");
    native_click(&context, &mut app, pos);
    let events = app.take_events();
    assert_eq!(events.iter().filter(|e| e.kind == "open").count(), 1);
    for event in events {
        model.handle(event);
    }
    assert!(model.idle(), "opening an uncached list must not fetch");
    native_click(&context, &mut app, pos);
    app.take_events();
    let fetch_pos = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            eframe::egui::epaint::Shape::Text(t)
                if t.galley.job.text == "Fetch" && (t.pos.y - pos.y).abs() < 16.0 =>
            {
                Some(t.pos + t.galley.size() * 0.5)
            }
            _ => None,
        })
        .expect("business row Fetch button is rendered");
    native_click(&context, &mut app, fetch_pos);
    for event in app.take_events() {
        model.handle(event);
    }
    assert!(matches!(model.active,Some(Active::Branches(ref id)) if id=="web-admin"));
    app.replace_description(view::document(&template, &model).unwrap())
        .unwrap();
    native_frame(&context, &mut app, vec![]);
    native_click(&context, &mut app, pos);
    app.take_events();
    let loading = native_frame(&context, &mut app, vec![]);
    assert!(text_position(&loading, "刷新中…").is_some());
    let mut loading_doc = view::document_json(&template, &model).unwrap();
    assert!(view::node_mut(&mut loading_doc["root"], "status:web-admin").is_none());
    assert!(app.take_events().iter().all(|e| e.kind != "open"));
    wait(&mut model);
    app.replace_description(view::document(&template, &model).unwrap())
        .unwrap();
    let output = native_frame(&context, &mut app, vec![]);
    assert!(text_position(&output, "刷新中…").is_none());
    let count = output
        .shapes
        .iter()
        .filter(|s| {
            let eframe::egui::epaint::Shape::Text(t) = &s.shape else {
                return false;
            };
            s.clip_rect.height() > 100.0
                && model.repos[0].branches.contains(&t.galley.job.text)
                && s.clip_rect
                    .contains_rect(t.galley.rect.translate(t.pos.to_vec2()))
        })
        .count();
    assert_eq!(
        count, 10,
        "real workbench popup must grow from loading to ten visible branches"
    );
    let pos = text_position(&output, "feature/native").expect("refreshed popup stays open");
    native_click(&context, &mut app, pos);
    let events = app.take_events();
    assert!(
        events
            .iter()
            .any(|e| e.kind == "change" && e.payload == "feature/native")
    );
    for event in events {
        model.handle(event);
    }
    assert_eq!(model.repos[0].target, "feature/native");
    assert_eq!(model.repos[0].project.branch, "dev");
    wait(&mut model);
    assert_eq!(model.repos[0].project.branch, "feature/native");
    assert!(model.repos[0].target.is_empty());
    let tasks = model.session_tasks.len();
    send(&mut model, "target:web-admin", "open", Value::Null);
    assert!(model.idle());
    assert_eq!(model.session_tasks.len(), tasks);
}

#[test]
fn connection_settings_persist_and_discard_previous_workspace_state() {
    let (root, mut model, _) = fixture();
    let config = root.join("connection.dat");
    model.config_path = Some(config.clone());
    model.scan();
    wait(&mut model);
    click(&mut model, "all");
    build(&mut model);
    wait(&mut model);
    assert!(!model.runs.is_empty());
    let connection = model.connection.clone();
    click(&mut model, "settings");
    send(&mut model, "hostInput", "change", json!("-invalid"));
    click(&mut model, "saveSettings");
    assert!(model.settings_open);
    assert!(model.connection == connection);
    assert_eq!(model.runs.len(), 1);
    send(&mut model, "hostInput", "change", json!(connection.host));
    send(
        &mut model,
        "rootInput",
        "change",
        json!(format!("{}/missing", connection.root)),
    );
    click(&mut model, "saveSettings");
    assert!(!model.settings_open);
    assert!(model.repos.is_empty());
    assert!(model.selected.is_empty());
    assert!(model.runs.is_empty());
    assert!(model.log_view == LogView::Session);
    wait(&mut model);
    assert!(!model.error.is_empty());
    assert!(model.repos.is_empty());
    let persisted = settings::load(&config).unwrap().unwrap();
    assert_eq!(persisted, model.connection);
    assert!(!root.join("app.json").exists());
    click(&mut model, "settings");
    send(&mut model, "rootInput", "change", json!(connection.root));
    click(&mut model, "saveSettings");
    wait(&mut model);
    assert!(model.error.is_empty(), "{}", model.error);
    assert_eq!(model.repos.len(), 4);
    assert!(model.runs.is_empty());
}

#[test]
fn connection_settings_reject_corruption_and_preserve_previous_value_on_invalid_save() {
    let (root, model, _) = fixture();
    let path = root.join("user-settings/connection.dat");
    assert!(settings::load(&path).unwrap().is_none());
    settings::save(&path, &model.connection).unwrap();
    let saved = fs::read(&path).unwrap();
    let invalid = Connection {
        host: "-invalid".into(),
        root: "/workspace".into(),
    };
    assert!(settings::save(&path, &invalid).is_err());
    assert_eq!(fs::read(&path).unwrap(), saved);
    let next = Connection {
        host: "another-host".into(),
        root: "/different/root".into(),
    };
    settings::save(&path, &next).unwrap();
    assert_eq!(settings::load(&path).unwrap(), Some(next));
    fs::write(&path, &saved[..saved.len() - 1]).unwrap();
    assert!(settings::load(&path).is_err());
    fs::write(&path, [saved.as_slice(), b"unexpected"].concat()).unwrap();
    assert!(settings::load(&path).is_err());
}

fn native_frame(
    ctx: &eframe::egui::Context,
    app: &mut desktop_app_engine::gui_v02::DashboardApp,
    events: Vec<eframe::egui::Event>,
) -> eframe::egui::FullOutput {
    use eframe::egui;
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1440.0, 900.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| app.draw(ctx),
    )
}
fn text_position(output: &eframe::egui::FullOutput, text: &str) -> Option<eframe::egui::Pos2> {
    output.shapes.iter().find_map(|s| match &s.shape {
        eframe::egui::epaint::Shape::Text(t) if t.galley.job.text == text => {
            Some(t.pos + t.galley.size() * 0.5)
        }
        _ => None,
    })
}
fn native_click(
    ctx: &eframe::egui::Context,
    app: &mut desktop_app_engine::gui_v02::DashboardApp,
    pos: eframe::egui::Pos2,
) {
    use eframe::egui;
    for pressed in [true, false] {
        native_frame(
            ctx,
            app,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
}
