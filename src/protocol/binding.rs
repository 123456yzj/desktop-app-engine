//! Version 0.2 values, pointers and deliberately limited formatting. No GUI dependencies.
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub type State = BTreeMap<String, Value>;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Binding {
    Literal {
        value: Value,
    },
    State {
        key: String,
        #[serde(default)]
        path: String,
    },
    Prop {
        key: String,
        #[serde(default)]
        path: String,
    },
    Event {
        #[serde(default)]
        path: String,
    },
}

impl Binding {
    pub fn resolve<'a>(&'a self, state: &'a State) -> Result<&'a Value, String> {
        match self {
            Self::Literal { value } => Ok(value),
            Self::State { key, path } => {
                if !valid_pointer(path) {
                    return Err(format!(
                        "state {key:?} path {path:?}: 非法 RFC 6901 pointer"
                    ));
                }
                state
                    .get(key)
                    .ok_or_else(|| format!("未定义状态 {key:?}"))?
                    .pointer(path)
                    .ok_or_else(|| format!("state {key:?} path {path:?}: 缺少字段"))
            }
            Self::Prop { .. } => Err("prop 绑定必须先经过模块作用域解析".into()),
            Self::Event { .. } => Err("event 绑定仅可在模块事件处理动作中使用".into()),
        }
    }
    pub fn resolve_event<'a>(
        &'a self,
        state: &'a State,
        payload: Option<&'a Value>,
    ) -> Result<&'a Value, String> {
        if let Self::Event { path } = self {
            if !valid_pointer(path) {
                return Err("event.path: 非法 RFC 6901 pointer".into());
            }
            payload
                .ok_or("event 绑定缺少事件载荷")?
                .pointer(path)
                .ok_or_else(|| format!("event.path {path:?}: 缺少字段"))
        } else {
            self.resolve(state)
        }
    }
}

pub fn valid_pointer(path: &str) -> bool {
    (path.is_empty() || path.starts_with('/')) && {
        let mut chars = path.chars();
        let mut valid = true;
        while let Some(c) = chars.next() {
            if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
                valid = false;
                break;
            }
        }
        valid
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    Text,
    Nullable(Box<ValueType>),
    Array(Option<Box<ValueType>>),
    Object(BTreeMap<String, ValueType>),
}

impl ValueType {
    pub fn of(value: &Value) -> Result<Self, String> {
        Ok(match value {
            Value::Null => Self::Null,
            Value::Bool(_) => Self::Bool,
            Value::Number(n) if n.as_f64().is_some_and(f64::is_finite) => Self::Number,
            Value::Number(_) => return Err("数字必须有限".into()),
            Value::String(s) => {
                crate::validation::string_size(s, "value")?;
                Self::Text
            }
            Value::Array(a) => {
                let mut element: Option<Box<ValueType>> = None;
                for v in a.iter().filter(|v| !v.is_null()) {
                    let t = Self::of(v)?;
                    element = Some(Box::new(if let Some(prior) = element {
                        Self::merge(*prior, t)?
                    } else {
                        t
                    }));
                }
                Self::Array(element)
            }
            Value::Object(o) => Self::Object(
                o.iter()
                    .map(|(k, v)| Ok((k.clone(), Self::of(v)?)))
                    .collect::<Result<_, String>>()?,
            ),
        })
    }
    pub fn accepts(&self, value: &Value) -> bool {
        match (self, value) {
            (Self::Nullable(t), v) => v.is_null() || t.accepts(v),
            (Self::Array(t), Value::Array(a)) => a
                .iter()
                .all(|v| v.is_null() || t.as_ref().is_none_or(|t| t.accepts(v))),
            (Self::Object(t), Value::Object(o)) => {
                t.len() == o.len()
                    && t.iter()
                        .all(|(k, t)| o.get(k).is_some_and(|v| t.accepts(v)))
            }
            _ => Self::of(value).is_ok_and(|t| &t == self),
        }
    }
    fn merge(a: Self, b: Self) -> Result<Self, String> {
        if a == b {
            return Ok(a);
        }
        match (a, b) {
            (Self::Null, t) | (t, Self::Null) => Ok(if matches!(t, Self::Nullable(_)) {
                t
            } else {
                Self::Nullable(Box::new(t))
            }),
            (Self::Nullable(a), Self::Nullable(b)) => {
                Ok(Self::Nullable(Box::new(Self::merge(*a, *b)?)))
            }
            (Self::Nullable(a), b) | (b, Self::Nullable(a)) => {
                Ok(Self::Nullable(Box::new(Self::merge(*a, b)?)))
            }
            (Self::Object(a), Self::Object(mut b)) if a.keys().eq(b.keys()) => Ok(Self::Object(
                a.into_iter()
                    .map(|(k, v)| {
                        let t = Self::merge(v, b.remove(&k).unwrap())?;
                        Ok((k, t))
                    })
                    .collect::<Result<_, String>>()?,
            )),
            (Self::Array(a), Self::Array(b)) => Ok(Self::Array(match (a, b) {
                (Some(a), Some(b)) => Some(Box::new(Self::merge(*a, *b)?)),
                (a, b) => a.or(b),
            })),
            _ => Err("数组元素类型不一致".into()),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Format {
    #[serde(rename = "type", default)]
    pub kind: FormatKind,
    #[serde(default)]
    pub precision: usize,
    #[serde(default)]
    pub unit: String,
}
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FormatKind {
    #[default]
    Text,
    Number,
    Percent,
}

impl Format {
    pub fn apply(&self, value: &Value) -> Result<String, String> {
        if self.precision > 12 {
            return Err("format.precision 必须在 0..12".into());
        }
        if self.kind == FormatKind::Text && self.precision != 0 {
            return Err("text 格式 precision 必须为 0".into());
        }
        if value.is_null() {
            return Ok("—".into());
        }
        let text = match self.kind {
            FormatKind::Text => match value {
                Value::String(s) => s.clone(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                _ => return Err("text 格式仅接受字符串、数字、布尔或 null".into()),
            },
            FormatKind::Number | FormatKind::Percent => {
                let n = value
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or("数值格式必须绑定有限数字")?;
                let n = if self.kind == FormatKind::Percent {
                    n * 100.0
                } else {
                    n
                };
                if !n.is_finite() {
                    return Err("格式化数值溢出".into());
                }
                let n = if n.abs() < 0.5 * 10.0_f64.powi(-(self.precision as i32)) {
                    0.0
                } else {
                    n
                };
                format!(
                    "{:.*}{}",
                    self.precision,
                    n,
                    if self.kind == FormatKind::Percent {
                        "%"
                    } else {
                        ""
                    }
                )
            }
        };
        Ok(format!("{text}{}", self.unit))
    }
}
