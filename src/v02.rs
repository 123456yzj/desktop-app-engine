//! Strict 0.2 contract, separate from the original 0.1 parser and executor.
use crate::{
    binding::{Binding, Format, State, ValueType},
    description::App,
};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Description {
    pub credential_store: Option<String>,
    #[serde(default)]
    pub data_sources: BTreeMap<String, crate::http::Source>,
    pub schema_version: String,
    pub app: App,
    pub state: State,
    pub root: Node,
    pub actions: BTreeMap<String, Action>,
    #[serde(default)]
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Plugin,
    Module,
    Row,
    Column,
    Grid,
    Panel,
    Card,
    Text,
    Input,
    Select,
    Button,
    Metric,
    Badge,
    Progress,
    Table,
    LineChart,
    BarChart,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Node {
    pub component: Option<String>,
    #[serde(skip)]
    pub registration: Option<crate::plugin::Registration>,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub id: String,
    #[serde(default)]
    pub layout: Layout,
    #[serde(default)]
    pub style: Style,
    pub visible: Option<Binding>,
    pub enabled: Option<Binding>,
    #[serde(default)]
    pub events: BTreeMap<String, EventAction>,
    #[serde(default)]
    pub children: Vec<Component>,
    #[serde(default)]
    pub state: State,
    #[serde(default)]
    pub actions: BTreeMap<String, Action>,
    #[serde(skip)]
    pub action_names: BTreeMap<String, String>,
    #[serde(default)]
    pub props: BTreeMap<String, Binding>,
    #[serde(default)]
    pub emits: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub on: BTreeMap<String, EventAction>,
    #[serde(skip)]
    pub listener_action_names: BTreeMap<String, String>,
    #[serde(skip)]
    pub state_names: BTreeMap<String, String>,
    pub title: Option<Binding>,
    pub text: Option<Binding>,
    pub label: Option<Binding>,
    pub value: Option<Binding>,
    pub placeholder: Option<Binding>,
    pub options: Option<Binding>,
    #[serde(default)]
    pub options_format: SelectOptionsFormat,
    pub format: Option<Format>,
    pub tone: Option<Tone>,
    pub rows: Option<Binding>,
    pub row_key: Option<String>,
    pub row_height: Option<f32>,
    #[serde(default)]
    pub columns: Vec<ColumnSpec>,
    pub series: Option<Vec<Series>>,
    pub x_axis: Option<Axis>,
    pub y_axis: Option<Axis>,
    pub legend: Option<bool>,
    pub zoom: Option<bool>,
    pub pan: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectOptionsFormat {
    #[default]
    Array,
    Lines,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
}

/// Normalize bound options without changing the selected value during refresh.
pub fn select_options(n: &Node, state: &State) -> Result<Vec<SelectOption>, String> {
    let value = required(&n.options, "options")?.resolve(state)?;
    let options: Vec<SelectOption> = match n.options_format {
        SelectOptionsFormat::Lines => value
            .as_str()
            .ok_or("select.optionsFormat=lines 要求 options 为字符串")?
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| SelectOption {
                label: s.into(),
                value: s.into(),
                disabled: false,
            })
            .collect(),
        SelectOptionsFormat::Array => value
            .as_array()
            .ok_or("select.options 必须为数组")?
            .iter()
            .map(|v| match v.as_str() {
                Some(s) => Ok(SelectOption {
                    label: s.into(),
                    value: s.into(),
                    disabled: false,
                }),
                None => {
                    serde_json::from_value(v.clone()).map_err(|e| format!("select.options: {e}"))
                }
            })
            .collect::<Result<_, _>>()?,
    };
    if options.len() > 10000 {
        return Err("select.options 最多 10000 项".into());
    }
    let mut values = std::collections::BTreeSet::new();
    for option in &options {
        if !values.insert(&option.value) {
            return Err(format!("select.options value 重复: {:?}", option.value));
        }
    }
    Ok(options)
}

/// Existing fixed IDs and state/literal bindings share the same event dispatch path.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EventAction {
    Fixed(String),
    Binding(Binding),
}
impl EventAction {
    pub fn resolve<'a>(&'a self, state: &'a State) -> Result<&'a str, String> {
        let id = match self {
            Self::Fixed(id) => id.as_str(),
            Self::Binding(binding) => binding
                .resolve(state)?
                .as_str()
                .ok_or("事件动作绑定必须为字符串")?,
        };
        if id.trim().is_empty() {
            return Err("事件动作 ID 不得为空".into());
        }
        Ok(id)
    }
}

