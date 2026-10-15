pub mod backend;
pub mod settings;
#[cfg(test)]
mod tests;
pub mod view;

use backend::{
    Backend, Branches, Connection, Finished, Operation, Project, ROOT_ID, Scan, Snapshot,
    repository_label,
};
use desktop_app_engine::{interaction::UiEvent, logging};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Instant,
};

pub struct Repo {
    pub project: Project,
    pub target: String,
    pub branches: Vec<String>,
    pub refresh_error: bool,
    pub status: String,
    pub latest: Option<(usize, usize)>,
}
impl Repo {
    fn scanned(
        project: Project,
        previous: Option<Self>,
        refresh: Option<&backend::BranchRefresh>,
    ) -> Self {
        let mut repo = previous.unwrap_or(Self {
            project: project.clone(),
            target: String::new(),
            branches: vec![],
            refresh_error: false,
            status: String::new(),
            latest: None,
        });
        if repo.project.head != project.head || repo.project.branch != project.branch {
            repo.status.clear();
        }
        repo.project = project;
        repo.target.clear();
        if let Some(refresh) = refresh {
            if let Some(branches) = &refresh.branches {
                repo.branches = branches.clone();
                repo.refresh_error = false;
            } else if refresh.error.is_some() {
                repo.refresh_error = true;
            }
        }
        repo
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Waiting,
    Running,
    Success,
    Failed,
    Skipped,
}
impl Status {
    pub fn text(self) -> &'static str {
        match self {
            Self::Waiting => "等待构建",
            Self::Running => "正在构建",
            Self::Success => "构建成功",
            Self::Failed => "构建失败",
            Self::Skipped => "未执行",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub root: Option<Project>,
    pub projects: Vec<Project>,
    pub all: bool,
}
pub struct Job {
    pub ids: Vec<String>,
    pub command: String,
    pub status: Status,
    pub task: Option<Arc<logging::Task>>,
}
pub struct Run {
    pub id: String,
    pub plan: Plan,
    pub prepare: Option<Arc<logging::Task>>,
    pub jobs: Vec<Job>,
    pub status: Status,
    pub started: Instant,
    pub duration: Option<f32>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogView {
    Session,
    Run(usize),
    Job(usize, usize),
}
impl LogView {
    pub fn key(self) -> String {
        match self {
            Self::Session => "session".into(),
            Self::Run(r) => format!("run:{r}"),
            Self::Job(r, j) => format!("job:{r}:{j}"),
        }
    }
}
#[derive(Clone)]
pub enum Active {
    Scan,
    Branches(String),
    Change(String),
    Prepare(usize),
    Build(usize, usize),
}
pub struct Controller {
    pub connection: Connection,
    pub draft_host: String,
    pub draft_root: String,
    pub settings_open: bool,
    pub repos: Vec<Repo>,
    pub root_repo: Option<Repo>,
    pub workspace_ready: bool,
    pub selected: BTreeSet<String>,
    pub search: String,
    pub all_script: bool,
    pub active: Option<Active>,
    pub runs: Vec<Run>,
    pub pending: Option<Plan>,
    pub log_view: LogView,
    pub auto_scroll: bool,
    pub error: String,
    pub note: String,
    pub dirty: bool,
    backend: Backend,
    tx: mpsc::Sender<Finished>,
    rx: mpsc::Receiver<Finished>,
    cancel: Arc<AtomicBool>,
    session_tasks: Vec<Arc<logging::Task>>,
    clear_cursors: BTreeMap<(String, bool), u64>,
    config_path: Option<PathBuf>,
}
impl Controller {
    pub fn new(connection: Connection, backend: Backend, config_path: Option<PathBuf>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            draft_host: connection.host.clone(),
            draft_root: connection.root.clone(),
            settings_open: connection.validate().is_err(),
            connection,
            repos: vec![],
            root_repo: None,
            workspace_ready: false,
            selected: BTreeSet::new(),
            search: String::new(),
            all_script: false,
            active: None,
            runs: vec![],
            pending: None,
            log_view: LogView::Session,
            auto_scroll: true,
            error: String::new(),
            note: "就绪".into(),
            dirty: true,
            backend,
            tx,
            rx,
            cancel: Arc::new(AtomicBool::new(false)),
            session_tasks: vec![],
            clear_cursors: BTreeMap::new(),
            config_path,
        }
    }
    pub fn idle(&self) -> bool {
        self.active.is_none()
    }
    fn repo_mut(&mut self, id: &str) -> Option<&mut Repo> {
        if id == ROOT_ID {
            self.root_repo.as_mut()
        } else {
            self.repos.iter_mut().find(|r| r.project.id == id)
        }
    }
    pub fn visible(&self, repo: &Repo) -> bool {
        let q = self.search.to_lowercase();
        repo.project.id.to_lowercase().contains(&q)
            || repo.project.branch.to_lowercase().contains(&q)
    }
    fn launch(
        &mut self,
        operation: Operation,
        phase: Active,
        label: String,
    ) -> Result<Arc<logging::Task>, String> {
        if !self.idle() {
            return Err("任务执行中".into());
        }
        let task = self.backend.start(
            &self.connection,
            operation,
            &label,
            self.tx.clone(),
            self.cancel.clone(),
        )?;
        self.session_tasks.push(task.clone());
        self.active = Some(phase);
        self.note = label;
        self.dirty = true;
        Ok(task)
    }
    pub fn scan(&mut self) {
        self.scan_workspace(true);
    }
    pub fn scan_read_only(&mut self) {
        self.scan_workspace(false);
    }
    fn scan_workspace(&mut self, fetch_branches: bool) {
        if !self.idle() {
            return;
        }
        self.error.clear();
        self.workspace_ready = false;
        if let Err(e) = self.launch(
            Operation::Scan { fetch_branches },
            Active::Scan,
            if fetch_branches {
                "扫描工作区并刷新分支".into()
            } else {
                "扫描工作区".into()
            },
        ) {
            self.error = e;
            self.dirty = true;
        }
    }
    fn refresh(&mut self, id: &str) {
        if !self.idle() {
            return;
        }
        if let Some(repo) = self.repo_mut(id) {
            repo.refresh_error = false;
        } else {
            return;
        }
        if let Err(e) = self.launch(
            Operation::Branches(id.into()),
            Active::Branches(id.into()),
            format!("{} · 刷新分支", repository_label(id)),
        ) {
            self.error = e;
            self.dirty = true;
        }
    }
    pub fn plan(&self, ids: &BTreeSet<String>) -> Result<Plan, String> {
        if !self.workspace_ready {
            return Err("工作区状态未确认，请重新扫描后构建".into());
        }
        if self
            .root_repo
            .as_ref()
            .is_some_and(|r| r.project.branch.is_empty())
        {
            return Err("Turbo 根仓库分支未确认，请重新扫描".into());
        }
        if ids.is_empty() {
            return Err("请选择业务项目".into());
        }
        let projects: Vec<_> = self
            .repos
            .iter()
            .filter(|r| ids.contains(&r.project.id))
            .map(|r| r.project.clone())
            .collect();
        if projects.len() != ids.len() || projects.iter().any(|p| !p.available()) {
            return Err("项目或当前分支不可用于构建，请重新扫描".into());
        }
        let all = projects.len() == self.repos.len();
        if all && !self.all_script {
            return Err("根目录缺少 build:dev 脚本".into());
        }
        Ok(Plan {
            root: self.root_repo.as_ref().map(|r| r.project.clone()),
            projects,
            all,
        })
    }
    fn review(&mut self, ids: BTreeSet<String>) {
        if !self.idle() {
            return;
        }
        match self.plan(&ids) {
            Ok(plan) => {
                self.pending = Some(plan);
                self.error.clear();
            }
            Err(e) => self.error = e,
        }
        self.dirty = true;
    }
    pub fn start_build(&mut self) {
        if !self.idle() {
            return;
        }
        let Some(plan) = self.pending.take() else {
            return;
        };
        // Re-read current snapshots at confirmation; the SSH preflight verifies them again.
        let ids = plan.projects.iter().map(|p| p.id.clone()).collect();
        let plan = match self.plan(&ids) {
            Ok(p) => p,
            Err(e) => {
                self.error = e;
                self.dirty = true;
                return;
            }
        };
        let index = self.runs.len();
        let id = format!("BUILD-{:03}", index + 1);
        let jobs = if plan.all {
            vec![Job {
                ids: plan.projects.iter().map(|p| p.id.clone()).collect(),
                command: "build:dev".into(),
                status: Status::Waiting,
                task: None,
            }]
        } else {
            plan.projects
                .iter()
                .map(|p| Job {
                    ids: vec![p.id.clone()],
                    command: p.script.clone().unwrap(),
                    status: Status::Waiting,
                    task: None,
                })
                .collect()
        };
        self.runs.push(Run {
            id: id.clone(),
            plan: plan.clone(),
            prepare: None,
            jobs,
            status: Status::Running,
            started: Instant::now(),
            duration: None,
        });
        for (j, job) in self.runs[index].jobs.iter().enumerate() {
            for repo in &mut self.repos {
                if job.ids.contains(&repo.project.id) {
                    repo.latest = Some((index, j));
                    repo.status = "准备构建".into();
                }
            }
        }
        self.open_logs(LogView::Run(index));
        self.error.clear();
        match self.launch(
            Operation::Prepare(plan.projects, plan.all, plan.root),
            Active::Prepare(index),
            format!("{id} · 校验分支 / 清理 .turbo"),
        ) {
            Ok(task) => self.runs[index].prepare = Some(task),
            Err(e) => {
                self.error = e;
                self.skip_from(index, 0);
                self.finish(index, false);
            }
        }
    }
    fn start_job(&mut self, run: usize, index: usize) {
        let projects = self.runs[run].plan.projects.clone();
        let command = self.runs[run].jobs[index].command.clone();
        self.runs[run].jobs[index].status = Status::Running;
        self.sync_job(run, index);
        match self.launch(
            Operation::Build(projects, command.clone(), self.runs[run].plan.root.clone()),
            Active::Build(run, index),
            format!("{}/P{} · pnpm {command}", self.runs[run].id, index + 1),
        ) {
            Ok(task) => self.runs[run].jobs[index].task = Some(task),
            Err(e) => {
                self.runs[run].jobs[index].status = Status::Failed;
                self.sync_job(run, index);
                self.error = e;
                self.skip_from(run, index + 1);
                self.finish(run, false);
            }
        }
    }
    fn sync_job(&mut self, run: usize, index: usize) {
        let job = &self.runs[run].jobs[index];
        for repo in &mut self.repos {
            if job.ids.contains(&repo.project.id) {
                repo.status = if self.runs[run].plan.all && job.status == Status::Failed {
                    "整批构建失败".into()
                } else {
                    job.status.text().into()
                };
            }
        }
    }
    fn skip_from(&mut self, run: usize, from: usize) {
        for j in from..self.runs[run].jobs.len() {
            self.runs[run].jobs[j].status = Status::Skipped;
            self.sync_job(run, j);
        }
    }
    fn finish(&mut self, run: usize, ok: bool) {
        let item = &mut self.runs[run];
        item.status = if ok { Status::Success } else { Status::Failed };
        item.duration = Some(item.started.elapsed().as_secs_f32());
        self.note = format!("{} · {}", item.id, item.status.text());
        self.dirty = true;
    }
    pub fn poll(&mut self) -> bool {
        while let Ok(result) = self.rx.try_recv() {
            let Some(phase) = self.active.take() else {
                continue;
            };
            self.note = "就绪".into();
            self.dirty = true;
            match phase {
                Active::Scan => match result.json::<Scan>() {
                    Ok(scan) => {
                        let refresh_errors: Vec<_> = scan
                            .branch_lists
                            .iter()
                            .filter_map(|(id, refresh)| {
                                refresh.error.as_ref().map(|error| {
                                    format!("{} · 分支刷新失败: {error}", repository_label(id))
                                })
                            })
                            .collect();
                        self.root_repo = scan.root_project.map(|project| {
                            Repo::scanned(
                                project,
                                self.root_repo.take(),
                                scan.branch_lists.get(ROOT_ID),
                            )
                        });
                        let mut previous: BTreeMap<_, _> = std::mem::take(&mut self.repos)
                            .into_iter()
                            .map(|r| (r.project.id.clone(), r))
                            .collect();
                        self.repos = scan
                            .projects
                            .into_iter()
                            .map(|project| {
                                let previous = previous.remove(&project.id);
                                let refresh = scan.branch_lists.get(&project.id);
                                Repo::scanned(project, previous, refresh)
                            })
                            .collect();
                        self.all_script = scan.all_script;
                        self.workspace_ready = true;
                        self.selected.retain(|id| {
                            self.repos
                                .iter()
                                .any(|r| r.project.id == *id && r.project.available())
                        });
                        self.error = refresh_errors.join("\n");
                    }
                    Err(e) => self.error = e,
                },
                Active::Branches(id) => match result.json::<Branches>() {
                    Ok(branches) => {
                        let mut root_changed = false;
                        let mut unavailable = false;
                        if let Some(repo) = self.repo_mut(&id) {
                            if repo.project.head != branches.head
                                || repo.project.branch != branches.branch
                            {
                                repo.status.clear();
                                root_changed = id == ROOT_ID;
                            }
                            repo.project.branch = branches.branch;
                            repo.project.head = branches.head;
                            repo.branches = branches.branches;
                            repo.refresh_error = false;
                            if !repo.branches.contains(&repo.target) {
                                repo.target.clear();
                            }
                            unavailable = !repo.project.available();
                        }
                        if unavailable {
                            self.selected.remove(&id);
                        }
                        self.error.clear();
                        if root_changed {
                            self.scan_read_only();
                        }
                    }
                    Err(e) => {
                        if let Some(repo) = self.repo_mut(&id) {
                            repo.refresh_error = true;
                        }
                        self.error = e;
                    }
                },
                Active::Change(id) => {
                    let response = result.read_json::<Snapshot>();
                    let mut unknown = false;
                    let mut changed = false;
                    if let Some(repo) = self.repo_mut(&id) {
                        repo.target.clear();
                        match response {
                            Ok(s) => {
                                changed =
                                    repo.project.branch != s.branch || repo.project.head != s.head;
                                repo.project.branch = s.branch;
                                repo.project.head = s.head;
                                repo.status.clear();
                            }
                            Err(_) => {
                                repo.project.branch.clear();
                                repo.project.head.clear();
                                repo.status = "分支未确认".into();
                                unknown = true;
                            }
                        }
                    }
                    if unknown {
                        self.selected.remove(&id);
                    }
                    if id == ROOT_ID && (changed || unknown) {
                        self.workspace_ready = false;
                    }
                    if result.success() {
                        self.error.clear();
                        if id == ROOT_ID {
                            self.scan_read_only();
                        }
                    } else {
                        self.error = result.error();
                    }
                }
                Active::Prepare(run) => {
                    if result.success() {
                        for j in 0..self.runs[run].jobs.len() {
                            self.sync_job(run, j);
                        }
                        self.start_job(run, 0);
                    } else {
                        self.error = result.error();
                        self.skip_from(run, 0);
                        self.finish(run, false);
                    }
                }
                Active::Build(run, j) => {
                    self.runs[run].jobs[j].status = if result.success() {
                        Status::Success
                    } else {
                        Status::Failed
                    };
                    self.sync_job(run, j);
                    if result.success() {
                        if j + 1 < self.runs[run].jobs.len() {
                            self.start_job(run, j + 1);
                        } else {
                            self.finish(run, true);
                        }
                    } else {
                        self.error = result.error();
                        self.skip_from(run, j + 1);
                        self.finish(run, false);
                    }
                }
            }
        }
        self.dirty
    }
    pub fn open_logs(&mut self, view: LogView) {
        self.log_view = view;
        self.clear_cursors.clear();
        self.dirty = true;
    }
    pub fn log_tasks(&self) -> Vec<Arc<logging::Task>> {
        match self.log_view {
            LogView::Session => self.session_tasks.clone(),
            LogView::Run(r) => self
                .runs
                .get(r)
                .map(|run| {
                    run.prepare
                        .iter()
                        .cloned()
                        .chain(run.jobs.iter().filter_map(|j| j.task.clone()))
                        .collect()
                })
                .unwrap_or_default(),
            LogView::Job(r, j) => self
                .runs
                .get(r)
                .map(|run| {
                    run.prepare
                        .iter()
                        .cloned()
                        .chain(run.jobs.get(j).and_then(|j| j.task.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
    pub fn log_text(&self) -> String {
        let mut text = String::new();
        for task in self.log_tasks() {
            for stderr in [false, true] {
                if !stderr
                    && self.log_view == LogView::Session
                    && (task.label == "扫描工作区" || task.label.ends_with("刷新分支"))
                {
                    continue;
                }
                let end = task.byte_count(stderr);
                let start = self
                    .clear_cursors
                    .get(&(task.id.clone(), stderr))
                    .copied()
                    .unwrap_or(0);
                if start >= end {
                    continue;
                }
                let preview = task.preview(stderr);
                let tail_start = end.saturating_sub(preview.len() as u64);
                let mut offset =
                    start.saturating_sub(tail_start).min(preview.len() as u64) as usize;
                while !preview.is_char_boundary(offset) {
                    offset += 1;
                }
                text.push_str(&format!(
                    "[{} · {}]\n{}\n",
                    task.label,
                    if stderr { "stderr" } else { "stdout" },
                    &preview[offset..]
                ));
            }
        }
        if text.len() > 48 * 1024 {
            let mut start = text.len() - 48 * 1024;
            while !text.is_char_boundary(start) {
                start += 1;
            }
            text = text[start..].into();
        }
        text
    }
    pub fn retry_ids(&self) -> BTreeSet<String> {
        self.runs
            .last()
            .filter(|r| r.status == Status::Failed)
            .map(|r| {
                r.jobs
                    .iter()
                    .filter(|j| matches!(j.status, Status::Failed | Status::Skipped))
                    .flat_map(|j| j.ids.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn handle(&mut self, event: UiEvent) {
        let id = event.component_id.as_str();
        if id == "search" && event.kind == "change" {
            self.search = event.payload.as_str().unwrap_or_default().into();
            self.dirty = true;
            return;
        }
        if id == "logSource" && event.kind == "change" {
            let key = event.payload.as_str().unwrap_or_default();
            let view = if key == "session" {
                Some(LogView::Session)
            } else {
                self.runs
                    .iter()
                    .enumerate()
                    .flat_map(|(r, run)| {
                        std::iter::once(LogView::Run(r))
                            .chain((0..run.jobs.len()).map(move |j| LogView::Job(r, j)))
                    })
                    .find(|v| v.key() == key)
            };
            if let Some(v) = view {
                self.open_logs(v);
            }
            return;
        }
        if id == "autoScroll" {
            self.auto_scroll = !self.auto_scroll;
            self.dirty = true;
            return;
        }
        if id == "clearLogs" {
            for task in self.log_tasks() {
                for stream in [false, true] {
                    self.clear_cursors
                        .insert((task.id.clone(), stream), task.byte_count(stream));
                }
            }
            self.dirty = true;
            return;
        }
        if id == "logFolder" {
            let path = self
                .log_tasks()
                .last()
                .map(|t| t.directory.clone())
                .unwrap_or_else(|| self.backend.session.directory.clone());
            if let Err(e) = logging::open_directory(&path) {
                self.error = e;
            }
            self.dirty = true;
            return;
        }
        if let Some(repo) = id.strip_prefix("logs:") {
            if let Some((r, j)) = self
                .repos
                .iter()
                .find(|r| r.project.id == repo)
                .and_then(|r| r.latest)
            {
                self.open_logs(LogView::Job(r, j));
            }
            return;
        }
        if !self.idle() {
            return;
        }
        if let Some(repo) = id.strip_prefix("target:") {
            if event.kind == "change"
                && self.pending.is_none()
                && !self.settings_open
                && let Some(r) = self.repo_mut(repo)
            {
                let value = event.payload.as_str().unwrap_or_default();
                if value != r.project.branch && r.branches.iter().any(|v| v == value) {
                    r.target = value.into();
                    self.dirty = true;
                    if let Err(error) = self.launch(
                        Operation::Switch(repo.into(), value.into()),
                        Active::Change(repo.into()),
                        format!("{} · git checkout", repository_label(repo)),
                    ) {
                        self.repo_mut(repo).unwrap().target.clear();
                        self.error = error;
                    }
                }
            }
            return;
        }
        if self.pending.is_some() {
            match id {
                "confirmBuild" => self.start_build(),
                "cancelBuild" => {
                    self.pending = None;
                    self.dirty = true;
                }
                _ => {}
            }
            return;
        }
        match id {
            "hostInput" => self.draft_host = event.payload.as_str().unwrap_or_default().into(),
            "rootInput" => self.draft_root = event.payload.as_str().unwrap_or_default().into(),
            "settings" => {
                self.settings_open = true;
                self.draft_host = self.connection.host.clone();
                self.draft_root = self.connection.root.clone();
            }
            "cancelSettings" => self.settings_open = false,
            "saveSettings" => {
                if let Err(e) = self.save_connection() {
                    self.error = e;
                }
            }
            "scan" => self.scan(),
            "all" => {
                self.selected = self
                    .repos
                    .iter()
                    .filter(|r| r.project.available())
                    .map(|r| r.project.id.clone())
                    .collect()
            }
            "clear" => self.selected.clear(),
            "selectVisible" => {
                let visible: Vec<_> = self
                    .repos
                    .iter()
                    .filter(|r| self.visible(r) && r.project.available())
                    .map(|r| r.project.id.clone())
                    .collect();
                let all = visible.iter().all(|id| self.selected.contains(id));
                for id in visible {
                    if all {
                        self.selected.remove(&id);
                    } else {
                        self.selected.insert(id);
                    }
                }
            }
            "build" => self.review(self.selected.clone()),
            "retry" => self.review(self.retry_ids()),
            _ => {
                if let Some(id) = id.strip_prefix("choose:") {
                    if self
                        .repos
                        .iter()
                        .any(|r| r.project.id == id && r.project.available())
                        && !self.selected.remove(id)
                    {
                        self.selected.insert(id.into());
                    }
                } else if let Some(id) = id.strip_prefix("fetch:") {
                    self.refresh(id);
                } else if let Some(id) = id.strip_prefix("build:") {
                    self.review(BTreeSet::from([id.into()]));
                } else if let Some(id) = id.strip_prefix("pull:")
                    && let Err(e) = self.launch(
                        Operation::Pull(id.into()),
                        Active::Change(id.into()),
                        format!("{} · git pull", repository_label(id)),
                    )
                {
                    self.error = e;
                }
            }
        }
        self.dirty = true;
    }
    fn save_connection(&mut self) -> Result<(), String> {
        let connection = Connection {
            host: self.draft_host.trim().into(),
            root: self.draft_root.trim().into(),
        };
        connection.validate()?;
        if let Some(path) = &self.config_path {
            settings::save(path, &connection)?;
        }
        if self.connection != connection {
            self.connection = connection;
            self.repos.clear();
            self.root_repo = None;
            self.workspace_ready = false;
            self.selected.clear();
            self.runs.clear();
            self.session_tasks.clear();
            self.open_logs(LogView::Session);
            self.search.clear();
        }
        self.settings_open = false;
        self.scan();
        Ok(())
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
