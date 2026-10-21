//! Business contracts tested without remote servers or external fixture files.
use super::{
    backend::{Backend, Connection, Operation, Project, ROOT_ID},
    *,
};
use desktop_app_engine::{compiler, renderer::DashboardApp, services::logging};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "dae-business-{}-{}-{}",
            std::process::id(),
            logging::now_ms(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn repo(id: &str) -> Repo {
    Repo {
        project: Project {
            id: id.into(),
            name: id.into(),
            branch: "dev".into(),
            head: "abc123".into(),
            script: Some(format!("build:dev:{}", id.trim_start_matches("web-"))),
        },
        target: String::new(),
        branches: vec!["dev".into(), "feature/native".into()],
        refresh_error: false,
        status: String::new(),
        latest: None,
    }
}
fn model(workspace: &Workspace) -> Controller {
    let backend = Backend {
        base: workspace.0.clone(),
        session: logging::Session::new(&workspace.0.join("logs")).unwrap(),
        program: "ssh.exe".into(),
        prefix_args: vec![],
    };
    let mut model = Controller::new(
        Connection {
            host: "build-host".into(),
            root: "/workspace".into(),
        },
        backend,
        None,
    );
    model.repos = ["web-admin", "web-bms", "web-copilot", "web-tms"]
        .into_iter()
        .map(repo)
        .collect();
    model.root_repo = Some(repo(ROOT_ID));
    model.workspace_ready = true;
    model.all_script = true;
    model
}
fn template() -> Value {
    serde_json::from_str(include_str!("../../../../examples/vben-package.json")).unwrap()
}

#[test]
fn build_plan_preserves_snapshots_and_checks_workspace_readiness() {
    let workspace = Workspace::new();
    let mut model = model(&workspace);
    let ids = BTreeSet::from(["web-bms".into()]);
    let plan = model.plan(&ids).unwrap();
    assert!(!plan.all);
    assert_eq!(plan.projects[0].script.as_deref(), Some("build:dev:bms"));
    assert_eq!(plan.root.unwrap().head, "abc123");
    model.workspace_ready = false;
    assert!(model.plan(&ids).is_err());
    model.workspace_ready = true;
    model.repos[1].project.script = None;
    assert!(model.plan(&ids).is_err());
    assert!(model.plan(&BTreeSet::from([ROOT_ID.into()])).is_err());
}

#[test]
fn whole_workspace_requires_root_build_script_but_partial_build_does_not() {
    let workspace = Workspace::new();
    let mut model = model(&workspace);
    let ids = model.repos.iter().map(|r| r.project.id.clone()).collect();
    assert!(model.plan(&ids).unwrap().all);
    model.all_script = false;
    assert!(model.plan(&ids).is_err());
    model.root_repo = None;
    assert!(
        model
            .plan(&BTreeSet::from(["web-admin".into()]))
            .unwrap()
            .root
            .is_none()
    );
}

#[test]
fn generated_workbench_compiles_with_loading_error_and_modal_states() {
    let workspace = Workspace::new();
    let mut model = model(&workspace);
    let template = template();
    for state in 0..4 {
        model.active = (state == 1).then(|| Active::Branches("web-bms".into()));
        model.repos[1].refresh_error = state == 2;
        model.settings_open = state == 3;
        compiler::compile(view::document(&template, &model).unwrap()).unwrap();
    }
    model.settings_open = false;
    let mut document = view::document_json(&template, &model).unwrap();
    assert!(view::node_mut(&mut document["root"], "build:@root").is_none());
    assert_eq!(document["state"]["target_web-bms"], "dev");
}

#[test]
fn renderer_draws_workbench_at_minimum_and_desktop_sizes() {
    let workspace = Workspace::new();
    let model = model(&workspace);
    let context = eframe::egui::Context::default();
    desktop_app_engine::renderer::install_chinese_font(&context).unwrap();
    let mut app = DashboardApp::embedded(view::document(&template(), &model).unwrap()).unwrap();
    for size in [[1120.0, 650.0], [1440.0, 900.0]] {
        let output = context.run(
            eframe::egui::RawInput {
                screen_rect: Some(eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(size[0], size[1]),
                )),
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        assert!(!output.shapes.is_empty());
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, eframe::egui::epaint::Shape::Text(t) if t.galley.job.text == "Fetch")));
    }
}

#[test]
fn connection_settings_detect_corruption_and_preserve_valid_value() {
    let workspace = Workspace::new();
    let path = workspace.0.join("settings/connection.dat");
    let connection = Connection {
        host: "build-host".into(),
        root: "/workspace".into(),
    };
    assert!(settings::load(&path).unwrap().is_none());
    settings::save(&path, &connection).unwrap();
    let saved = std::fs::read(&path).unwrap();
    assert_eq!(settings::load(&path).unwrap(), Some(connection));
    assert!(
        settings::save(
            &path,
            &Connection {
                host: "-invalid".into(),
                root: "/workspace".into()
            }
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    std::fs::write(&path, &saved[..saved.len() - 1]).unwrap();
    assert!(settings::load(&path).is_err());
    std::fs::write(&path, [saved.as_slice(), b"unexpected"].concat()).unwrap();
    assert!(settings::load(&path).is_err());
}

#[test]
fn remote_commands_quote_workspace_and_separate_prepare_from_build() {
    let connection = Connection {
        host: "build-host".into(),
        root: "/workspace/it's a project".into(),
    };
    let prepare =
        backend::remote_script(&connection, &Operation::Prepare(vec![], false, None)).unwrap();
    let build = backend::remote_script(
        &connection,
        &Operation::Build(vec![], "build:dev:bms".into(), None),
    )
    .unwrap();
    assert!(prepare.contains("cd -- '/workspace/it'\"'\"'s a project'"));
    assert!(prepare.contains("stage=clean-turbo;"));
    assert!(!build.contains("stage=clean-turbo;"));
    assert!(build.contains("pnpm 'build:dev:bms'"));
    assert!(
        backend::remote_script(
            &Connection {
                host: "-bad".into(),
                root: "/workspace".into()
            },
            &Operation::Pull("web-bms".into())
        )
        .is_err()
    );
}
