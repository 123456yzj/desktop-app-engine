//! State and ordered action execution for native and headless hosts.

mod checks;
mod events;
mod http;
mod legacy;
mod plugins;
#[cfg(test)]
mod tests;
use crate::{
    binding::{Binding, State, ValueType},
    compiler::{self, CompiledDescription, Metadata},
    component::Tree,
    v02::{Description, Step},
};
pub use legacy::Engine;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
#[derive(Debug, Clone)]
pub struct UiEvent {
    pub component_id: String,
    pub target_key: Option<String>,
    pub kind: String,
    pub payload: Value,
    pub sequence: u64,
    pub generation: u64,
}
pub struct Runtime {
    credential_store: Option<crate::credentials::Store>,
    http_tx: mpsc::Sender<(String, crate::http::Response)>,
    http_rx: mpsc::Receiver<(String, crate::http::Response)>,
    http_jobs: std::collections::HashSet<String>,
    http_due: BTreeMap<String, std::time::Instant>,
    http_started: bool,
    pub(crate) plugins: crate::plugin::Host,
    pub(crate) plugin_images: BTreeMap<String, crate::plugin::Image>,
    description: Description,
    metadata: Metadata,
    pub(crate) tree: Tree,
    state: State,
    version: u64,
    key_versions: BTreeMap<String, u64>,
    generation: u64,
    pub(crate) error: Option<String>,
    types: BTreeMap<String, ValueType>,
    last_sequence: Option<u64>,
    command_base: PathBuf,
    jobs: BTreeMap<String, Arc<AtomicBool>>,
    command_callbacks: BTreeMap<String, String>,
    completed_tx: mpsc::Sender<(String, Value)>,
    completed_rx: mpsc::Receiver<(String, Value)>,
}
impl Runtime {
    pub fn new(description: Description) -> Result<Self, String> {
        Self::from_compiled(compiler::compile(description)?)
    }
    pub fn from_compiled(compiled: CompiledDescription) -> Result<Self, String> {
        let CompiledDescription {
            description,
            metadata,
        } = compiled;
        let (completed_tx, completed_rx) = mpsc::channel();
        let (http_tx, http_rx) = mpsc::channel();
        let credential_store = description
            .credential_store
            .as_deref()
            .map(crate::credentials::Store::new)
            .transpose()?;
        let types = description
            .state
            .iter()
            .map(|(k, v)| Ok((k.clone(), ValueType::of(v)?)))
            .collect::<Result<_, String>>()?;
        Ok(Self {
            credential_store,
            http_tx,
            http_rx,
            http_jobs: Default::default(),
            http_due: Default::default(),
            http_started: false,
            plugins: crate::plugin::Host::default(),
            plugin_images: BTreeMap::new(),
            key_versions: description.state.keys().map(|k| (k.clone(), 1)).collect(),
            tree: Tree::new(&description.root, &metadata),
            state: description.state.clone(),
            description,
            metadata,
            types,
            version: 1,
            generation: 1,
            error: None,
            last_sequence: None,
            command_base: std::env::current_exe()
                .map_err(|e| e.to_string())?
                .parent()
                .ok_or("无法定位程序目录")?
                .to_owned(),
            jobs: BTreeMap::new(),
            command_callbacks: BTreeMap::new(),
            completed_tx,
            completed_rx,
        })
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn version(&self) -> u64 {
        self.version
    }
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub fn set_command_base(&mut self, base: PathBuf) {
        self.command_base = base;
    }
    pub(crate) fn command_base(&self) -> &std::path::Path {
        &self.command_base
    }
    pub(crate) fn key_versions(&self) -> &BTreeMap<String, u64> {
        &self.key_versions
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn advance_generation(&mut self, previous: u64) {
        self.generation = previous + 1;
    }
    pub(crate) fn has_plugins(&self) -> bool {
        !self.description.plugins.is_empty()
    }
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), String> {
        self.set_many(BTreeMap::from([(key.to_owned(), value)]))
    }
    /// Validate a complete response before committing any of its mappings.
    pub fn set_many(&mut self, updates: BTreeMap<String, Value>) -> Result<(), String> {
        for (key, value) in &updates {
            if !self
                .types
                .get(key)
                .ok_or_else(|| format!("未定义状态 {key:?}"))?
                .accepts(value)
            {
                return Err(format!("state.{key}: 赋值类型不匹配"));
            }
        }
        let mut candidate = self.description.clone();
        candidate.state = self.state.clone();
        candidate.state.extend(updates.clone());
        compiler::validate_flat(&candidate, &self.metadata)?;
        let changed: Vec<_> = updates
            .into_iter()
            .filter(|(key, value)| self.state.get(key) != Some(value))
            .collect();
        if !changed.is_empty() {
            self.version += 1;
            for (key, value) in changed {
                self.state.insert(key.clone(), value);
                self.key_versions.insert(key, self.version);
            }
        }
        Ok(())
    }
    /// Host updates use the same lexical scope as bindings authored on this component.
    pub fn set_from(&mut self, component: &str, key: &str, value: Value) -> Result<(), String> {
        let resolved = self
            .tree
            .nodes
            .iter()
            .find(|n| n.spec.id == component)
            .ok_or_else(|| format!("未定义组件 {component:?}"))?
            .metadata
            .state_names
            .get(key)
            .cloned()
            .ok_or_else(|| format!("组件 {component:?} 当前作用域未定义状态 {key:?}"))?;
        self.set(&resolved, value)
    }
    pub fn state_from(&self, component: &str, key: &str) -> Result<&Value, String> {
        let resolved = self
            .tree
            .nodes
            .iter()
            .find(|n| n.spec.id == component)
            .ok_or_else(|| format!("未定义组件 {component:?}"))?
            .metadata
            .state_names
            .get(key)
            .ok_or_else(|| format!("组件 {component:?} 当前作用域未定义状态 {key:?}"))?;
        self.state
            .get(resolved)
            .ok_or_else(|| format!("未定义状态 {key:?}"))
    }
    pub fn invoke(&mut self, id: &str) -> Result<(), String> {
        crate::logging::event("INFO", "action.invoke", serde_json::json!({"action":id}));
        let result = self.invoke_event(id, None, 0);
        if let Err(error) = &result {
            crate::logging::event(
                "ERROR",
                "action.failed",
                serde_json::json!({"action":id,"error":error}),
            );
        }
        result
    }
    fn invoke_event(
        &mut self,
        id: &str,
        payload: Option<Value>,
        depth: usize,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("模块事件调用超过 32 层，可能存在 emit 循环".into());
        }
        if let Some(sample) = &self
            .description
            .actions
            .get(id)
            .ok_or_else(|| format!("未定义动作 {id:?}"))?
            .event_payload
        {
            let data = payload
                .as_ref()
                .ok_or("eventPayload 动作必须通过模块事件调用")?;
            if !ValueType::of(sample)?.accepts(data) {
                return Err("eventPayload: 事件载荷类型不匹配".into());
            }
        }
        let snapshot = self.state.clone();
        let steps = self
            .description
            .actions
            .get(id)
            .ok_or_else(|| format!("未定义动作 {id:?}"))?
            .steps
            .clone();
        for (i, step) in steps.iter().enumerate() {
            let result = (|| {
                let (target, value) = match step {
                    Step::OpenLogs => {
                        crate::logging::open_viewer();
                        return Ok(());
                    }
                    Step::Request { source } => {
                        self.request_http_snapshot(source, &snapshot)?;
                        return Ok(());
                    }
                    Step::PluginCall {
                        target,
                        method,
                        args,
                        result_target,
                    } => {
                        let registration = self
                            .metadata
                            .nodes
                            .get(target)
                            .and_then(|n| n.registration.as_ref())
                            .ok_or("非插件组件")?;
                        let data = args.resolve_event(&snapshot, payload.as_ref())?.clone();
                        registration
                            .definition
                            .methods
                            .get(method)
                            .ok_or("未声明插件方法")?
                            .args
                            .validate(&data)?;
                        self.plugins.send(
                            target,
                            serde_json::json!({"op":"call","method":method,"args":data}),
                            result_target.clone(),
                            Some(method.clone()),
                        )?;
                        return Ok(());
                    }
                    Step::Emit {
                        event,
                        payload: value,
                    } => {
                        let data = value.resolve_event(&snapshot, payload.as_ref())?.clone();
                        let module = self.metadata.actions[id]
                            .module
                            .as_ref()
                            .ok_or("emit 缺少模块作用域")?;
                        let n = compiler::module_node(&self.description.root, module)
                            .ok_or("emit 模块不存在")?;
                        let sample = n
                            .emits
                            .get(event)
                            .ok_or_else(|| format!("未声明 emits 事件 {event:?}"))?;
                        if !ValueType::of(sample)?.accepts(&data) {
                            return Err(format!("emit {event:?}: 载荷类型不匹配"));
                        }
                        if let Some(handler) =
                            compiler::listener(n, &self.metadata, event, &self.state)?
                        {
                            self.invoke_event(&handler, Some(data), depth + 1)?;
                        }
                        return Ok(());
                    }
                    Step::Command {
                        program,
                        args,
                        cwd,
                        target,
                        timeout_ms,
                        on_complete,
                    } => {
                        if self.jobs.contains_key(target) {
                            return Err(format!("命令 {target:?} 已在运行"));
                        }
                        let request = crate::command::Request {
                            program: if std::path::Path::new(program).components().count() > 1 {
                                self.command_base
                                    .join(program)
                                    .to_string_lossy()
                                    .into_owned()
                            } else {
                                program.clone()
                            },
                            args: args
                                .iter()
                                .map(|a| a.resolve_event(&snapshot, payload.as_ref()))
                                .collect::<Result<_, _>>()?,
                            cwd: crate::command::working_directory(&self.command_base, cwd),
                            timeout_ms: *timeout_ms,
                        };
                        self.set(
                            target,
                            crate::command::result(
                                "running",
                                true,
                                -1,
                                String::new(),
                                String::new(),
                                String::new(),
                            ),
                        )?;
                        let cancel = Arc::new(AtomicBool::new(false));
                        let worker_cancel = cancel.clone();
                        let tx = self.completed_tx.clone();
                        let key = target.clone();
                        let label = format!("{id} / {target}");
                        std::thread::Builder::new()
                            .name("command".into())
                            .spawn(move || {
                                let _ = tx.send((
                                    key,
                                    crate::command::run_logged(
                                        request,
                                        worker_cancel,
                                        &label,
                                        crate::logging::global(),
                                    ),
                                ));
                            })
                            .map_err(|e| {
                                let _ = self.set(
                                    target,
                                    crate::command::result(
                                        "failed",
                                        false,
                                        -1,
                                        String::new(),
                                        String::new(),
                                        e.to_string(),
                                    ),
                                );
                                if let Some(callback) = on_complete {
                                    let _ = self.invoke(callback);
                                }
                                e.to_string()
                            })?;
                        self.jobs.insert(target.clone(), cancel);
                        if let Some(callback) = on_complete {
                            self.command_callbacks
                                .insert(target.clone(), callback.clone());
                        }
                        return Ok(());
                    }
                    Step::Set { target, value } => (
                        target,
                        value.resolve_event(&snapshot, payload.as_ref())?.clone(),
                    ),
                    Step::ParseJson { target, value } => {
                        let text = value
                            .resolve_event(&snapshot, payload.as_ref())?
                            .as_str()
                            .ok_or("parseJson value 必须为字符串")?;
                        let parsed =
                            serde_json::from_str(text).map_err(|e| format!("parseJson: {e}"))?;
                        (target, parsed)
                    }
                    Step::Concat { target, parts } => {
                        let mut result = String::new();
                        for p in parts {
                            let next = p
                                .resolve_event(&snapshot, payload.as_ref())?
                                .as_str()
                                .ok_or("concat 必须为字符串")?;
                            if result.len() + next.len() > crate::validation::MAX_STRING_BYTES {
                                return Err("concat 超过字符串大小限制".into());
                            }
                            result.push_str(next);
                        }
                        (target, Value::String(result))
                    }
                };
                self.set(target, value)
            })();
            result.map_err(|e| format!("actions.{id}.steps[{i}]: {e}"))?;
        }
        Ok(())
    }
    pub fn commands_running(&self) -> bool {
        !self.jobs.is_empty()
    }
    pub fn poll_commands(&mut self) {
        while let Ok((target, result)) = self.completed_rx.try_recv() {
            self.jobs.remove(&target);
            if let Err(e) = self.set(&target, result) {
                self.error = Some(e);
            }
            if let Some(callback) = self.command_callbacks.remove(&target)
                && let Err(e) = self.invoke(&callback)
            {
                self.error = Some(format!("命令 {target} onComplete: {e}"));
            }
        }
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        for cancel in self.jobs.values() {
            cancel.store(true, Ordering::Relaxed);
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        for _ in 0..self.jobs.len() {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if self.completed_rx.recv_timeout(remaining).is_err() {
                break;
            }
        }
    }
}
