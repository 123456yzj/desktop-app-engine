//! Ordered commit is the only place that mutates business state or invokes actions.
use crate::{
    binding::{Binding, State, ValueType},
    component::Tree,
    v02::{Description, Step},
};
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
    pub credential_store: Option<crate::credentials::Store>,
    http_tx: mpsc::Sender<(String, crate::http::Response)>,
    http_rx: mpsc::Receiver<(String, crate::http::Response)>,
    http_jobs: std::collections::HashSet<String>,
    http_due: BTreeMap<String, std::time::Instant>,
    http_started: bool,
    pub plugins: crate::plugin::Host,
    pub plugin_images: BTreeMap<String, crate::plugin::Image>,
    pub description: Description,
    pub tree: Tree,
    pub state: State,
    pub version: u64,
    pub key_versions: BTreeMap<String, u64>,
    pub generation: u64,
    pub error: Option<String>,
    types: BTreeMap<String, ValueType>,
    last_sequence: Option<u64>,
    pub command_base: PathBuf,
    jobs: BTreeMap<String, Arc<AtomicBool>>,
    command_callbacks: BTreeMap<String, String>,
    completed_tx: mpsc::Sender<(String, Value)>,
    completed_rx: mpsc::Receiver<(String, Value)>,
}
impl Runtime {
    pub fn new(description: Description) -> Result<Self, String> {
        let description = crate::scope::compile(description)?;
        crate::v02::validate_flat(&description)?;
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
            tree: Tree::new(&description.root),
            state: description.state.clone(),
            description,
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
        crate::v02::validate_flat(&candidate)?;
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
            .spec
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
            .spec
            .state_names
            .get(key)
            .ok_or_else(|| format!("组件 {component:?} 当前作用域未定义状态 {key:?}"))?;
        self.state
            .get(resolved)
            .ok_or_else(|| format!("未定义状态 {key:?}"))
    }
    pub fn commit(&mut self, mut events: Vec<UiEvent>) {
        events.sort_by_key(|e| e.sequence);
        for e in events {
            if e.generation != self.generation
                || self.last_sequence.is_some_and(|s| e.sequence <= s)
            {
                continue;
            }
            self.last_sequence = Some(e.sequence);
            let Some(node) = self.tree.nodes.iter().find(|n| n.spec.id == e.component_id) else {
                continue;
            };
            let active = |b: &Option<Binding>| {
                b.as_ref().is_none_or(|b| {
                    b.resolve(&self.state)
                        .is_ok_and(|v| v.as_bool() == Some(true))
                })
            };
            let mut parent = node.parent;
            let mut ancestors_active = true;
            while let Some(id) = parent {
                let n = &self.tree.nodes[id];
                ancestors_active &= active(&n.spec.visible) && active(&n.spec.enabled);
                parent = n.parent;
            }
            if !ancestors_active || !active(&node.spec.visible) || !active(&node.spec.enabled) {
                continue;
            }
            let action = node.spec.events.get(&e.kind).cloned();
            let plugin_contract = node
                .spec
                .registration
                .as_ref()
                .and_then(|r| r.definition.events.get(&e.kind))
                .cloned();
            if node.spec.kind == crate::v02::Kind::Plugin {
                let Some(contract) = &plugin_contract else {
                    continue;
                };
                if let Err(err) = contract.validate(&e.payload) {
                    self.error = Some(err);
                    continue;
                }
            }
            let action_names = node.spec.action_names.clone();
            if node.spec.kind == crate::v02::Kind::Select && e.kind == "change" {
                let allowed =
                    crate::v02::select_options(&node.spec, &self.state).is_ok_and(|options| {
                        options
                            .iter()
                            .any(|o| !o.disabled && e.payload.as_str() == Some(o.value.as_str()))
                    });
                if !allowed {
                    self.error = Some("select.change 必须选择可用选项".into());
                    continue;
                }
            }
            let binding = if e.kind == "change" {
                node.spec.value.clone()
            } else {
                None
            };
            self.error = None;
            if let Some(Binding::State { key, .. }) = binding
                && let Err(err) = self.set(&key, e.payload.clone())
            {
                self.error = Some(err);
                continue;
            }
            if let Some(action) = action {
                let result = action
                    .resolve(&self.state)
                    .and_then(|id| match &action {
                        crate::v02::EventAction::Fixed(_) => Ok(id.to_owned()),
                        crate::v02::EventAction::Binding(_) => action_names
                            .get(id)
                            .cloned()
                            .ok_or_else(|| format!("当前作用域未定义动作 {id:?}")),
                    })
                    .and_then(|id| {
                        if plugin_contract.is_some() {
                            self.invoke_event(&id, Some(e.payload.clone()), 0)
                        } else {
                            self.invoke(&id)
                        }
                    });
                if let Err(err) = result {
                    self.error = Some(format!(
                        "组件 {:?} events.{}: {err}",
                        e.component_id, e.kind
                    ));
                }
            }
        }
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
                        let n = crate::v02::module_node(&self.description.root, target)
                            .ok_or("插件组件不存在")?;
                        let registration = n.registration.as_ref().ok_or("非插件组件")?;
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
                        module,
                    } => {
                        let data = value.resolve_event(&snapshot, payload.as_ref())?.clone();
                        let module = module.as_ref().ok_or("emit 缺少模块作用域")?;
                        let n = crate::v02::module_node(&self.description.root, module)
                            .ok_or("emit 模块不存在")?;
                        let sample = n
                            .emits
                            .get(event)
                            .ok_or_else(|| format!("未声明 emits 事件 {event:?}"))?;
                        if !ValueType::of(sample)?.accepts(&data) {
                            return Err(format!("emit {event:?}: 载荷类型不匹配"));
                        }
                        if let Some(handler) = crate::v02::listener(n, event, &self.state)? {
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
    pub fn request_http(&mut self, source: &str) -> Result<bool, String> {
        self.request_http_snapshot(source, &self.state.clone())
    }
    fn request_http_snapshot(&mut self, id: &str, snapshot: &State) -> Result<bool, String> {
        let source = self
            .description
            .data_sources
            .get(id)
            .ok_or_else(|| format!("未定义 dataSource {id:?}"))?
            .clone();
        if self.http_jobs.contains(id) || !source.is_enabled(snapshot)? {
            return Ok(false);
        }
        let request = source.resolve_with_store(snapshot, self.credential_store.clone())?;
        let updated = self.state[&source.status_target]["updatedAt"]
            .as_u64()
            .unwrap_or(0);
        self.set(
            &source.status_target,
            crate::http::status("loading", true, 0, "", updated),
        )?;
        let tx = self.http_tx.clone();
        let name = id.to_owned();
        if let Err(e) = std::thread::Builder::new()
            .name("http".into())
            .spawn(move || {
                let _ = tx.send((name, crate::http::run(request)));
            })
        {
            self.set(
                &source.status_target,
                crate::http::status("failed", false, 0, "无法启动请求线程", updated),
            )?;
            return Err(e.to_string());
        }
        self.http_jobs.insert(id.to_owned());
        self.http_due.remove(id);
        Ok(true)
    }
    /// Called by GUI frames or headless hosts; constructing/checking a document never sends HTTP.
    pub fn poll_http(&mut self) {
        while let Ok((id, response)) = self.http_rx.try_recv() {
            self.http_jobs.remove(&id);
            let source = self.description.data_sources[&id].clone();
            let old_updated = self.state[&source.status_target]["updatedAt"]
                .as_u64()
                .unwrap_or(0);
            let result = (|| {
                let body = response.result?;
                let mut updates = source.map(&body)?;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_millis() as u64;
                updates.insert(
                    source.status_target.clone(),
                    crate::http::status("success", false, response.code, "", now),
                );
                self.set_many(updates)
            })();
            if let Err(error) = result
                && let Err(e) = self.set(
                    &source.status_target,
                    crate::http::status("failed", false, response.code, &error, old_updated),
                )
            {
                self.error = Some(e);
            }
            if let Some(interval) = source.interval_ms {
                self.http_due.insert(
                    id,
                    std::time::Instant::now() + std::time::Duration::from_millis(interval),
                );
            }
        }
        let now = std::time::Instant::now();
        if !self.http_started {
            self.http_started = true;
            for (id, source) in &self.description.data_sources {
                if source.on_load {
                    self.http_due.insert(id.clone(), now);
                } else if let Some(ms) = source.interval_ms {
                    self.http_due
                        .insert(id.clone(), now + std::time::Duration::from_millis(ms));
                }
            }
        }
        let due: Vec<_> = self
            .http_due
            .iter()
            .filter(|(_, time)| **time <= now)
            .map(|(id, _)| id.clone())
            .collect();
        for id in due {
            match self.request_http(&id) {
                Ok(true) => {}
                Ok(false) => {
                    self.http_due
                        .insert(id, now + std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    self.error = Some(e);
                    self.http_due
                        .insert(id, now + std::time::Duration::from_secs(1));
                }
            }
        }
    }
    pub fn http_active(&self) -> bool {
        !self.http_jobs.is_empty()
            || self
                .description
                .data_sources
                .values()
                .any(|s| s.on_load || s.interval_ms.is_some())
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
    pub fn sync_plugins(&mut self, visible: Vec<(String, u32, u32)>) -> Result<(), String> {
        for (id, width, height) in &visible {
            let n = crate::v02::module_node(&self.description.root, id).ok_or("组件不存在")?;
            self.plugins.sync(
                id,
                n.registration.as_ref().ok_or("缺少插件注册")?,
                crate::v02::plugin_props(n, &self.state)?,
                *width,
                *height,
            )?;
        }
        self.plugins
            .retain(&visible.into_iter().map(|(id, _, _)| id).collect::<Vec<_>>());
        Ok(())
    }
    pub fn poll_plugins(&mut self) {
        self.plugins.poll();
        for completion in std::mem::take(&mut self.plugins.completed) {
            let result = (|| {
                let response = completion.response?;
                let n = crate::v02::module_node(&self.description.root, &completion.id)
                    .ok_or("插件组件不存在")?;
                let r = n.registration.as_ref().ok_or("插件未注册")?.clone();
                if let Some(method) = completion.method {
                    let result = response.get("result").ok_or("插件方法未返回 result")?;
                    r.definition
                        .methods
                        .get(&method)
                        .ok_or("未知插件方法")?
                        .result
                        .validate(result)?;
                    if let Some(target) = completion.target {
                        self.set(&target, result.clone())?;
                    }
                }
                if let Some(image) = completion.image {
                    self.plugin_images.insert(completion.id.clone(), image);
                }
                if let Some(events) = response.get("events") {
                    let resolved = self.tree.resolve(&self.state)?;
                    let index = self
                        .tree
                        .nodes
                        .iter()
                        .position(|n| n.spec.id == completion.id)
                        .ok_or("插件组件不存在")?;
                    if !resolved[index].visible || !resolved[index].enabled {
                        return Ok(());
                    }
                    for event in events.as_array().ok_or("插件 events 必须为数组")? {
                        let name = event
                            .get("name")
                            .and_then(Value::as_str)
                            .ok_or("缺少事件 name")?;
                        let payload = event.get("payload").ok_or("缺少事件 payload")?;
                        r.definition
                            .events
                            .get(name)
                            .ok_or("未声明插件事件")?
                            .validate(payload)?;
                        let n = crate::v02::module_node(&self.description.root, &completion.id)
                            .unwrap();
                        if let Some(action) = n.events.get(name) {
                            let id = action.resolve(&self.state)?;
                            let id = match action {
                                crate::v02::EventAction::Fixed(_) => id.to_owned(),
                                _ => n.action_names.get(id).ok_or("未定义事件动作")?.clone(),
                            };
                            self.invoke_event(&id, Some(payload.clone()), 0)?;
                        }
                    }
                }
                Ok::<_, String>(())
            })();
            if let Err(e) = result {
                self.error = Some(format!("插件 {}: {e}", completion.id));
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
