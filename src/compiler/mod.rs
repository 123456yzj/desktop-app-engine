//! Resolve authored names and resources before constructing a runtime.
mod metadata;
mod scope;
mod validation;
pub use metadata::CompiledDescription;
pub(crate) use metadata::{Metadata, NodeMetadata, action_uses_event, listener, module_node};
pub(crate) use validation::validate_flat;

use crate::{
    plugin,
    protocol::v02::{Description, Kind, Node, required},
};
use std::path::Path;

pub fn compile(description: Description) -> Result<CompiledDescription, String> {
    compile_at(
        description,
        &std::env::current_dir().map_err(|e| e.to_string())?,
    )
}

/// Resource paths are resolved once, relative to the input document directory.
pub fn compile_at(description: Description, base: &Path) -> Result<CompiledDescription, String> {
    let registry = plugin::discover(base, &description.plugins)?;
    let mut metadata = Metadata::default();
    fn register(
        n: &Node,
        registry: &std::collections::BTreeMap<String, plugin::Registration>,
        metadata: &mut Metadata,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 64 || metadata.nodes.contains_key(&n.id) {
            return Err("重复组件 ID 或深度超过 64".into());
        }
        let registration = if n.kind == Kind::Plugin {
            let name = required(&n.component, "component")?;
            Some(
                registry
                    .get(name)
                    .ok_or_else(|| format!("未注册插件组件 {name:?}"))?
                    .clone(),
            )
        } else {
            None
        };
        metadata.nodes.insert(
            n.id.clone(),
            NodeMetadata {
                registration,
                ..Default::default()
            },
        );
        for child in &n.children {
            register(&child.0, registry, metadata, depth + 1)?;
        }
        Ok(())
    }
    register(&description.root, &registry, &mut metadata, 0)?;
    let compiled = scope::compile(description, metadata)?;
    validate_flat(&compiled.description, &compiled.metadata)?;
    Ok(compiled)
}