/// Checks fields for each component before deserializing the shared storage model.
#[derive(Debug, Clone)]
pub struct Component(pub Node);
impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        parse_node(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
fn parse_node(value: serde_json::Value) -> Result<Node, String> {
    let kind: Kind = serde_json::from_value(value.get("type").cloned().ok_or("缺少 type")?)
        .map_err(|e| e.to_string())?;
    let specific: &[&str] = match kind {
        Kind::Plugin => &["component", "props"],
        Kind::Module => &["children", "state", "actions", "props", "emits", "on"],
        Kind::Row | Kind::Column => &["children", "state", "actions"],
        Kind::Grid => &["children", "columns", "state", "actions"],
        Kind::Panel | Kind::Card => &["children", "title", "state", "actions"],
        Kind::Text => &["text"],
        Kind::Badge => &["text", "tone"],
        Kind::Input => &["value", "placeholder"],
        Kind::Select => &["value", "placeholder", "options", "optionsFormat"],
        Kind::Button => &["text"],
        Kind::Metric => &["label", "value", "format"],
        Kind::Progress => &["value", "text"],
        Kind::Table => &["rows", "rowKey", "rowHeight", "columns"],
        Kind::LineChart | Kind::BarChart => {
            &["series", "xAxis", "yAxis", "legend", "title", "zoom", "pan"]
        }
    };
    for key in value.as_object().ok_or("组件必须是对象")?.keys() {
        if ![
            "type", "id", "layout", "style", "visible", "enabled", "events",
        ]
        .contains(&key.as_str())
            && !specific.contains(&key.as_str())
        {
            return Err(format!("组件 {:?}: unknown field {key:?}", value.get("id")));
        }
    }
    if let Some(layout) = value.get("layout").and_then(serde_json::Value::as_object) {
        let container = matches!(
            kind,
            Kind::Module | Kind::Row | Kind::Column | Kind::Grid | Kind::Panel | Kind::Card
        );
        for key in layout.keys() {
            if !container && ["gap", "align", "justify", "wrap", "overflow"].contains(&key.as_str())
            {
                return Err(format!(
                    "组件 {:?}: layout.{key} 仅用于容器",
                    value.get("id")
                ));
            }
            if kind == Kind::Grid && key == "justify" {
                return Err("grid 不接受 layout.justify（轨道定义列宽）".into());
            }
        }
    }
    serde_json::from_value(value).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ColumnSpec {
    Track(Track),
    Data(Column),
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Column {
    pub id: String,
    pub field: String,
    pub title: String,
    pub width: Track,
    #[serde(default)]
    pub format: Format,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub fixed: Option<f32>,
    pub fr: Option<f32>,
    pub auto: Option<bool>,
}
impl Track {
    pub fn validate(&self) -> Result<(), String> {
        if usize::from(self.fixed.is_some())
            + usize::from(self.fr.is_some())
            + usize::from(self.auto.is_some())
            != 1
        {
            return Err("轨道必须恰好指定 fixed/fr/auto 之一".into());
        }
        if self
            .fixed
            .is_some_and(|n| !n.is_finite() || !(0.0..=1_000_000.0).contains(&n))
            || self
                .fr
                .is_some_and(|n| !n.is_finite() || n <= 0.0 || n > 1_000_000.0)
            || self.auto == Some(false)
        {
            return Err("轨道尺寸/权重非法".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Layout {
    pub width: Option<Track>,
    pub height: Option<Track>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    #[serde(default)]
    pub padding: f32,
    #[serde(default)]
    pub gap: f32,
    pub shrink: Option<f32>,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub align: Align,
    #[serde(default)]
    pub justify: Justify,
    #[serde(default)]
    pub overflow: Overflow,
}
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Overflow {
    #[default]
    Clip,
    Scroll,
}
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    #[default]
    Accent,
    Success,
    Warning,
    Danger,
    Muted,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Style {
    pub background: Option<Token>,
    pub color: Option<Token>,
    pub font_size: Option<f32>,
    #[serde(default)]
    pub border_width: f32,
    #[serde(default)]
    pub radius: f32,
    pub border_color: Option<Token>,
    pub font_family: Option<FontFamily>,
    pub font_weight: Option<FontWeight>,
    pub text_wrap: Option<bool>,
    pub text_overflow_tooltip: Option<bool>,
    pub text_align: Option<TextAlign>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
    pub hover: Option<ControlStyle>,
    pub pressed: Option<ControlStyle>,
    pub disabled: Option<ControlStyle>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControlStyle {
    pub background: Option<Token>,
    pub color: Option<Token>,
    pub border_color: Option<Token>,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontFamily {
    Proportional,
    Monospace,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontWeight {
    Normal,
    Bold,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    Start,
    Center,
    End,
}
#[derive(Debug, Clone, Copy)]
pub enum Token {
    Surface,
    SurfaceRaised,
    TextPrimary,
    TextMuted,
    Border,
    Accent,
    Success,
    Warning,
    Danger,
    Focus,
    Primary,
    OnPrimary,
    Hex([u8; 4]),
}
impl<'de> Deserialize<'de> for Token {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "surface" => Self::Surface,
            "surfaceRaised" => Self::SurfaceRaised,
            "textPrimary" => Self::TextPrimary,
            "textMuted" => Self::TextMuted,
            "border" => Self::Border,
            "accent" => Self::Accent,
            "success" => Self::Success,
            "warning" => Self::Warning,
            "danger" => Self::Danger,
            "focus" => Self::Focus,
            "primary" => Self::Primary,
            "onPrimary" => Self::OnPrimary,
            _ => {
                let hex = value
                    .strip_prefix('#')
                    .filter(|s| {
                        matches!(s.len(), 6 | 8) && s.bytes().all(|b| b.is_ascii_hexdigit())
                    })
                    .ok_or_else(|| {
                        serde::de::Error::custom("颜色必须为主题 Token 或 #RRGGBB / #RRGGBBAA")
                    })?;
                let mut rgba = [255; 4];
                for (i, channel) in rgba.iter_mut().enumerate().take(hex.len() / 2) {
                    *channel = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                        .map_err(serde::de::Error::custom)?;
                }
                Self::Hex(rgba)
            }
        })
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Series {
    pub id: String,
    pub name: String,
    pub data: Binding,
    pub x_field: String,
    pub y_field: String,
    #[serde(default)]
    pub monotonic_x: bool,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Axis {
    pub scale: Option<Scale>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scale {
    Linear,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub steps: Vec<Step>,
    #[serde(rename = "eventPayload", default)]
    pub event_payload: Option<serde_json::Value>,
    #[serde(skip)]
    pub event_samples: Vec<serde_json::Value>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Step {
    OpenLogs,
    Request {
        source: String,
    },
    PluginCall {
        target: String,
        method: String,
        args: Binding,
        #[serde(rename = "resultTarget")]
        result_target: Option<String>,
    },
    Emit {
        event: String,
        payload: Binding,
        #[serde(skip)]
        module: Option<String>,
    },
    Set {
        target: String,
        value: Binding,
    },
    ParseJson {
        target: String,
        value: Binding,
    },
    Concat {
        target: String,
        parts: Vec<Binding>,
    },
    Command {
        program: String,
        #[serde(default)]
        args: Vec<crate::command::Argument>,
        #[serde(default)]
        cwd: Option<String>,
        target: String,
        #[serde(rename = "timeoutMs", default = "crate::command::default_timeout")]
        timeout_ms: u64,
        #[serde(rename = "onComplete")]
        on_complete: Option<String>,
    },
}

pub fn parse(source: &str) -> Result<Description, String> {
    parse_at(source, &std::env::current_dir().map_err(|e| e.to_string())?)
}
pub fn parse_at(source: &str, base: &std::path::Path) -> Result<Description, String> {
    // The root uses the same strict component field check as its descendants.
    let mut raw: serde_json::Value =
        serde_json::from_str(source).map_err(|e| format!("JSON: {e}"))?;
    fn structural(v: &serde_json::Value, path: &str) -> Result<(), String> {
        match v {
            serde_json::Value::Object(o) => {
                for (k, v) in o {
                    if k == "state"
                        || k == "responseSample"
                        || k == "emits"
                        || k == "eventPayload" && !v.is_null()
                        || k == "value"
                            && o.get("kind").and_then(serde_json::Value::as_str) == Some("literal")
                    {
                        continue;
                    }
                    let p = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    if v.is_null() {
                        return Err(format!("{p}: 结构字段不得为 null，请省略可选字段"));
                    }
                    structural(v, &p)?;
                }
            }
            serde_json::Value::Array(a) => {
                for (i, v) in a.iter().enumerate() {
                    structural(v, &format!("{path}[{i}]"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    structural(&raw, "")?;
    let root = parse_node(raw.get("root").cloned().ok_or("root: 缺少字段")?)
        .map_err(|e| format!("root: {e}"))?;
    raw["root"] = serde_json::json!({"type":"column","id":"__parse_root"});
    let mut d: Description = serde_path_to_error::deserialize(raw)
        .map_err(|e| format!("描述字段 {}: {}", e.path(), e.inner()))?;
    d.root = root;
    let registry = crate::plugin::discover(base, &d.plugins)?;
    fn register(
        n: &mut Node,
        registry: &BTreeMap<String, crate::plugin::Registration>,
    ) -> Result<(), String> {
        if n.kind == Kind::Plugin {
            let name = required(&n.component, "component")?;
            n.registration = Some(
                registry
                    .get(name)
                    .ok_or_else(|| format!("未注册插件组件 {name:?}"))?
                    .clone(),
            );
        }
        for c in &mut n.children {
            register(&mut c.0, registry)?;
        }
        Ok(())
    }
    register(&mut d.root, &registry)?;
    validate(&d)?;
    Ok(d)
}

pub fn validate(d: &Description) -> Result<(), String> {
    validate_flat(&crate::scope::compile(d.clone())?)
}

pub(crate) fn action_uses_event(action: &Action) -> bool {
    let is_event = |b: &Binding| matches!(b, Binding::Event { .. });
    action.steps.iter().any(|step| match step {
        Step::Request { .. } | Step::OpenLogs => false,
        Step::PluginCall { args, .. } => is_event(args),
        Step::Set { value, .. } | Step::ParseJson { value, .. } => is_event(value),
        Step::Concat { parts, .. } => parts.iter().any(is_event),
        Step::Emit { payload, .. } => is_event(payload),
        Step::Command { args, .. } => args
            .iter()
            .any(|a| matches!(a,crate::command::Argument::Binding(b) if is_event(b))),
    })
}
pub(crate) fn module_node<'a>(n: &'a Node, id: &str) -> Option<&'a Node> {
    if n.id == id {
        Some(n)
    } else {
        n.children.iter().find_map(|c| module_node(&c.0, id))
    }
}
pub(crate) fn listener(n: &Node, event: &str, state: &State) -> Result<Option<String>, String> {
    n.on.get(event)
        .map(|a| {
            let id = a.resolve(state)?;
            match a {
                EventAction::Fixed(_) => Ok(id.to_owned()),
                EventAction::Binding(_) => n
                    .listener_action_names
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("on.{event}: 父作用域未定义动作 {id:?}")),
            }
        })
        .transpose()
}

pub(crate) fn validate_flat(d: &Description) -> Result<(), String> {
    if d.schema_version != "0.2" {
        return Err("schemaVersion: 仅支持 0.2".into());
    }
    if d.app.name.trim().is_empty() {
        return Err("app.name: 不能为空".into());
    }
    for (field, n) in [
        ("width", d.app.window.width),
        ("height", d.app.window.height),
    ] {
        if !n.is_finite() || !(160.0..=8192.0).contains(&n) {
            return Err(format!("app.window.{field}: 必须在 160..8192"));
        }
    }
    for (key, v) in &d.state {
        if key.trim().is_empty() {
            return Err("state 键不能为空".into());
        }
        ValueType::of(v).map_err(|e| format!("state.{key}: {e}"))?;
    }
    validate_node(&d.root, d, &mut HashSet::new(), 0)?;
    if d.data_sources.len() > 64 {
        return Err("dataSources 最多 64 个".into());
    }
    let mut targets = HashSet::new();
    if let Some(name) = &d.credential_store {
        crate::credentials::valid_name(name)?;
    }
    for (name, source) in &d.data_sources {
        if source.requires_credentials() && d.credential_store.is_none() {
            return Err("凭据请求头必须配置顶层 credentialStore".into());
        }
        if name.trim().is_empty() {
            return Err("dataSource 名称不得为空".into());
        }
        source
            .validate(&d.state)
            .map_err(|e| format!("dataSources.{name}: {e}"))?;
        for target in
            std::iter::once(&source.status_target).chain(source.mappings.iter().map(|m| &m.target))
        {
            if !targets.insert(target) {
                return Err(format!("dataSources 不能共享写入目标 {target:?}"));
            }
        }
    }
    let mut contexts: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    fn collect(
        n: &Node,
        d: &Description,
        contexts: &mut BTreeMap<String, Vec<serde_json::Value>>,
    ) -> Result<(), String> {
        for (event, sample) in &n.emits {
            if let Some(id) = listener(n, event, &d.state)? {
                if !d.actions.contains_key(&id) {
                    return Err(format!("on.{event}: 未定义动作 {id:?}"));
                }
                contexts.entry(id).or_default().push(sample.clone());
            }
        }
        for c in &n.children {
            collect(&c.0, d, contexts)?;
        }
        Ok(())
    }
    collect(&d.root, d, &mut contexts)?;
    for (id, a) in &d.actions {
        if id.trim().is_empty() || a.steps.is_empty() {
            return Err(format!("actions.{id}: ID 和 steps 不得为空"));
        }
        let mut known_samples = a.event_samples.clone();
        known_samples.extend(contexts.get(id).into_iter().flatten().cloned());
        let samples = if known_samples.is_empty() {
            None
        } else {
            Some(&known_samples)
        };
        if let Some(declared) = &a.event_payload {
            let contract = ValueType::of(declared)?;
            for sample in samples.into_iter().flatten() {
                if !contract.accepts(sample) {
                    return Err(format!("actions.{id}.eventPayload: 事件载荷类型不匹配"));
                }
            }
        }
        if action_uses_event(a) && samples.is_none() && a.event_payload.is_none() {
            return Err(format!(
                "actions.{id}: event 绑定需要模块 on 事件上下文或 eventPayload 类型声明"
            ));
        }
        let payloads: Vec<_> = if action_uses_event(a) {
            let mut values: Vec<_> = samples.into_iter().flatten().map(Some).collect();
            if let Some(sample) = &a.event_payload {
                values.push(Some(sample));
            }
            values
        } else {
            vec![None]
        };
        for payload in payloads {
            for (i, step) in a.steps.iter().enumerate() {
                let result = (|| {
                    let (target, value) = match step {
                        Step::OpenLogs => return Ok(()),
                        Step::Request { source } => {
                            if !d.data_sources.contains_key(source) {
                                return Err(format!("未定义 dataSource {source:?}"));
                            }
                            return Ok(());
                        }
                        Step::PluginCall {
                            target,
                            method,
                            args,
                            result_target,
                        } => {
                            let n = module_node(&d.root, target)
                                .ok_or("pluginCall target 组件不存在")?;
                            let r = n
                                .registration
                                .as_ref()
                                .ok_or("pluginCall target 必须为插件组件")?;
                            let m = r.definition.methods.get(method).ok_or("插件未声明该方法")?;
                            m.args.validate(args.resolve_event(&d.state, payload)?)?;
                            if let Some(key) = result_target {
                                m.result
                                    .validate(d.state.get(key).ok_or("resultTarget 状态不存在")?)?;
                            }
                            return Ok(());
                        }
                        Step::Emit {
                            event,
                            payload: value,
                            module,
                        } => {
                            let module = module.as_ref().ok_or("emit 缺少模块作用域")?;
                            let n = module_node(&d.root, module).ok_or("emit 模块不存在")?;
                            let sample = n
                                .emits
                                .get(event)
                                .ok_or_else(|| format!("未声明 emits 事件 {event:?}"))?;
                            if !ValueType::of(sample)?
                                .accepts(value.resolve_event(&d.state, payload)?)
                            {
                                return Err(format!("emit {event:?}: 载荷类型不匹配"));
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
                            if let Some(id) = on_complete {
                                let callback = d.actions.get(id).ok_or("onComplete 动作不存在")?;
                                if action_uses_event(callback) || callback.event_payload.is_some() {
                                    return Err("onComplete 动作不能要求 event 载荷".into());
                                }
                            }
                            return crate::command::validate_event(
                                program,
                                args,
                                cwd,
                                target,
                                *timeout_ms,
                                &d.state,
                                payload,
                            );
                        }
                        Step::Set { target, value } => {
                            (target, value.resolve_event(&d.state, payload)?.clone())
                        }
                        Step::ParseJson { target, value } => {
                            if !d.state.contains_key(target) {
                                return Err("parseJson target 状态不存在".into());
                            }
                            if !value.resolve_event(&d.state, payload)?.is_string() {
                                return Err("parseJson value 必须为字符串".into());
                            }
                            return Ok(());
                        }
                        Step::Concat { target, parts } => {
                            let mut s = String::new();
                            for p in parts {
                                s.push_str(
                                    p.resolve_event(&d.state, payload)?
                                        .as_str()
                                        .ok_or("concat 仅接受字符串")?,
                                );
                            }
                            (target, serde_json::Value::String(s))
                        }
                    };
                    let initial = d
                        .state
                        .get(target)
                        .ok_or_else(|| format!("未定义状态 {target:?}"))?;
                    if !ValueType::of(initial)?.accepts(&value) {
                        return Err("赋值类型不匹配".into());
                    }
                    Ok::<_, String>(())
                })();
                result.map_err(|e| format!("actions.{id}.steps[{i}]: {e}"))?;
            }
        }
    }
    Ok(())
}

fn validate_node(
    n: &Node,
    d: &Description,
    ids: &mut HashSet<String>,
    depth: usize,
) -> Result<(), String> {
    let result = (|| {
        if depth > 64 || n.id.trim().is_empty() || !ids.insert(n.id.clone()) {
            return Err("ID 为空、重复组件 ID 或深度超过 64".into());
        }
        for b in [&n.visible, &n.enabled].into_iter().flatten() {
            if !b.resolve(&d.state)?.is_boolean() {
                return Err("visible/enabled 必须为布尔值".into());
            }
        }
        for prop in n.props.values() {
            let value = prop.resolve(&d.state)?;
            if n.kind != Kind::Plugin {
                ValueType::of(value)?;
            }
        }
        let l = &n.layout;
        for t in [&l.width, &l.height].into_iter().flatten() {
            t.validate()?;
        }
        for v in [
            Some(l.padding),
            Some(l.gap),
            l.min_width,
            l.max_width,
            l.min_height,
            l.max_height,
            l.shrink,
            Some(n.style.border_width),
            Some(n.style.radius),
            n.style.font_size,
            n.style.padding_x,
            n.style.padding_y,
        ]
        .into_iter()
        .flatten()
        {
            if !v.is_finite() || !(0.0..=1_000_000.0).contains(&v) {
                return Err("layout/style: 尺寸必须有限且在 0..1000000".into());
            }
        }
        if l.min_width.unwrap_or(0.0) > l.max_width.unwrap_or(f32::MAX)
            || l.min_height.unwrap_or(0.0) > l.max_height.unwrap_or(f32::MAX)
        {
            return Err("layout: min > max".into());
        }
        if l.wrap && n.kind != Kind::Row {
            return Err("layout.wrap 仅用于 row".into());
        }
        for (event, action) in &n.events {
            let allowed: &[&str] = match n.kind {
                Kind::Plugin => &[],
                Kind::Button => &["click"],
                Kind::Input => &["change", "submit"],
                Kind::Select => &["change", "open"],
                Kind::Table => &["rowSelect", "sortChange"],
                _ => &[],
            };
            if !allowed.contains(&event.as_str())
                && !(n.kind == Kind::Plugin
                    && n.registration
                        .as_ref()
                        .is_some_and(|r| r.definition.events.contains_key(event)))
            {
                return Err(format!("不支持事件 {event:?}"));
            }
            let resolved = action
                .resolve(&d.state)
                .map_err(|e| format!("events.{event}: {e}"))?;
            let action = match action {
                EventAction::Fixed(_) => resolved,
                EventAction::Binding(_) => n
                    .action_names
                    .get(resolved)
                    .map(String::as_str)
                    .ok_or_else(|| format!("当前作用域未定义动作 {resolved:?}"))?,
            };
            if !d.actions.contains_key(action) {
                return Err(format!("未定义动作 {action:?}"));
            }
            if n.kind == Kind::Plugin {
                let contract = &n
                    .registration
                    .as_ref()
                    .ok_or("缺少插件注册")?
                    .definition
                    .events[event];
                contract.validate(
                    d.actions[action]
                        .event_payload
                        .as_ref()
                        .ok_or("插件事件动作必须声明 eventPayload")?,
                )?;
            } else if action_uses_event(&d.actions[action])
                || d.actions[action].event_payload.is_some()
            {
                return Err("event 处理动作不可直接绑定控件事件，请通过 emit 调用".into());
            }
        }
        for b in [&n.text, &n.label, &n.placeholder, &n.title]
            .into_iter()
            .flatten()
        {
            if !b.resolve(&d.state)?.is_string() {
                return Err("文本属性必须为字符串".into());
            }
        }
        match n.kind {
            Kind::Plugin => {
                let r = n.registration.as_ref().ok_or("缺少插件注册")?;
                r.definition.props.validate(&plugin_props(n, &d.state)?)?;
            }
            Kind::Text | Kind::Button | Kind::Badge => {
                required(&n.text, "text")?;
            }
            Kind::Input | Kind::Select => {
                let b = required(&n.value, "value")?;
                if !b.resolve(&d.state)?.is_string() {
                    return Err("input.value 必须为字符串".into());
                }
                if matches!(b,Binding::State {path,..} if !path.is_empty()) {
                    return Err("input.value 暂仅支持顶层状态（path 为空）".into());
                }
                if n.kind == Kind::Select {
                    select_options(n, &d.state)?;
                }
            }
            Kind::Metric | Kind::Progress => {
                let v = required(&n.value, "value")?
                    .resolve(&d.state)?
                    .as_f64()
                    .ok_or("value 必须为数字")?;
                if n.kind == Kind::Progress && !(0.0..=1.0).contains(&v) {
                    return Err("progress.value 必须在 [0,1]".into());
                }
                if n.kind == Kind::Metric {
                    required(&n.label, "label")?;
                    n.format
                        .clone()
                        .unwrap_or_default()
                        .apply(&serde_json::json!(v))?;
                }
            }
            Kind::Grid => {
                if n.columns.is_empty() || n.columns.len() > 64 {
                    return Err("grid.columns 数量必须在 1..64".into());
                }
                for c in &n.columns {
                    match c {
                        ColumnSpec::Track(t) if t.auto.is_none() => t.validate()?,
                        _ => return Err("grid.columns 仅接受 fixed/fr".into()),
                    }
                }
            }
            Kind::Table => {
                crate::table::validate(n, &d.state)?;
            }
            Kind::LineChart | Kind::BarChart => {
                crate::chart::validate(n, &d.state)?;
            }
            _ => {}
        }
        for child in &n.children {
            validate_node(&child.0, d, ids, depth + 1)?;
        }
        Ok::<_, String>(())
    })();
    result.map_err(|e| format!("组件 {:?}: {e}", n.id))
}
pub fn required<'a, T>(v: &'a Option<T>, field: &str) -> Result<&'a T, String> {
    v.as_ref().ok_or_else(|| format!("缺少字段 {field}"))
}
pub fn plugin_props(n: &Node, state: &State) -> Result<serde_json::Value, String> {
    let props: serde_json::Map<String, serde_json::Value> = n
        .props
        .iter()
        .map(|(k, b)| Ok((k.clone(), b.resolve(state)?.clone())))
        .collect::<Result<_, String>>()?;
    Ok(serde_json::Value::Object(props))
}
