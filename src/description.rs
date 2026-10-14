use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Description {
    pub schema_version: String,
    pub app: App,
    pub state: BTreeMap<String, String>,
    pub root: Component,
    pub actions: BTreeMap<String, Action>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    pub name: String,
    pub window: Window,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Component {
    Row {
        id: String,
        children: Vec<Component>,
    },
    Column {
        id: String,
        children: Vec<Component>,
    },
    Text {
        id: String,
        text: Property,
    },
    Input {
        id: String,
        value: Property,
        placeholder: Option<Property>,
    },
    Button {
        id: String,
        text: Property,
        #[serde(rename = "onClick")]
        on_click: String,
    },
}

impl Component {
    pub fn id(&self) -> &str {
        match self {
            Self::Row { id, .. }
            | Self::Column { id, .. }
            | Self::Text { id, .. }
            | Self::Input { id, .. }
            | Self::Button { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Property {
    Literal { value: String },
    State { key: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Step {
    Set {
        target: String,
        value: Property,
    },
    Concat {
        target: String,
        parts: Vec<Property>,
    },
}
