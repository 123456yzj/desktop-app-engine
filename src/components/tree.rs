//! Retained arena: external IDs are strings, internal IDs are indices, never hashes.
use crate::{
    binding::State,
    v02::{Kind, Node},
};
pub type NodeId = usize;
pub struct ComponentNode {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub spec: Node,
    pub(crate) metadata: crate::compiler::NodeMetadata,
}
pub struct Tree {
    pub nodes: Vec<ComponentNode>,
}
#[derive(Clone, Debug)]
pub struct Resolved {
    pub visible: bool,
    pub enabled: bool,
    pub text: String,
    pub label: String,
    pub title: String,
    pub placeholder: String,
    pub value: Option<serde_json::Value>,
    pub options: Vec<crate::v02::SelectOption>,
    pub api: std::collections::BTreeMap<String, serde_json::Value>,
}
impl Tree {
    pub fn ancestor(&self, id: NodeId, kind: Kind) -> Option<NodeId> {
        let mut parent = self.nodes[id].parent;
        while let Some(id) = parent {
            if self.nodes[id].spec.kind == kind {
                return Some(id);
            }
            parent = self.nodes[id].parent;
        }
        None
    }
    pub fn horizontal(&self, id: NodeId) -> bool {
        let node = &self.nodes[id];
        use super::catalogue::LayoutKind;
        match super::catalogue::definition(node.spec.kind).layout {
            LayoutKind::Row => true,
            LayoutKind::Directional | LayoutKind::Splitter => !node.spec.vertical,
            LayoutKind::Page => node
                .children
                .iter()
                .any(|&id| self.nodes[id].spec.kind == Kind::Sider),
            _ => false,
        }
    }
    pub(crate) fn new(root: &Node, metadata: &crate::compiler::Metadata) -> Self {
        fn add(
            tree: &mut Tree,
            spec: &Node,
            parent: Option<NodeId>,
            metadata: &crate::compiler::Metadata,
        ) -> NodeId {
            let id = tree.nodes.len();
            let mut shallow = spec.clone();
            shallow.children.clear();
            tree.nodes.push(ComponentNode {
                parent,
                children: vec![],
                spec: shallow,
                metadata: metadata.nodes[&spec.id].clone(),
            });
            for child in &spec.children {
                let c = add(tree, &child.0, Some(id), metadata);
                tree.nodes[id].children.push(c);
            }
            id
        }
        let mut tree = Self { nodes: vec![] };
        add(&mut tree, root, None, metadata);
        tree
    }
    pub fn resolve(&self, state: &State) -> Result<Vec<Resolved>, String> {
        let mut result = Vec::with_capacity(self.nodes.len());
        for n in &self.nodes {
            let mut r = resolve_node(n, state)?;
            if let Some(p) = n.parent {
                let parent: &Resolved = &result[p];
                r.visible &= parent.visible;
                r.enabled &= parent.enabled;
                if matches!(self.nodes[p].spec.kind, Kind::App | Kind::ConfigProvider) {
                    r.enabled &= !crate::v02::widgets::flag(
                        &self.nodes[p].spec,
                        state,
                        "componentDisabled",
                        false,
                    );
                }
            }
            result.push(r);
        }
        Ok(result)
    }
    pub fn binding_version(
        &self,
        id: NodeId,
        versions: &std::collections::BTreeMap<String, u64>,
    ) -> u64 {
        let n = &self.nodes[id].spec;
        [
            &n.visible,
            &n.enabled,
            &n.text,
            &n.label,
            &n.value,
            &n.title,
            &n.placeholder,
            &n.rows,
            &n.options,
        ]
        .into_iter()
        .flatten()
        .chain(n.series.iter().flatten().map(|s| &s.data))
        .chain(n.props.values())
        .chain(n.api.values())
        .filter_map(|b| match b {
            crate::binding::Binding::State { key, .. } => versions.get(key).copied(),
            _ => None,
        })
        .max()
        .unwrap_or(1)
    }
    pub fn resolve_changed(
        &self,
        state: &State,
        versions: &std::collections::BTreeMap<String, u64>,
        since: u64,
        resolved: &mut [Resolved],
    ) -> Result<usize, String> {
        let mut changed = vec![false; self.nodes.len()];
        let mut count = 0;
        for (id, n) in self.nodes.iter().enumerate() {
            if self.binding_version(id, versions) > since || n.parent.is_some_and(|p| changed[p]) {
                let mut r = resolve_node(n, state)?;
                if let Some(p) = n.parent {
                    r.visible &= resolved[p].visible;
                    r.enabled &= resolved[p].enabled;
                    if matches!(self.nodes[p].spec.kind, Kind::App | Kind::ConfigProvider) {
                        r.enabled &= !crate::v02::widgets::flag(
                            &self.nodes[p].spec,
                            state,
                            "componentDisabled",
                            false,
                        );
                    }
                }
                resolved[id] = r;
                changed[id] = true;
                count += 1;
            }
        }
        Ok(count)
    }
}
fn resolve_node(n: &ComponentNode, state: &State) -> Result<Resolved, String> {
    let text = |b: &Option<crate::binding::Binding>| -> Result<String, String> {
        Ok(b.as_ref()
            .map(|b| b.resolve(state))
            .transpose()?
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned())
    };
    let mut value = match n.spec.kind {
        kind if kind.is_input()
            || crate::v02::widgets::is_widget(kind)
            || matches!(kind, Kind::Metric | Kind::Progress | Kind::Splitter) =>
        {
            n.spec
                .value
                .as_ref()
                .map(|b| b.resolve(state).cloned())
                .transpose()?
        }
        _ => None,
    };
    let mut rendered = text(&n.spec.text)?;
    if n.spec.kind == Kind::Metric {
        rendered = n
            .spec
            .format
            .clone()
            .unwrap_or_default()
            .apply(value.as_ref().unwrap())?;
        rendered = format!(
            "{}{}{}",
            crate::v02::widgets::string(&n.spec, state, "prefix", ""),
            rendered,
            crate::v02::widgets::string(&n.spec, state, "suffix", "")
        );
        value = None;
    }
    let inherited_disabled = crate::v02::widgets::flag(&n.spec, state, "disabled", false)
        || matches!(n.spec.kind, Kind::Button | Kind::Switch)
            && crate::v02::widgets::flag(&n.spec, state, "loading", false);
    Ok(Resolved {
        visible: n
            .spec
            .visible
            .as_ref()
            .map(|b| b.resolve(state))
            .transpose()?
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        enabled: n
            .spec
            .enabled
            .as_ref()
            .map(|b| b.resolve(state))
            .transpose()?
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
            && !inherited_disabled,
        text: rendered,
        label: text(&n.spec.label)?,
        title: text(&n.spec.title)?,
        placeholder: text(&n.spec.placeholder)?,
        value,
        options: if n.spec.kind == Kind::Radio {
            crate::v02::select_options(&n.spec, state)?
        } else {
            vec![]
        },
        api: n
            .spec
            .api
            .iter()
            .map(|(key, b)| Ok((key.clone(), b.resolve(state)?.clone())))
            .collect::<Result<_, String>>()?,
    })
}
