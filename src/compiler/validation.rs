//! Semantic validation over resolved names and plugin registrations.
use super::{Metadata, action_uses_event, listener, module_node};
use crate::{
    binding::{Binding, ValueType},
    v02::*,
};
use std::collections::{BTreeMap, HashSet};

pub(crate) fn validate_flat(d: &Description, metadata: &Metadata) -> Result<(), String> {
    if d.schema_version != "0.2" {
        return Err("schemaVersion: 仅支持 0.2".into());
    }
    if d.app.name.trim().is_empty() {
        return Err("app.name: 不能为空".into());
    }
    for (field, n) in [
        ("width", d.app.window.width),
        ("height", d.app.window.height),
    ] {
        if !n.is_finite() || !(160.0..=8192.0).contains(&n) {
            return Err(format!("app.window.{field}: 必须在 160..8192"));
        }
    }
    for (key, v) in &d.state {
        if key.trim().is_empty() {
            return Err("state 键不能为空".into());
        }
        ValueType::of(v).map_err(|e| format!("state.{key}: {e}"))?;
    }
    validate_node(&d.root, d, metadata, &mut HashSet::new(), 0)?;
    if d.data_sources.len() > 64 {
        return Err("dataSources 最多 64 个".into());
    }
    let mut targets = HashSet::new();
    if let Some(name) = &d.credential_store {
        crate::credentials::valid_name(name)?;
    }
    for (name, source) in &d.data_sources {
        if source.requires_credentials() && d.credential_store.is_none() {
            return Err("凭据请求头必须配置顶层 credentialStore".into());
        }
        if name.trim().is_empty() {
            return Err("dataSource 名称不得为空".into());
        }
        source
            .validate(&d.state)
            .map_err(|e| format!("dataSources.{name}: {e}"))?;
        for target in
            std::iter::once(&source.status_target).chain(source.mappings.iter().map(|m| &m.target))
        {
            if !targets.insert(target) {
                return Err(format!("dataSources 不能共享写入目标 {target:?}"));
            }
        }
    }
    let mut contexts: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    fn collect(
        n: &Node,
        d: &Description,
        metadata: &Metadata,
        contexts: &mut BTreeMap<String, Vec<serde_json::Value>>,
    ) -> Result<(), String> {
        for (event, sample) in &n.emits {
            if let Some(id) = listener(n, metadata, event, &d.state)? {
                if !d.actions.contains_key(&id) {
                    return Err(format!("on.{event}: 未定义动作 {id:?}"));
                }
                contexts.entry(id).or_default().push(sample.clone());
            }
        }
        for c in &n.children {
            collect(&c.0, d, metadata, contexts)?;
        }
        Ok(())
    }
    collect(&d.root, d, metadata, &mut contexts)?;
    for (id, a) in &d.actions {
        if id.trim().is_empty() || a.steps.is_empty() {
            return Err(format!("actions.{id}: ID 和 steps 不得为空"));
        }
        let mut known_samples = metadata.actions[id].event_samples.clone();
        known_samples.extend(contexts.get(id).into_iter().flatten().cloned());
        let samples = if known_samples.is_empty() {
            None
        } else {
            Some(&known_samples)
        };
        if let Some(declared) = &a.event_payload {
            let contract = ValueType::of(declared)?;
            for sample in samples.into_iter().flatten() {
                if !contract.accepts(sample) {
                    return Err(format!("actions.{id}.eventPayload: 事件载荷类型不匹配"));
                }
            }
        }
        if action_uses_event(a) && samples.is_none() && a.event_payload.is_none() {
            return Err(format!(
                "actions.{id}: event 绑定需要模块 on 事件上下文或 eventPayload 类型声明"
            ));
        }
        let payloads: Vec<_> = if action_uses_event(a) {
            let mut values: Vec<_> = samples.into_iter().flatten().map(Some).collect();
            if let Some(sample) = &a.event_payload {
                values.push(Some(sample));
            }
            values
        } else {
            vec![None]
        };
        for payload in payloads {
            for (i, step) in a.steps.iter().enumerate() {
                let result = (|| {
                    let (target, value) = match step {
                        Step::OpenLogs => return Ok(()),
                        Step::Request { source } => {
                            if !d.data_sources.contains_key(source) {
                                return Err(format!("未定义 dataSource {source:?}"));
                            }
                            return Ok(());
                        }
                        Step::PluginCall {
                            target,
                            method,
                            args,
                            result_target,
                        } => {
                            module_node(&d.root, target).ok_or("pluginCall target 组件不存在")?;
                            let r = metadata.nodes[target]
                                .registration
                                .as_ref()
                                .ok_or("pluginCall target 必须为插件组件")?;
                            let m = r.definition.methods.get(method).ok_or("插件未声明该方法")?;
                            m.args.validate(args.resolve_event(&d.state, payload)?)?;
                            if let Some(key) = result_target {
                                m.result
                                    .validate(d.state.get(key).ok_or("resultTarget 状态不存在")?)?;
                            }
                            return Ok(());
                        }
                        Step::Emit {
                            event,
                            payload: value,
                        } => {
                            let module = metadata.actions[id]
                                .module
                                .as_ref()
                                .ok_or("emit 缺少模块作用域")?;
                            let n = module_node(&d.root, module).ok_or("emit 模块不存在")?;
                            let sample = n
                                .emits
                                .get(event)
                                .ok_or_else(|| format!("未声明 emits 事件 {event:?}"))?;
                            if !ValueType::of(sample)?
                                .accepts(value.resolve_event(&d.state, payload)?)
                            {
                                return Err(format!("emit {event:?}: 载荷类型不匹配"));
                            }
                            return Ok(());
                        }
                        Step::Command {
                            program,
                            args,
                            cwd,
                            target,
                            timeout_ms,
                            on_complete,
                        } => {
                            if let Some(id) = on_complete {
                                let callback = d.actions.get(id).ok_or("onComplete 动作不存在")?;
                                if action_uses_event(callback) || callback.event_payload.is_some() {
                                    return Err("onComplete 动作不能要求 event 载荷".into());
                                }
                            }
                            return crate::command::validate_event(
                                program,
                                args,
                                cwd,
                                target,
                                *timeout_ms,
                                &d.state,
                                payload,
                            );
                        }
                        Step::Set { target, value } => {
                            (target, value.resolve_event(&d.state, payload)?.clone())
                        }
                        Step::ParseJson { target, value } => {
                            if !d.state.contains_key(target) {
                                return Err("parseJson target 状态不存在".into());
                            }
                            if !value.resolve_event(&d.state, payload)?.is_string() {
                                return Err("parseJson value 必须为字符串".into());
                            }
                            return Ok(());
                        }
                        Step::Concat { target, parts } => {
                            let mut s = String::new();
                            for p in parts {
                                s.push_str(
                                    p.resolve_event(&d.state, payload)?
                                        .as_str()
                                        .ok_or("concat 仅接受字符串")?,
                                );
                            }
                            (target, serde_json::Value::String(s))
                        }
                    };
                    let initial = d
                        .state
                        .get(target)
                        .ok_or_else(|| format!("未定义状态 {target:?}"))?;
                    if !ValueType::of(initial)?.accepts(&value) {
                        return Err("赋值类型不匹配".into());
                    }
                    Ok::<_, String>(())
                })();
                result.map_err(|e| format!("actions.{id}.steps[{i}]: {e}"))?;
            }
        }
    }
    Ok(())
}

