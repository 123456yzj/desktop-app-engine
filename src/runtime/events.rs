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
                ancestors_active &=
                    !crate::protocol::widgets::flag(&n.spec, &self.state, "disabled", false)
                        && (!matches!(
                            n.spec.kind,
                            crate::protocol::Kind::ConfigProvider | crate::protocol::Kind::App
                        ) || !crate::protocol::widgets::flag(
                            &n.spec,
                            &self.state,
                            "componentDisabled",
                            false,
                        ));
                parent = n.parent;
            }
            if !ancestors_active || !active(&node.spec.visible) || !active(&node.spec.enabled) {
                continue;
            }
            if crate::protocol::widgets::flag(&node.spec, &self.state, "disabled", false) {
                continue;
            }
            if matches!(
                node.spec.kind,
                crate::protocol::Kind::Button | crate::protocol::Kind::Switch
            ) && crate::protocol::widgets::flag(&node.spec, &self.state, "loading", false)
            {
                continue;
            }
            if e.kind == "change"
                && matches!(
                    node.spec.kind,
                    crate::protocol::Kind::Input | crate::protocol::Kind::InputNumber
                )
                && crate::protocol::widgets::flag(&node.spec, &self.state, "readOnly", false)
            {
                continue;
            }
            if let Err(error) = crate::protocol::widgets::validate_secondary(
                &node.spec,
                &self.state,
                &e.kind,
                &e.payload,
            ) {
                self.error = Some(error);
                continue;
            }
            let mut action = node.spec.events.get(&e.kind).cloned();
            let plugin_contract = node
                .metadata
                .registration
                .as_ref()
                .and_then(|r| r.definition.events.get(&e.kind))
                .cloned();
            if node.spec.kind == crate::protocol::Kind::Plugin {
                let Some(contract) = &plugin_contract else {
                    continue;
                };
                if let Err(err) = contract.validate(&e.payload) {
                    self.error = Some(err);
                    continue;
                }
            }
            let action_names = node.metadata.action_names.clone();
            let is_form = node.spec.kind == crate::protocol::Kind::Form;
            if crate::protocol::widgets::is_widget(node.spec.kind)
                && e.kind == "change"
                && let Err(error) =
                    crate::protocol::widgets::validate_value(&node.spec, &self.state, &e.payload)
            {
                self.error = Some(error);
                continue;
            }
            if node.spec.kind.is_input()
                && e.kind == "change"
                && let Err(error) =
                    crate::protocol::validate_input_value(&node.spec, &self.state, &e.payload)
            {
                self.error = Some(error);
                continue;
            }
            if node.spec.kind == crate::protocol::Kind::Splitter && e.kind == "change" {
                if node.spec.resizable == Some(false) {
                    continue;
                }
                if !e
                    .payload
                    .as_f64()
                    .is_some_and(|v| v.is_finite() && (0.0..=1.0).contains(&v))
                {
                    self.error = Some("splitter.change 必须为 [0,1] 的比例".into());
                    continue;
                }
            }
            if matches!(
                node.spec.kind,
                crate::protocol::Kind::Select | crate::protocol::Kind::Radio
            ) && e.kind == "change"
                && !crate::protocol::widgets::multiple(&node.spec, &self.state)
            {
                let allowed = e.payload.as_str() == Some("")
                    && node.spec.kind == crate::protocol::Kind::Select
                    && crate::protocol::widgets::flag(&node.spec, &self.state, "allowClear", false)
                    || crate::protocol::select_options(&node.spec, &self.state).is_ok_and(
                        |options| {
                            options.iter().any(|o| {
                                !o.disabled && e.payload.as_str() == Some(o.value.as_str())
                            })
                        },
                    );
                if !allowed {
                    self.error = Some("select.change 必须选择可用选项".into());
                    continue;
                }
            }
            let binding = if let Some(property) =
                crate::protocol::widgets::event_property(node.spec.kind, &e.kind)
            {
                node.spec.api.get(property).cloned()
            } else if e.kind == "change" {
                if node.spec.kind == crate::protocol::Kind::Text {
                    node.spec.text.clone()
                } else {
                    node.spec.value.clone()
                }
            } else {
                None
            };
            self.error = None;
            if is_form && matches!(e.kind.as_str(), "submit" | "reset") {
                match self.form_event(&e.component_id, e.kind == "reset") {
                    Ok(valid) => {
                        let kind = if valid { e.kind.as_str() } else { "invalid" };
                        action = self
                            .tree
                            .nodes
                            .iter()
                            .find(|n| n.spec.id == e.component_id)
                            .and_then(|n| n.spec.events.get(kind))
                            .cloned();
                    }
                    Err(error) => {
                        self.error = Some(error);
                        continue;
                    }
                }
            }
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
                        crate::protocol::EventAction::Fixed(_) => Ok(id.to_owned()),
                        crate::protocol::EventAction::Binding(_) => action_names
                            .get(id)
                            .cloned()
                            .ok_or_else(|| format!("当前作用域未定义动作 {id:?}")),
                    })
                    .and_then(|id| self.invoke_event(&id, Some(e.payload.clone()), 0));
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
