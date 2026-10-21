//! Commit ordered input against the current document generation and lexical scope.
use super::{Binding, Runtime, UiEvent};

impl Runtime {
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
                .metadata
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
            let action_names = node.metadata.action_names.clone();
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
}
