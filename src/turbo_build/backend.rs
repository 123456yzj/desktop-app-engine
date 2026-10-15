use desktop_app_engine::{command, logging};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub const ROOT_ID: &str = "@root";
pub fn repository_label(id: &str) -> &str {
    if id == ROOT_ID { "Turbo 根仓库" } else { id }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    pub host: String,
    pub root: String,
}
impl Connection {
    pub fn validate(&self) -> Result<(), String> {
        if self.host.is_empty()
            || self.host.starts_with('-')
            || self
                .host
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err("SSH 地址无效".into());
        }
        if !self.root.starts_with('/') || self.root.chars().any(char::is_control) {
            return Err("工作区必须是 Linux 绝对路径".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub branch: String,
    pub head: String,
    pub script: Option<String>,
}
impl Project {
    pub fn available(&self) -> bool {
        !self.branch.is_empty() && self.script.is_some()
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    pub root_project: Option<Project>,
    pub projects: Vec<Project>,
    pub all_script: bool,
    pub branch_lists: std::collections::BTreeMap<String, BranchRefresh>,
}
#[derive(Deserialize)]
pub struct BranchRefresh {
    pub branches: Option<Vec<String>>,
    pub error: Option<String>,
}
#[derive(Deserialize)]
pub struct Branches {
    pub branch: String,
    pub head: String,
    pub branches: Vec<String>,
}
#[derive(Deserialize)]
pub struct Snapshot {
    pub branch: String,
    pub head: String,
}
#[derive(Clone, Debug)]
pub enum Operation {
    Scan { fetch_branches: bool },
    Branches(String),
    Switch(String, String),
    Pull(String),
    Prepare(Vec<Project>, bool, Option<Project>),
    Build(Vec<Project>, String, Option<Project>),
}
impl Operation {
    pub fn label(&self) -> String {
        match self {
            Self::Scan { .. } => "扫描工作区".into(),
            Self::Branches(id) => format!("{} · 刷新分支", repository_label(id)),
            Self::Switch(id, _) => format!("{} · git checkout", repository_label(id)),
            Self::Pull(id) => format!("{} · git pull", repository_label(id)),
            Self::Prepare(..) => "校验分支并清理 .turbo".into(),
            Self::Build(_, script, _) => format!("pnpm {script}"),
        }
    }
}
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
pub fn remote_script(connection: &Connection, operation: &Operation) -> Result<String, String> {
    connection.validate()?;
    let payload = match operation {
        Operation::Scan { fetch_branches } => {
            json!({"operation":"scan","fetchBranches":fetch_branches})
        }
        Operation::Branches(id) => json!({"operation":"branches","id":id}),
        Operation::Switch(id, branch) => json!({"operation":"switch","id":id,"branch":branch}),
        Operation::Pull(id) => json!({"operation":"pull","id":id}),
        Operation::Prepare(projects, all, root) => {
            json!({"operation":"verify","projects":projects,"rootProject":root,"script":if *all {Some("build:dev")}else{None}})
        }
        Operation::Build(projects, script, root) => {
            json!({"operation":"verify","projects":projects,"rootProject":root,"script":script})
        }
    };
    let report =
        "rc=$?; set +e; printf '[vben] stage=%s exit=%s\\n' \"$stage\" \"$rc\" >&2; exit \"$rc\"";
    let mut script = format!(
        "set -euo pipefail; export GIT_TERMINAL_PROMPT=0 GIT_MERGE_AUTOEDIT=no; stage=workspace; trap {} EXIT; cd -- {}; stage=operation; node -e {} -- {}; ",
        quote(report),
        quote(&connection.root),
        quote(include_str!("remote.js")),
        quote(&payload.to_string())
    );
    match operation {
        Operation::Prepare(..)=>script.push_str("stage=clean-turbo; printf '[clean-turbo] rm -rf -- ./.turbo\\n' >&2; rm -rf -- ./.turbo; "),
        Operation::Build(_,name,_)=>{ script.push_str(&format!("stage=build; printf '%s\\n' {} >&2; pnpm {}; ",quote(&format!("$ pnpm {name}")),quote(name))); },
        _=>{}
    }
    Ok(script)
}
pub struct Backend {
    pub base: PathBuf,
    pub session: Arc<logging::Session>,
    pub program: String,
    pub prefix_args: Vec<String>,
}
pub struct Finished {
    pub operation: Operation,
    pub result: Value,
    pub task: Arc<logging::Task>,
}
impl Backend {
    pub fn request(
        &self,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<command::Request, String> {
        let script = remote_script(connection, operation)?;
        let mut args = self.prefix_args.clone();
        args.extend(vec![
            "-T".into(),
            "-o".into(),
            "BatchMode=yes".into(),
            "-o".into(),
            "ConnectTimeout=15".into(),
            "--".into(),
            connection.host.clone(),
            format!("bash -lc {}", quote(&script)),
        ]);
        Ok(command::Request {
            program: self.program.clone(),
            args,
            cwd: self.base.clone(),
            timeout_ms: if matches!(operation, Operation::Build(..)) {
                3_600_000
            } else if matches!(
                operation,
                Operation::Scan {
                    fetch_branches: true
                }
            ) {
                600_000
            } else {
                120_000
            },
        })
    }
    pub fn start(
        &self,
        connection: &Connection,
        operation: Operation,
        label: &str,
        tx: std::sync::mpsc::Sender<Finished>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Arc<logging::Task>, String> {
        let request = self.request(connection, &operation)?;
        let task = self
            .session
            .task(label, &request.program, &request.cwd, request.timeout_ms)?;
        let handle = task.clone();
        std::thread::spawn(move || {
            let result = command::run_with_task(request, cancel, Some(handle.clone()));
            let _ = tx.send(Finished {
                operation,
                result,
                task: handle,
            });
        });
        Ok(task)
    }
}
impl Finished {
    pub fn success(&self) -> bool {
        self.result["status"] == "succeeded"
            && self.result["exitCode"] == 0
            && self
                .task
                .finished()
                .is_some_and(|r| r["outputComplete"] == true)
    }
    pub fn error(&self) -> String {
        let text = self.task.preview(true);
        let error = self.result["error"].as_str().unwrap_or("");
        format!(
            "{} · {}{}{}",
            self.operation.label(),
            self.result["status"].as_str().unwrap_or("失败"),
            if error.is_empty() { "" } else { " · " },
            if error.is_empty() {
                text.as_str()
            } else {
                error
            }
        )
    }
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        if !self.success() {
            return Err(self.error());
        }
        self.read_json()
    }
    pub fn read_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        let path = self.task.directory.join("stdout.log");
        let size = std::fs::metadata(&path).map_err(|e| e.to_string())?.len();
        if size > 4 * 1024 * 1024 {
            return Err("远端响应超过 4 MiB".into());
        }
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        serde_json::from_slice(&data).map_err(|e| format!("远端响应无法解析: {e}"))
    }
}
