//! Common normalization and value validation. Component files own their API declarations.
pub(crate) use super::catalogue::PropertyType;
use crate::binding::{Binding, State};
use crate::protocol::*;
use PropertyType::*;
use serde_json::{Value, json};
pub(crate) const SIZE: PropertyType = Enum(&["small", "medium", "large"]);

pub(crate) fn is_widget(kind: Kind) -> bool {
    !matches!(
        kind,
        Kind::Plugin
            | Kind::Module
            | Kind::Row
            | Kind::Column
            | Kind::Grid
            | Kind::Panel
            | Kind::Card
            | Kind::Flex
            | Kind::Space
            | Kind::Divider
            | Kind::Layout
            | Kind::Header
            | Kind::Sider
            | Kind::Content
            | Kind::Footer
            | Kind::Splitter
            | Kind::Masonry
            | Kind::Text
            | Kind::Input
            | Kind::Select
            | Kind::Checkbox
            | Kind::Radio
            | Kind::Switch
            | Kind::InputNumber
            | Kind::Slider
            | Kind::Form
            | Kind::FormItem
            | Kind::Button
            | Kind::Metric
            | Kind::Badge
            | Kind::Progress
            | Kind::Table
            | Kind::LineChart
            | Kind::BarChart
    )
}
pub(crate) fn properties(kind: Kind) -> Vec<(&'static str, PropertyType)> {
    let mut properties = vec![("disabled", Bool), ("size", SIZE)];
    properties.extend_from_slice(super::catalogue::definition(kind).properties);
    properties
}
pub(crate) fn primary(kind: Kind) -> Option<&'static str> {
    super::catalogue::definition(kind).primary
}
pub(crate) fn events(kind: Kind) -> &'static [&'static str] {
    super::catalogue::definition(kind).events
}

