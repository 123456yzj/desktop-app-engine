use crate::{
    binding::State,
    plugin::Registration,
    protocol::{Action, Description, EventAction, Node},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub(crate) struct NodeMetadata {
    pub registration: Option<Registration>,
    pub action_names: BTreeMap<String, String>,
    pub listener_action_names: BTreeMap<String, String>,
    pub state_names: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ActionMetadata {
    pub module: Option<String>,
    pub event_samples: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Metadata {
    pub nodes: BTreeMap<String, NodeMetadata>,
    pub actions: BTreeMap<String, ActionMetadata>,
}

/// Validated, resolved description. Only the compiler can construct this type.
#[derive(Debug, Clone)]
pub struct CompiledDescription {
    pub(crate) description: Description,
    pub(crate) metadata: Metadata,
}

impl CompiledDescription {
    pub fn app(&self) -> &crate::protocol::App {
        &self.description.app
    }
    pub fn credential_namespace(&self) -> Option<&str> {
        self.description.credential_store.as_deref()
    }
}

pub(crate) fn action_uses_event(action: &Action) -> bool {
    use crate::{binding::Binding, protocol::Step};
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

pub(crate) fn listener(
    n: &Node,
    metadata: &Metadata,
    event: &str,
    state: &State,
) -> Result<Option<String>, String> {
    n.on.get(event)
        .map(|a| {
            let id = a.resolve(state)?;
            match a {
                EventAction::Fixed(_) => Ok(id.to_owned()),
                EventAction::Binding(_) => metadata.nodes[&n.id]
                    .listener_action_names
                    .get(id)
                    .cloned()
                    .ok_or_else(|| format!("on.{event}: 父作用域未定义动作 {id:?}")),
            }
        })
        .transpose()
}
