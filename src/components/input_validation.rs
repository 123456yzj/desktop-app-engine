//! Component input values and form field validation.
use crate::binding::{Binding, State};
use crate::v02::*;
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) fn validate_numeric(n: &Node) -> Result<(), String> {
    let (min, max) = n.number_range();
    let step = n.step.unwrap_or(1.0);
    if !min.is_finite()
        || !max.is_finite()
        || min.abs() > 1e12
        || max.abs() > 1e12
        || min >= max
        || !step.is_finite()
        || !(1e-12..=1e12).contains(&step)
    {
        return Err("数字控件要求 -1e12 <= min < max <= 1e12，step 在 1e-12..1e12".into());
    }
    if n.precision.is_some_and(|p| p > 12) {
        return Err("inputNumber.precision 必须在 0..12".into());
    }
    Ok(())
}

pub(crate) fn validate_input_value(n: &Node, state: &State, value: &Value) -> Result<(), String> {
    match n.kind {
        Kind::Checkbox | Kind::Switch => {
            if n.kind == Kind::Checkbox && n.api.contains_key("options") {
                widgets::validate_keys(value)?;
                let options = widgets::property(n, state, "options")
                    .and_then(Value::as_array)
                    .ok_or("checkbox.options 必须为数组")?;
                if !value
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|key| options.iter().any(|o| o == key || &o["value"] == key))
                {
                    return Err("checkbox.value 必须属于 options".into());
                }
                return Ok(());
            }
            if !value.is_boolean() {
                return Err("value 必须为布尔值".into());
            }
        }
        Kind::InputNumber | Kind::Slider => {
            let (min, max) = n.number_range();
            if n.kind == Kind::Slider && widgets::flag(n, state, "range", false) {
                if !value.as_array().is_some_and(|a| {
                    a.len() == 2
                        && a.iter().all(|v| {
                            v.as_f64()
                                .is_some_and(|v| v.is_finite() && (min..=max).contains(&v))
                        })
                        && a[0].as_f64() <= a[1].as_f64()
                }) {
                    return Err("范围 slider.value 必须为 min..max 内按升序排列的两个数字".into());
                }
                return Ok(());
            }
            if !value
                .as_f64()
                .is_some_and(|v| v.is_finite() && (min..=max).contains(&v))
            {
                return Err("value 必须为 min..max 内的有限数字".into());
            }
        }
        Kind::Radio => {
            let selected = value.as_str().ok_or("radio.value 必须为字符串")?;
            let options = select_options(n, state)?;
            if options.len() > 256 {
                return Err("radio.options 最多 256 项".into());
            }
            // Empty selection is useful for a required form field.
            if !selected.is_empty() && !options.iter().any(|o| o.value == selected) {
                return Err("radio.value 必须为空或属于 options".into());
            }
        }
        Kind::Input => {
            let text = value.as_str().ok_or("input.value 必须为字符串")?;
            if text.chars().count()
                > widgets::number(n, state, "maxLength", crate::MAX_INPUT_CHARS as f64) as usize
            {
                return Err("input.value 超过 maxLength".into());
            }
        }
        Kind::Select => {
            let options = select_options(n, state)?;
            let selected = if widgets::multiple(n, state) {
                widgets::validate_keys(value)?;
                let values = value.as_array().unwrap();
                if values.len() > widgets::number(n, state, "maxCount", 10000.) as usize {
                    return Err("select.value 超过 maxCount".into());
                }
                values.iter().filter_map(Value::as_str).collect::<Vec<_>>()
            } else {
                vec![value.as_str().ok_or("select.value 必须为字符串")?]
            };
            if widgets::string(n, state, "mode", "single") != "tags"
                && selected
                    .iter()
                    .any(|s| !s.is_empty() && !options.iter().any(|o| o.value == *s))
            {
                return Err("select.value 必须为空或属于 options".into());
            }
        }
        _ => {}
    }
    Ok(())
}

