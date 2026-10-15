//! Resolve lexical module names once, before runtime. Internal names cannot be authored.
use crate::{
    binding::{Binding, State},
    v02::{Action, Description, EventAction, Kind, Node, Step},
};
use std::collections::{BTreeMap, HashSet};

const PREFIX: &str = "@scope/";
#[derive(Clone, Default)]
struct Names {
    state: BTreeMap<String, String>,
    actions: BTreeMap<String, String>,
    props: BTreeMap<String, Binding>,
    module: Option<String>,
}
fn valid_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() || name.starts_with(PREFIX) {
        return Err(format!("名称 {name:?} 为空或使用保留前缀 {PREFIX}"));
    }
    Ok(())
}
fn internal(id: &str, name: &str) -> String {
    format!("{PREFIX}{}:{id}/{name}", id.len())
}
fn lookup(map: &BTreeMap<String, String>, name: &str, kind: &str) -> Result<String, String> {
    valid_name(name)?;
    map.get(name)
        .cloned()
        .ok_or_else(|| format!("当前作用域未定义{kind} {name:?}"))
}
fn binding(b: &mut Binding, names: &Names, allow_event: bool) -> Result<(), String> {
    match b {
        Binding::State { key, .. } => *key = lookup(&names.state, key, "状态")?,
        Binding::Prop { key, path } => {
            valid_name(key)?;
            if !crate::binding::valid_pointer(path) {
                return Err("prop.path: 非法 RFC 6901 pointer".into());
            }
            let source = names
                .props
                .get(key)
                .ok_or_else(|| format!("当前作用域未定义 prop {key:?}"))?;
            *b = match source {
                Binding::State { key, path: base } => Binding::State {
                    key: key.clone(),
                    path: format!("{base}{path}"),
                },
                Binding::Literal { value } => Binding::Literal {
                    value: value
                        .pointer(path)
                        .ok_or_else(|| format!("prop {key:?}: 缺少字段 {path:?}"))?
                        .clone(),
                },
                _ => return Err("prop 来源未解析".into()),
            };
        }
        Binding::Event { .. } if !allow_event => return Err("event 绑定仅可用于动作步骤".into()),
        _ => {}
    }
    Ok(())
}
fn write_target(target: &str, names: &Names) -> Result<String, String> {
    if names.props.contains_key(target) {
        return Err(format!("prop {target:?} 为只读，不可写入"));
    }
    lookup(&names.state, target, "状态")
}
fn event(e: &mut EventAction, names: &Names, state: &State) -> Result<(), String> {
    match e {
        EventAction::Fixed(id) => *id = lookup(&names.actions, id, "动作")?,
        EventAction::Binding(b) => {
            binding(b, names, false)?;
            let initial = b
                .resolve(state)?
                .as_str()
                .ok_or("事件动作绑定必须为字符串")?;
            lookup(&names.actions, initial, "动作")?;
            // Keep authored IDs in state; runtime resolves them through this node's action map.
        }
    }
    Ok(())
}
fn action(mut action: Action, names: &Names) -> Result<Action, String> {
    for step in &mut action.steps {
        match step {
            Step::OpenLogs => {}
            Step::Request { source } => {
                valid_name(source)?;
            }
            Step::PluginCall {
                args,
                result_target,
                ..
            } => {
                binding(args, names, true)?;
                if let Some(target) = result_target {
                    *target = write_target(target, names)?;
                }
            }
            Step::Emit {
                module,
                payload,
                event,
            } => {
                valid_name(event)?;
                *module = Some(
                    names
                        .module
                        .clone()
                        .ok_or("emit 动作必须声明在 module 内")?,
                );
                binding(payload, names, true)?;
            }
            Step::Set { target, value } | Step::ParseJson { target, value } => {
                *target = write_target(target, names)?;
                binding(value, names, true)?;
            }
            Step::Concat { target, parts } => {
                *target = write_target(target, names)?;
                for p in parts {
                    binding(p, names, true)?;
                }
            }
            Step::Command {
                target,
                args,
                on_complete,
                ..
            } => {
                *target = write_target(target, names)?;
                if let Some(id) = on_complete {
                    *id = lookup(&names.actions, id, "动作")?;
                }
                for arg in args {
                    if let crate::command::Argument::Binding(b) = arg {
                        binding(b, names, true)?;
                    }
                }
            }
        }
    }
    Ok(action)
}