fn validate_node(
    n: &Node,
    d: &Description,
    metadata: &Metadata,
    ids: &mut HashSet<String>,
    depth: usize,
) -> Result<(), String> {
    let result = (|| {
        if depth > 64 || n.id.trim().is_empty() || !ids.insert(n.id.clone()) {
            return Err("ID 为空、重复组件 ID 或深度超过 64".into());
        }
        let compiled = &metadata.nodes[&n.id];
        for b in [&n.visible, &n.enabled].into_iter().flatten() {
            if !b.resolve(&d.state)?.is_boolean() {
                return Err("visible/enabled 必须为布尔值".into());
            }
        }
        for prop in n.props.values() {
            let value = prop.resolve(&d.state)?;
            if n.kind != Kind::Plugin {
                ValueType::of(value)?;
            }
        }
        let l = &n.layout;
        for t in [&l.width, &l.height].into_iter().flatten() {
            t.validate()?;
        }
        for v in [
            Some(l.padding),
            Some(l.gap),
            l.row_gap,
            l.column_gap,
            l.min_width,
            l.max_width,
            l.min_height,
            l.max_height,
            l.shrink,
            Some(n.style.border_width),
            Some(n.style.radius),
            n.style.font_size,
            n.style.padding_x,
            n.style.padding_y,
        ]
        .into_iter()
        .flatten()
        {
            if !v.is_finite() || !(0.0..=1_000_000.0).contains(&v) {
                return Err("layout/style: 尺寸必须有限且在 0..1000000".into());
            }
        }
        if l.min_width.unwrap_or(0.0) > l.max_width.unwrap_or(f32::MAX)
            || l.min_height.unwrap_or(0.0) > l.max_height.unwrap_or(f32::MAX)
        {
            return Err("layout: min > max".into());
        }
        if l.wrap
            && !(n.kind == Kind::Row || matches!(n.kind, Kind::Flex | Kind::Space) && !n.vertical)
        {
            return Err("layout.wrap 仅用于 row 或横向 flex/space".into());
        }
        for (event, action) in &n.events {
            let allowed: &[&str] = match n.kind {
                Kind::Button => &["click"],
                Kind::Input => &["change", "submit"],
                Kind::Select => &["change", "open"],
                Kind::Splitter => &["change"],
                Kind::Table => &["rowSelect", "sortChange"],
                _ => &[],
            };
            if !allowed.contains(&event.as_str())
                && !(n.kind == Kind::Plugin
                    && compiled
                        .registration
                        .as_ref()
                        .is_some_and(|r| r.definition.events.contains_key(event)))
            {
                return Err(format!("不支持事件 {event:?}"));
            }
            let resolved = action
                .resolve(&d.state)
                .map_err(|e| format!("events.{event}: {e}"))?;
            let action = match action {
                EventAction::Fixed(_) => resolved,
                EventAction::Binding(_) => compiled
                    .action_names
                    .get(resolved)
                    .map(String::as_str)
                    .ok_or_else(|| format!("当前作用域未定义动作 {resolved:?}"))?,
            };
            if !d.actions.contains_key(action) {
                return Err(format!("未定义动作 {action:?}"));
            }
            if n.kind == Kind::Plugin {
                let contract = &compiled
                    .registration
                    .as_ref()
                    .ok_or("缺少插件注册")?
                    .definition
                    .events[event];
                contract.validate(
                    d.actions[action]
                        .event_payload
                        .as_ref()
                        .ok_or("插件事件动作必须声明 eventPayload")?,
                )?;
            } else if action_uses_event(&d.actions[action])
                || d.actions[action].event_payload.is_some()
            {
                return Err("event 处理动作不可直接绑定控件事件，请通过 emit 调用".into());
            }
        }
        for b in [&n.text, &n.label, &n.placeholder, &n.title]
            .into_iter()
            .flatten()
        {
            if !b.resolve(&d.state)?.is_string() {
                return Err("文本属性必须为字符串".into());
            }
        }
        match n.kind {
            Kind::Splitter => {
                if n.children.len() != 2 {
                    return Err("splitter.children 必须恰好包含两个面板（可嵌套 splitter）".into());
                }
                if n.bar_size
                    .is_some_and(|v| !v.is_finite() || !(4.0..=32.0).contains(&v))
                {
                    return Err("splitter.barSize 必须在 4..32".into());
                }
                if let Some(b) = &n.value {
                    let ratio = b
                        .resolve(&d.state)?
                        .as_f64()
                        .ok_or("splitter.value 必须为数字")?;
                    if !ratio.is_finite() || !(0.0..=1.0).contains(&ratio) {
                        return Err("splitter.value 必须在 [0,1]".into());
                    }
                    if matches!(b, Binding::State { path, .. } if !path.is_empty()) {
                        return Err("splitter.value 仅支持顶层状态（path 为空）".into());
                    }
                }
            }
            Kind::Masonry => {
                if !(1..=64).contains(&n.column_count.unwrap_or(3)) {
                    return Err("masonry.columnCount 必须在 1..64".into());
                }
                if n.min_column_width
                    .is_some_and(|v| !v.is_finite() || v <= 0.0 || v > 1_000_000.0)
                {
                    return Err("masonry.minColumnWidth 必须大于 0 且不超过 1000000".into());
                }
            }
            Kind::Divider if n.vertical && n.text.is_some() => {
                return Err("纵向 divider 不接受 text".into());
            }
            Kind::Plugin => {
                compiled
                    .registration
                    .as_ref()
                    .ok_or("缺少插件注册")?
                    .definition
                    .props
                    .validate(&plugin_props(n, &d.state)?)?;
            }
            Kind::Text | Kind::Button | Kind::Badge => {
                required(&n.text, "text")?;
            }
            Kind::Input | Kind::Select => {
                let b = required(&n.value, "value")?;
                if !b.resolve(&d.state)?.is_string() {
                    return Err("input.value 必须为字符串".into());
                }
                if matches!(b,Binding::State {path,..} if !path.is_empty()) {
                    return Err("input.value 暂仅支持顶层状态（path 为空）".into());
                }
                if n.kind == Kind::Select {
                    select_options(n, &d.state)?;
                }
            }
            Kind::Metric | Kind::Progress => {
                let v = required(&n.value, "value")?
                    .resolve(&d.state)?
                    .as_f64()
                    .ok_or("value 必须为数字")?;
                if n.kind == Kind::Progress && !(0.0..=1.0).contains(&v) {
                    return Err("progress.value 必须在 [0,1]".into());
                }
                if n.kind == Kind::Metric {
                    required(&n.label, "label")?;
                    n.format
                        .clone()
                        .unwrap_or_default()
                        .apply(&serde_json::json!(v))?;
                }
            }
            Kind::Grid => {
                if n.columns.is_empty() || n.columns.len() > 64 {
                    return Err("grid.columns 数量必须在 1..64".into());
                }
                for c in &n.columns {
                    match c {
                        ColumnSpec::Track(t) if t.auto.is_none() => t.validate()?,
                        _ => return Err("grid.columns 仅接受 fixed/fr".into()),
                    }
                }
            }
            Kind::Table => crate::table::validate(n, &d.state)?,
            Kind::LineChart | Kind::BarChart => crate::chart::validate(n, &d.state)?,
            _ => {}
        }
        for child in &n.children {
            validate_node(&child.0, d, metadata, ids, depth + 1)?;
        }
        Ok::<_, String>(())
    })();
    result.map_err(|e| format!("组件 {:?}: {e}", n.id))
}