/// User validation is separate from protocol validation: an empty required field is a valid document.
pub(crate) fn field_error(item: &Node, value: &Value) -> Option<String> {
    for rule in &item.rules {
        let error = if rule.required
            && (value.as_str().is_some_and(|s| s.trim().is_empty())
                || value == &Value::Bool(false)
                || value.as_array().is_some_and(Vec::is_empty))
        {
            Some("此项必填".into())
        } else if let Some(s) = value.as_str() {
            let len = s.chars().count();
            if rule.min_length.is_some_and(|min| len < min) {
                Some(format!("至少 {} 个字符", rule.min_length.unwrap()))
            } else if rule.max_length.is_some_and(|max| len > max) {
                Some(format!("最多 {} 个字符", rule.max_length.unwrap()))
            } else if rule.pattern.as_ref().is_some_and(|pattern| {
                regex::Regex::new(pattern).is_ok_and(|regex| !regex.is_match(s))
            }) {
                Some("格式不符合要求".into())
            } else {
                None
            }
        } else if let Some(v) = value.as_f64() {
            if rule.min.is_some_and(|min| v < min) {
                Some(format!("不能小于 {}", rule.min.unwrap()))
            } else if rule.max.is_some_and(|max| v > max) {
                Some(format!("不能大于 {}", rule.max.unwrap()))
            } else {
                None
            }
        } else {
            None
        };
        if let Some(error) = error {
            return Some(rule.message.clone().unwrap_or(error));
        }
    }
    None
}

pub(crate) fn validate_forms(root: &Node) -> Result<(), String> {
    fn walk(n: &Node, fields: &mut Option<BTreeSet<String>>) -> Result<(), String> {
        if n.kind == Kind::Form {
            if fields.is_some() {
                return Err(format!("组件 {:?}: form 不可嵌套", n.id));
            }
            let mut own = Some(BTreeSet::new());
            for child in &n.children {
                walk(&child.0, &mut own)?;
            }
            return Ok(());
        }
        if n.kind == Kind::Button && n.html_type != ButtonType::Button && fields.is_none() {
            return Err(format!(
                "组件 {:?}: submit/reset 按钮必须位于 form 内",
                n.id
            ));
        }
        if n.kind == Kind::FormItem {
            let fields = fields
                .as_mut()
                .ok_or_else(|| format!("组件 {:?}: formItem 必须位于 form 内", n.id))?;
            if n.children.len() != 1 || !n.children[0].0.kind.is_input() {
                return Err(format!(
                    "组件 {:?}: formItem.children 必须恰好包含一个输入控件",
                    n.id
                ));
            }
            let control = &n.children[0].0;
            let Some(Binding::State { key, path }) = &control.value else {
                return Err(format!(
                    "组件 {:?}: formItem 的输入控件必须绑定可写 state",
                    n.id
                ));
            };
            if !path.is_empty() || !fields.insert(key.clone()) {
                return Err(format!(
                    "组件 {:?}: 同一 form 的字段必须绑定不同的顶层状态",
                    n.id
                ));
            }
            if n.rules.len() > 32 {
                return Err("formItem.rules 最多 32 项".into());
            }
            for rule in &n.rules {
                if (!rule.required
                    && rule.min_length.is_none()
                    && rule.max_length.is_none()
                    && rule.min.is_none()
                    && rule.max.is_none())
                    && rule.pattern.is_none()
                    || rule.message.as_ref().is_some_and(|s| s.trim().is_empty())
                    || rule.min_length.unwrap_or(0)
                        > rule.max_length.unwrap_or(crate::MAX_INPUT_CHARS)
                    || rule.max_length.is_some_and(|v| v > crate::MAX_INPUT_CHARS)
                    || [rule.min, rule.max]
                        .into_iter()
                        .flatten()
                        .any(|v| !v.is_finite() || v.abs() > 1e12)
                    || rule.min.unwrap_or(-1e12) > rule.max.unwrap_or(1e12)
                {
                    return Err(format!(
                        "组件 {:?}: formItem.rules 规则为空或范围非法",
                        n.id
                    ));
                }
                if let Some(pattern) = &rule.pattern {
                    if pattern.len() > 4096 {
                        return Err("pattern 最多 4096 字节".into());
                    }
                    regex::Regex::new(pattern)
                        .map_err(|e| format!("formItem.rules.pattern: {e}"))?;
                    if !matches!(
                        control.kind,
                        Kind::Input
                            | Kind::Select
                            | Kind::Radio
                            | Kind::AutoComplete
                            | Kind::Mentions
                            | Kind::TreeSelect
                            | Kind::DatePicker
                            | Kind::TimePicker
                            | Kind::ColorPicker
                    ) {
                        return Err("pattern 仅用于字符串字段".into());
                    }
                }
                if (rule.min_length.is_some() || rule.max_length.is_some())
                    && !matches!(control.kind, Kind::Input | Kind::Select | Kind::Radio)
                {
                    return Err("minLength/maxLength 仅用于字符串字段".into());
                }
                if (rule.min.is_some() || rule.max.is_some())
                    && !matches!(control.kind, Kind::InputNumber | Kind::Slider)
                {
                    return Err("min/max 规则仅用于数字字段".into());
                }
            }
        }
        for child in &n.children {
            walk(&child.0, fields)?;
        }
        Ok(())
    }
    walk(root, &mut None)
}