pub(crate) fn normalize(value: &mut Value, kind: Kind) -> Result<(), String> {
    if value.get("api").is_some() {
        return Err("api 为内部字段，请直接声明组件属性".into());
    }
    let object = value.as_object_mut().ok_or("组件必须为对象")?;
    let mut api = serde_json::Map::new();
    for (name, _) in properties(kind) {
        if let Some(v) = object.remove(name) {
            let binding = if v.get("kind").is_some() {
                v
            } else {
                json!({"kind":"literal", "value":v})
            };
            api.insert(name.into(), binding);
        }
    }
    if is_widget(kind) {
        for name in ["text", "title", "label", "placeholder", "value", "options"] {
            if let Some(v) = object.get_mut(name)
                && v.get("kind").is_none()
            {
                *v = json!({"kind":"literal", "value":v.clone()});
            }
        }
        if let Some(name) = primary(kind) {
            if let Some(binding) = api.get(name) {
                if object.contains_key("value") {
                    return Err(format!("value 与 {name} 不可同时声明"));
                }
                object.insert("value".into(), binding.clone());
            } else if !object.contains_key("value") {
                let default_name = format!("default{}{}", name[..1].to_uppercase(), &name[1..]);
                let default = api.get(&default_name).cloned().unwrap_or_else(|| {
                    let ty = properties(kind)
                        .into_iter()
                        .find(|(key, _)| *key == name)
                        .unwrap()
                        .1;
                    let value = match ty {
                        Integer(min, _) => json!(min),
                        Array => json!([]),
                        _ => json!(""),
                    };
                    json!({"kind":"literal", "value":value})
                });
                object.insert("value".into(), default);
            }
        }
    }
    if !api.is_empty() {
        object.insert("api".into(), Value::Object(api));
    }
    Ok(())
}
pub(crate) fn validate(n: &Node, state: &State) -> Result<(), String> {
    for (name, binding) in &n.api {
        let (_, ty) = properties(n.kind)
            .into_iter()
            .find(|(key, _)| key == name)
            .ok_or_else(|| format!("不支持属性 {name:?}"))?;
        let v = binding.resolve(state)?;
        if !property_valid(ty, v)? {
            return Err(format!("属性 {name:?} 的类型或取值非法"));
        }
        if matches!(
            name.as_str(),
            "open"
                | "pageSize"
                | "current"
                | "checked"
                | "openKeys"
                | "checkedKeys"
                | "expandedKeys"
        ) && matches!(binding, Binding::State { path, .. } if !path.is_empty())
        {
            return Err(format!("可写属性 {name} 的 state.path 必须为空"));
        }
        if matches!(
            name.as_str(),
            "openKeys"
                | "checkedKeys"
                | "expandedKeys"
                | "selectedKeys"
                | "defaultSelectedKeys"
                | "defaultOpenKeys"
                | "defaultExpandedKeys"
        ) {
            validate_keys(v)?;
            validate_item_keys(n, state, v)?;
        }
        if name == "pageSizeOptions"
            && !v.as_array().is_some_and(|a| {
                !a.is_empty()
                    && a.iter()
                        .all(|x| x.as_u64().is_some_and(|x| (1..=1000000).contains(&x)))
            })
        {
            return Err("pageSizeOptions 必须为正整数数组".into());
        }
        if name == "options" && n.kind == Kind::Checkbox {
            validate_options(v, false)?;
        }
    }
    if is_widget(n.kind) {
        if let Some(options) = &n.options {
            validate_options(
                options.resolve(state)?,
                matches!(n.kind, Kind::Cascader | Kind::TreeSelect),
            )?;
        }
        for name in ["minDate", "maxDate"] {
            if let Some(s) = property(n, state, name).and_then(Value::as_str) {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|_| format!("{name} 必须为有效的 YYYY-MM-DD 日期"))?;
            }
        }
        if let (Some(min), Some(max)) = (
            property(n, state, "minDate").and_then(Value::as_str),
            property(n, state, "maxDate").and_then(Value::as_str),
        ) && min > max
        {
            return Err("minDate 不可大于 maxDate".into());
        }
        if let Some(b) = &n.value {
            if matches!(b, Binding::State { path, .. } if !path.is_empty()) {
                return Err("可写 value 的 state.path 必须为空".into());
            }
            validate_value(n, state, b.resolve(state)?)?;
        } else if n.kind.is_input() || n.kind == Kind::QrCode {
            return Err("缺少 value".into());
        }
        if matches!(n.kind, Kind::Tabs | Kind::Collapse) {
            let items = items(n, state);
            for child in &n.children {
                if !items
                    .iter()
                    .any(|item| item["key"].as_str() == Some(child.0.id.as_str()))
                {
                    return Err(format!(
                        "{:?} 子组件 {:?} 必须在 items 中有对应 key",
                        n.kind, child.0.id
                    ));
                }
            }
        }
    }
    if matches!(n.kind, Kind::Input | Kind::Select)
        && let Some(binding) = &n.value
    {
        validate_input_value(n, state, binding.resolve(state)?)?;
    }
    Ok(())
}
fn property_valid(ty: PropertyType, v: &Value) -> Result<bool, String> {
    Ok(match ty {
        Bool => v.is_boolean(),
        Text => v
            .as_str()
            .is_some_and(|s| s.chars().count() <= crate::MAX_INPUT_CHARS),
        Number(min, max) => v
            .as_f64()
            .is_some_and(|x| x.is_finite() && (min..=max).contains(&x)),
        Integer(min, max) => v.as_u64().is_some_and(|x| (min..=max).contains(&x)),
        Enum(options) => v.as_str().is_some_and(|x| options.contains(&x)),
        Array => v.as_array().is_some_and(|a| a.len() <= 10000),
        Items => {
            validate_items(v)?;
            true
        }
    })
}
pub(crate) fn validate_keys(value: &Value) -> Result<(), String> {
    let values = value.as_array().ok_or("键值必须为字符串数组")?;
    let mut seen = std::collections::BTreeSet::new();
    if values.len() > 10000
        || !values
            .iter()
            .all(|v| v.as_str().is_some_and(|s| !s.is_empty() && seen.insert(s)))
    {
        return Err("键值必须为不重复、非空的字符串数组".into());
    }
    Ok(())
}
fn validate_options(value: &Value, nested: bool) -> Result<(), String> {
    let options = value.as_array().ok_or("options 必须为数组")?;
    let normalized: Vec<_> = options
        .iter()
        .map(|v| {
            if v.is_string() {
                json!({"value":v})
            } else {
                v.clone()
            }
        })
        .collect();
    validate_items(&json!(normalized))?;
    fn walk(options: &[Value], nested: bool) -> Result<(), String> {
        for option in options {
            let object = option.as_object().ok_or("option 必须为字符串或对象")?;
            if !object
                .get("value")
                .or_else(|| object.get("key"))
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.is_empty()))
            {
                return Err("option 缺少非空 value/key".into());
            }
            if let Some(label) = object.get("label").or_else(|| object.get("title"))
                && !label.is_string()
            {
                return Err("option.label/title 必须为字符串".into());
            }
            if let Some(children) = object.get("children") {
                if !nested {
                    return Err("此控件不支持嵌套 options".into());
                }
                walk(
                    children.as_array().ok_or("option.children 必须为数组")?,
                    nested,
                )?;
            }
        }
        Ok(())
    }
    walk(&normalized, nested)
}
pub(crate) fn validate_secondary(
    n: &Node,
    state: &State,
    event: &str,
    payload: &Value,
) -> Result<(), String> {
    if let Some(name) = event_property(n.kind, event) {
        let ty = properties(n.kind)
            .into_iter()
            .find(|(key, _)| *key == name)
            .ok_or("事件属性未定义")?
            .1;
        if !property_valid(ty, payload)? {
            return Err(format!("{event} 的 {name} 载荷类型或取值非法"));
        }
        if matches!(name, "openKeys" | "checkedKeys" | "expandedKeys") {
            validate_keys(payload)?;
            validate_item_keys(n, state, payload)?;
        }
    }
    Ok(())
}
fn validate_item_keys(n: &Node, state: &State, value: &Value) -> Result<(), String> {
    fn contains(items: &[Value], key: &str) -> bool {
        items.iter().any(|item| {
            item.get("key")
                .or_else(|| item.get("value"))
                .and_then(Value::as_str)
                == Some(key)
                || item
                    .get("children")
                    .and_then(Value::as_array)
                    .is_some_and(|children| contains(children, key))
        })
    }
    if value.as_array().is_some_and(|keys| {
        keys.iter()
            .any(|key| !contains(items(n, state), key.as_str().unwrap_or("")))
    }) {
        return Err("状态中的 key 不属于组件 items/treeData".into());
    }
    Ok(())
}
pub(crate) fn validate_items(value: &Value) -> Result<(), String> {
    fn walk(
        items: &[Value],
        keys: &mut std::collections::BTreeSet<String>,
        depth: usize,
        count: &mut usize,
    ) -> Result<(), String> {
        if depth > 32 {
            return Err("items 嵌套超过 32 层".into());
        }
        for item in items {
            *count += 1;
            if *count > 10000 {
                return Err("items 最多 10000 项".into());
            }
            let obj = item.as_object().ok_or("items 每项必须为对象")?;
            if let Some(key) = obj.get("key").or_else(|| obj.get("value")) {
                let key = key.as_str().ok_or("item.key/value 必须为字符串")?;
                if key.is_empty() || !keys.insert(key.to_owned()) {
                    return Err("item.key/value 为空或重复".into());
                }
            }
            if let Some(disabled) = obj.get("disabled")
                && !disabled.is_boolean()
            {
                return Err("item.disabled 必须为布尔值".into());
            }
            if let Some(children) = obj.get("children") {
                walk(
                    children.as_array().ok_or("item.children 必须为数组")?,
                    keys,
                    depth + 1,
                    count,
                )?;
            }
        }
        Ok(())
    }
    walk(
        value.as_array().ok_or("items 必须为数组")?,
        &mut Default::default(),
        0,
        &mut 0,
    )
}
pub(crate) fn property<'a>(n: &'a Node, state: &'a State, name: &str) -> Option<&'a Value> {
    n.api.get(name).and_then(|b| b.resolve(state).ok())
}
pub(crate) fn flag(n: &Node, state: &State, name: &str, default: bool) -> bool {
    property(n, state, name)
        .and_then(Value::as_bool)
        .unwrap_or(default)
}
pub(crate) fn number(n: &Node, state: &State, name: &str, default: f64) -> f64 {
    property(n, state, name)
        .and_then(Value::as_f64)
        .unwrap_or(default)
}
pub(crate) fn string<'a>(n: &'a Node, state: &'a State, name: &str, default: &'a str) -> &'a str {
    property(n, state, name)
        .and_then(Value::as_str)
        .unwrap_or(default)
}
pub(crate) fn tr<'a>(n: &Node, state: &State, zh: &'a str, en: &'a str) -> &'a str {
    if string(n, state, "@locale", string(n, state, "locale", "zh-CN")) == "en-US" {
        en
    } else {
        zh
    }
}
pub(crate) fn items<'a>(n: &'a Node, state: &'a State) -> &'a [Value] {
    property(
        n,
        state,
        if n.kind == Kind::Tree {
            "treeData"
        } else if n.kind == Kind::Tour {
            "steps"
        } else {
            "items"
        },
    )
    .and_then(Value::as_array)
    .map(Vec::as_slice)
    .unwrap_or(&[])
}
pub(crate) fn validate_value(n: &Node, state: &State, value: &Value) -> Result<(), String> {
    let array = || validate_keys(value).is_ok();
    let valid = match n.kind {
        Kind::Pagination => value.as_u64().is_some_and(|n| n > 0 && n <= 1000000000),
        Kind::Steps | Kind::Carousel | Kind::Tour => value.as_u64().is_some_and(|n| n <= 10000),
        Kind::Menu | Kind::Tree | Kind::Transfer | Kind::Collapse => array(),
        Kind::Upload => value
            .as_array()
            .is_some_and(|a| a.len() <= 10000 && a.iter().all(Value::is_object)),
        Kind::Cascader if flag(n, state, "multiple", false) => {
            value.as_array().is_some_and(|paths| {
                paths.len() <= 10000
                    && paths.iter().all(|p| {
                        validate_keys(p).is_ok() && p.as_array().is_some_and(|p| !p.is_empty())
                    })
            })
        }
        Kind::Cascader => array(),
        Kind::TreeSelect
            if flag(n, state, "multiple", false) || flag(n, state, "treeCheckable", false) =>
        {
            array()
        }
        Kind::Rate => value.as_f64().is_some_and(|x| {
            x >= 0.
                && x <= number(n, state, "count", 5.)
                && (flag(n, state, "allowHalf", false) && (x * 2.).fract() == 0. || x.fract() == 0.)
        }),
        Kind::DatePicker | Kind::Calendar => value.as_str().is_some_and(|s| {
            s.is_empty()
                || chrono::NaiveDate::parse_from_str(s.get(..10).unwrap_or(s), "%Y-%m-%d").is_ok()
                    && (s.len() == 10
                        || n.kind == Kind::DatePicker
                            && flag(n, state, "showTime", false)
                            && chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                                .is_ok())
                    && property(n, state, "minDate")
                        .and_then(Value::as_str)
                        .is_none_or(|min| s.get(..10).unwrap_or(s) >= min)
                    && property(n, state, "maxDate")
                        .and_then(Value::as_str)
                        .is_none_or(|max| s.get(..10).unwrap_or(s) <= max)
        }),
        Kind::TimePicker => value.as_str().is_some_and(|s| {
            s.is_empty() || chrono::NaiveTime::parse_from_str(s, "%H:%M:%S").is_ok()
        }),
        Kind::ColorPicker => value
            .as_str()
            .is_some_and(|s| s.is_empty() || serde_json::from_value::<Token>(json!(s)).is_ok()),
        _ => value.is_string(),
    };
    if !valid {
        return Err(format!("{:?}.value 类型或取值非法", n.kind));
    }
    if matches!(
        n.kind,
        Kind::Menu | Kind::Tree | Kind::Collapse | Kind::Tabs
    ) {
        let mut keys = std::collections::BTreeSet::new();
        fn collect<'a>(items: &'a [Value], keys: &mut std::collections::BTreeSet<&'a str>) {
            for item in items {
                if let Some(s) = item
                    .get("key")
                    .or_else(|| item.get("value"))
                    .and_then(Value::as_str)
                {
                    keys.insert(s);
                }
                if let Some(a) = item["children"].as_array() {
                    collect(a, keys);
                }
            }
        }
        collect(items(n, state), &mut keys);
        let selected = if let Some(s) = value.as_str() {
            vec![s]
        } else {
            value
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .collect()
        };
        if selected.iter().any(|s| !s.is_empty() && !keys.contains(s)) {
            return Err("选中的 key 不属于组件 items/treeData".into());
        }
        if matches!(n.kind, Kind::Menu | Kind::Tree)
            && !flag(n, state, "multiple", false)
            && selected.len() > 1
        {
            return Err("单选组件最多一个 selectedKey".into());
        }
        if n.kind == Kind::Collapse && flag(n, state, "accordion", false) && selected.len() > 1 {
            return Err("accordion 最多展开一项".into());
        }
    }
    Ok(())
}
pub(crate) fn multiple(n: &Node, state: &State) -> bool {
    matches!(string(n, state, "mode", "single"), "multiple" | "tags")
}
/// Explicit secondary bindings prevent event payloads from selecting arbitrary state.
pub(crate) fn event_property(kind: Kind, event: &str) -> Option<&'static str> {
    super::catalogue::definition(kind)
        .secondary
        .iter()
        .find(|(name, _)| *name == event)
        .map(|(_, property)| *property)
}
