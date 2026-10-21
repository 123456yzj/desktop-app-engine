//! Strict 0.2 application protocol.
use crate::{
    binding::{Binding, Format, State},
    protocol::App,
};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

mod parse;
pub use parse::parse;

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
    #[serde(default)]
    pub props: BTreeMap<String, Binding>,
    #[serde(default)]
    pub emits: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub on: BTreeMap<String, EventAction>,
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
        parse::parse_node(value)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
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
