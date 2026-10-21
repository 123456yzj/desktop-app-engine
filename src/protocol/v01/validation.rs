use super::description::{Component, Description, Property, Step};
use std::collections::HashSet;

pub const MAX_STRING_BYTES: usize = 65_536;

pub fn validate(description: &Description) -> Result<(), String> {
    if description.schema_version != "0.1" {
        return Err(format!(
            "schemaVersion: 不支持版本 {:?}，仅支持 0.1",
            description.schema_version
        ));
    }
    nonempty(&description.app.name, "app.name")?;
    for (field, size) in [
        ("app.window.width", description.app.window.width),
        ("app.window.height", description.app.window.height),
    ] {
        if !size.is_finite() || !(160.0..=8192.0).contains(&size) {
            return Err(format!("{field}: 大小必须在 160..8192 之间"));
        }
    }
    for (key, value) in &description.state {
        nonempty(key, "state 的键")?;
        string_size(value, &format!("state[{key:?}]"))?;
    }
    validate_component(
        &description.root,
        "root",
        description,
        &mut HashSet::new(),
        0,
    )?;
    for (id, action) in &description.actions {
        let path = format!("actions[{id:?}]");
        nonempty(id, "actions 的键")?;
        if action.steps.is_empty() {
            return Err(format!("{path}.steps: 至少需要一个步骤"));
        }
        for (index, step) in action.steps.iter().enumerate() {
            let path = format!("{path}.steps[{index}]");
            let target = match step {
                Step::Set { target, value } => {
                    validate_property(value, &format!("{path}.value"), description)?;
                    target
                }
                Step::Concat { target, parts } => {
                    for (index, part) in parts.iter().enumerate() {
                        validate_property(part, &format!("{path}.parts[{index}]"), description)?;
                    }
                    target
                }
            };
            state_reference(target, &format!("{path}.target"), description)?;
        }
    }
    Ok(())
}

fn validate_component(
    node: &Component,
    path: &str,
    description: &Description,
    ids: &mut HashSet<String>,
    depth: usize,
) -> Result<(), String> {
    let path = format!("{path} (组件 {:?})", node.id());
    nonempty(node.id(), &format!("{path}.id"))?;
    if !ids.insert(node.id().to_owned()) {
        return Err(format!("{path}.id: 重复组件 ID"));
    }
    if depth > 64 {
        return Err(format!("{path}: 嵌套深度超过 64"));
    }
    match node {
        Component::Row { children, .. } | Component::Column { children, .. } => {
            for (index, child) in children.iter().enumerate() {
                validate_component(
                    child,
                    &format!("{path}.children[{index}]"),
                    description,
                    ids,
                    depth + 1,
                )?;
            }
        }
        Component::Text { text, .. } => {
            validate_property(text, &format!("{path}.text"), description)?
        }
        Component::Input {
            value, placeholder, ..
        } => {
            validate_property(value, &format!("{path}.value"), description)?;
            if let Some(placeholder) = placeholder {
                validate_property(placeholder, &format!("{path}.placeholder"), description)?;
            }
        }
        Component::Button { text, on_click, .. } => {
            validate_property(text, &format!("{path}.text"), description)?;
            if !description.actions.contains_key(on_click) {
                return Err(format!("{path}.onClick: 未定义动作 {on_click:?}"));
            }
        }
    }
    Ok(())
}

fn validate_property(
    property: &Property,
    path: &str,
    description: &Description,
) -> Result<(), String> {
    match property {
        Property::Literal { value } => string_size(value, path),
        Property::State { key } => state_reference(key, path, description),
    }
}

fn state_reference(key: &str, path: &str, description: &Description) -> Result<(), String> {
    if description.state.contains_key(key) {
        Ok(())
    } else {
        Err(format!("{path}: 未定义状态 {key:?}"))
    }
}

fn nonempty(value: &str, path: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{path}: 不能为空"))
    } else {
        Ok(())
    }
}

pub fn string_size(value: &str, path: &str) -> Result<(), String> {
    if value.len() > MAX_STRING_BYTES {
        Err(format!("{path}: 字符串超过 {MAX_STRING_BYTES} UTF-8 字节"))
    } else {
        Ok(())
    }
}