pub fn compile(mut d: Description) -> Result<Description, String> {
    let names = Names {
        state: d
            .state
            .keys()
            .map(|k| {
                Ok((k.clone(), {
                    valid_name(k)?;
                    k.clone()
                }))
            })
            .collect::<Result<_, String>>()?,
        actions: d
            .actions
            .keys()
            .map(|k| {
                Ok((k.clone(), {
                    valid_name(k)?;
                    k.clone()
                }))
            })
            .collect::<Result<_, String>>()?,
        ..Names::default()
    };
    let mut actions = BTreeMap::new();
    for (id, source) in &mut d.data_sources {
        valid_name(id)?;
        for b in source.bindings_mut() {
            binding(b, &names, false)?;
        }
        source.status_target = write_target(&source.status_target, &names)?;
        for mapping in &mut source.mappings {
            mapping.target = write_target(&mapping.target, &names)?;
        }
    }
    for (id, a) in std::mem::take(&mut d.actions) {
        actions.insert(id, action(a, &names)?);
    }
    fn node(
        n: &mut Node,
        parent: &Names,
        state: &mut State,
        actions: &mut BTreeMap<String, Action>,
        ids: &mut HashSet<String>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 64 || n.id.trim().is_empty() || !ids.insert(n.id.clone()) {
            return Err("ID 为空、重复组件 ID 或深度超过 64".into());
        }
        if (!n.state.is_empty() || !n.actions.is_empty())
            && !matches!(
                n.kind,
                Kind::Module | Kind::Row | Kind::Column | Kind::Grid | Kind::Panel | Kind::Card
            )
        {
            return Err("只有容器可声明局部 state/actions".into());
        }
        let mut names = parent.clone();
        if n.kind != Kind::Module
            && n.kind != Kind::Plugin
            && (!n.props.is_empty() || !n.emits.is_empty() || !n.on.is_empty())
        {
            return Err("只有 module 可声明 props/emits/on".into());
        }
        if n.kind == Kind::Module {
            names.module = Some(n.id.clone());
        }
        for (key, b) in &mut n.props {
            valid_name(key)?;
            if n.state.contains_key(key) {
                return Err(format!("prop 与局部 state 同名 {key:?}"));
            }
            binding(b, parent, false)?;
            b.resolve(state)?;
            if n.kind == Kind::Module {
                names.props.insert(key.clone(), b.clone());
                names.state.remove(key);
            }
        }
        n.listener_action_names = parent.actions.clone();
        for (name, sample) in &n.emits {
            valid_name(name)?;
            crate::binding::ValueType::of(sample)?;
        }
        for (name, handler) in &mut n.on {
            if !n.emits.contains_key(name) {
                return Err(format!("on.{name}: 未声明 emits 事件"));
            }
            event(handler, parent, state)?;
        }
        for (key, value) in std::mem::take(&mut n.state) {
            valid_name(&key)?;
            let resolved = internal(&n.id, &key);
            names.props.remove(&key);
            names.state.insert(key, resolved.clone());
            state.insert(resolved, value);
        }
        let local_actions = std::mem::take(&mut n.actions);
        for id in local_actions.keys() {
            valid_name(id)?;
            names.actions.insert(id.clone(), internal(&n.id, id));
        }
        for (id, a) in local_actions {
            actions.insert(names.actions[&id].clone(), action(a, &names)?);
        }
        n.action_names = names.actions.clone();
        n.state_names = names.state.clone();
        if matches!(n.kind, Kind::Input | Kind::Select)
            && matches!(n.value, Some(Binding::Prop { .. }))
        {
            return Err("prop 为只读，input.value 请绑定局部 state 并通过 emit 通知父级".into());
        }
        for b in [
            &mut n.visible,
            &mut n.enabled,
            &mut n.text,
            &mut n.title,
            &mut n.label,
            &mut n.value,
            &mut n.placeholder,
            &mut n.rows,
            &mut n.options,
        ]
        .into_iter()
        .flatten()
        {
            binding(b, &names, false)?;
        }
        for series in n.series.iter_mut().flatten() {
            binding(&mut series.data, &names, false)?;
        }
        for e in n.events.values_mut() {
            event(e, &names, state)?;
        }
        for child in &mut n.children {
            node(&mut child.0, &names, state, actions, ids, depth + 1)
                .map_err(|e| format!("组件 {:?}: {e}", child.0.id))?;
        }
        Ok(())
    }
    node(
        &mut d.root,
        &names,
        &mut d.state,
        &mut actions,
        &mut HashSet::new(),
        0,
    )?;
    d.actions = actions;
    // Retain initial listener contracts when a dynamic listener switches away later.
    fn infer(
        n: &Node,
        state: &State,
        actions: &mut BTreeMap<String, Action>,
    ) -> Result<(), String> {
        for (name, sample) in &n.emits {
            if let Some(id) = crate::v02::listener(n, name, state)? {
                let action = actions
                    .get_mut(&id)
                    .ok_or_else(|| format!("on.{name}: 未定义动作 {id:?}"))?;
                action.event_samples.push(sample.clone());
            }
        }
        for c in &n.children {
            infer(&c.0, state, actions)?;
        }
        Ok(())
    }
    infer(&d.root, &d.state, &mut d.actions)?;
    Ok(d)
}
