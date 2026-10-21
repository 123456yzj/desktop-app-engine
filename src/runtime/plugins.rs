//! Plugin lifecycle and validated async results for the active runtime.
use super::{Runtime, Value, compiler};

impl Runtime {
    pub fn sync_plugins(&mut self, visible: Vec<(String, u32, u32)>) -> Result<(), String> {
        for (id, width, height) in &visible {
            let n = compiler::module_node(&self.description.root, id).ok_or("组件不存在")?;
            self.plugins.sync(
                id,
                self.metadata.nodes[id]
                    .registration
                    .as_ref()
                    .ok_or("缺少插件注册")?,
                crate::v02::plugin_props(n, &self.state)?,
                *width,
                *height,
            )?;
        }
        self.plugins
            .retain(&visible.into_iter().map(|(id, _, _)| id).collect::<Vec<_>>());
        Ok(())
    }
    pub fn poll_plugins(&mut self) {
        self.plugins.poll();
        for completion in std::mem::take(&mut self.plugins.completed) {
            let result = (|| {
                let response = completion.response?;
                let r = self
                    .metadata
                    .nodes
                    .get(&completion.id)
                    .and_then(|n| n.registration.as_ref())
                    .ok_or("插件未注册")?
                    .clone();
                if let Some(method) = completion.method {
                    let result = response.get("result").ok_or("插件方法未返回 result")?;
                    r.definition
                        .methods
                        .get(&method)
                        .ok_or("未知插件方法")?
                        .result
                        .validate(result)?;
                    if let Some(target) = completion.target {
                        self.set(&target, result.clone())?;
                    }
                }
                if let Some(image) = completion.image {
                    self.plugin_images.insert(completion.id.clone(), image);
                }
                if let Some(events) = response.get("events") {
                    let resolved = self.tree.resolve(&self.state)?;
                    let index = self
                        .tree
                        .nodes
                        .iter()
                        .position(|n| n.spec.id == completion.id)
                        .ok_or("插件组件不存在")?;
                    if !resolved[index].visible || !resolved[index].enabled {
                        return Ok(());
                    }
                    for event in events.as_array().ok_or("插件 events 必须为数组")? {
                        let name = event
                            .get("name")
                            .and_then(Value::as_str)
                            .ok_or("缺少事件 name")?;
                        let payload = event.get("payload").ok_or("缺少事件 payload")?;
                        r.definition
                            .events
                            .get(name)
                            .ok_or("未声明插件事件")?
                            .validate(payload)?;
                        let n =
                            compiler::module_node(&self.description.root, &completion.id).unwrap();
                        if let Some(action) = n.events.get(name) {
                            let id = action.resolve(&self.state)?;
                            let id = match action {
                                crate::v02::EventAction::Fixed(_) => id.to_owned(),
                                _ => self.metadata.nodes[&n.id]
                                    .action_names
                                    .get(id)
                                    .ok_or("未定义事件动作")?
                                    .clone(),
                            };
                            self.invoke_event(&id, Some(payload.clone()), 0)?;
                        }
                    }
                }
                Ok::<_, String>(())
            })();
            if let Err(e) = result {
                self.error = Some(format!("插件 {}: {e}", completion.id));
            }
        }
    }
}
