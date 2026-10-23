use super::{Binding, Runtime};
use crate::v02::{self, Kind};
use std::collections::BTreeMap;

impl Runtime {
    /// Field diagnostics appear after the first submit and are refreshed on state changes.
    pub fn form_error(&self, item_id: &str) -> Option<&str> {
        self.form_errors.get(item_id).map(String::as_str)
    }

    fn form_fields(&self, form: usize) -> Vec<usize> {
        self.tree
            .nodes
            .iter()
            .enumerate()
            .filter(|(id, n)| {
                n.spec.kind == Kind::FormItem && self.tree.ancestor(*id, Kind::Form) == Some(form)
            })
            .map(|(id, _)| id)
            .collect()
    }

    pub(super) fn refresh_form_errors(&mut self) -> Result<(), String> {
        self.form_errors.clear();
        if self.submitted_forms.is_empty() {
            return Ok(());
        }
        let resolved = self.tree.resolve(&self.state)?;
        for (form, node) in self.tree.nodes.iter().enumerate() {
            if !self.submitted_forms.contains(&node.spec.id) {
                continue;
            }
            for id in self.form_fields(form) {
                let item = &self.tree.nodes[id];
                let child = item.children[0];
                if !resolved[child].visible || !resolved[child].enabled {
                    continue;
                }
                let value = self.tree.nodes[child]
                    .spec
                    .value
                    .as_ref()
                    .unwrap()
                    .resolve(&self.state)?;
                if let Some(error) = v02::field_error(&item.spec, value) {
                    self.form_errors.insert(item.spec.id.clone(), error);
                }
            }
        }
        Ok(())
    }

    pub(super) fn form_event(&mut self, form_id: &str, reset: bool) -> Result<bool, String> {
        let form = self
            .tree
            .nodes
            .iter()
            .position(|n| n.spec.id == form_id && n.spec.kind == Kind::Form)
            .ok_or("未定义 form")?;
        let fields = self.form_fields(form);
        if reset {
            let mut updates = BTreeMap::new();
            for &id in &fields {
                let child = self.tree.nodes[id].children[0];
                if let Some(Binding::State { key, .. }) = &self.tree.nodes[child].spec.value {
                    updates.insert(key.clone(), self.description.state[key].clone());
                }
            }
            self.set_many(updates)?;
            self.submitted_forms.remove(form_id);
            self.refresh_form_errors()?;
            Ok(true)
        } else {
            self.submitted_forms.insert(form_id.to_owned());
            self.refresh_form_errors()?;
            Ok(fields
                .iter()
                .all(|&id| !self.form_errors.contains_key(&self.tree.nodes[id].spec.id)))
        }
    }
}
