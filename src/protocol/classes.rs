//! Authored class shorthand is lowered before defaults, measurement and validation.
use super::{Kind, Token};
use hypen_tailwind_parse::{CssProperty, Variant, parse_class};
use serde_json::{Map, Value, json};

pub(super) fn normalize(node: &mut Value, kind: Kind) -> Result<(), String> {
    let Some(classes) = node.as_object_mut().and_then(|o| o.remove("class")) else {
        return Ok(());
    };
    let classes = classes.as_str().ok_or("class 必须是静态字符串")?;
    if classes.len() > 16_384 || classes.split_whitespace().count() > 256 {
        return Err("class 最多 16384 UTF-8 字节、256 个类名".into());
    }
    let mut style = Map::new();
    let mut layout = Map::new();
    for class in classes.split_whitespace() {
        apply(class, kind, &mut style, &mut layout).map_err(|e| format!("class {class:?}: {e}"))?;
    }
    for (field, mut generated) in [("style", style), ("layout", layout)] {
        let object = node.as_object_mut().ok_or("组件必须是对象")?;
        if let Some(explicit) = object.remove(field) {
            let explicit = explicit
                .as_object()
                .ok_or_else(|| format!("{field} 必须是对象"))?;
            if field == "layout" && explicit.contains_key("gap") {
                generated.remove("rowGap");
                generated.remove("columnGap");
            }
            for (key, value) in explicit {
                // State colors merge individually; tracks such as width replace atomically.
                if field == "style"
                    && matches!(key.as_str(), "hover" | "pressed" | "disabled")
                    && let (Some(Value::Object(base)), Value::Object(overrides)) =
                        (generated.get_mut(key), value)
                {
                    base.extend(overrides.clone());
                    continue;
                }
                generated.insert(key.clone(), value.clone());
            }
        }
        if !generated.is_empty() {
            object.insert(field.into(), Value::Object(generated));
        }
    }
    Ok(())
}

fn apply(
    class: &str,
    kind: Kind,
    style: &mut Map<String, Value>,
    layout: &mut Map<String, Value>,
) -> Result<(), String> {
    if class.contains('!') {
        return Err("暂不支持 important 修饰符".into());
    }
    // Work around unreachable named text-wrap and gap-x/y branches in parser 0.6.6.
    let parsed = match class {
        "text-wrap" | "text-nowrap" => Some((
            Variant::None,
            vec![CssProperty::new(
                "text-wrap",
                if class == "text-wrap" {
                    "wrap"
                } else {
                    "nowrap"
                },
            )],
        )),
        _ if class.starts_with("gap-x-") || class.starts_with("gap-y-") => {
            let axis = if class.starts_with("gap-x-") {
                "column-gap"
            } else {
                "row-gap"
            };
            parse_class(&format!("gap-{}", &class[6..]))
                .map_err(|e| e.to_string())?
                .map(|(variant, mut props)| {
                    for prop in &mut props {
                        prop.property = axis.into();
                    }
                    (variant, props)
                })
        }
        _ => parse_class(class).map_err(|e| e.to_string())?,
    };
    let (variant, props) = parsed.ok_or("未知或暂不支持的类名")?;
    let state = match variant {
        Variant::None => None,
        Variant::State(ref name) => Some(match name.as_str() {
            "hover" => "hover",
            "active" => "pressed",
            "disabled" => "disabled",
            _ => return Err("仅支持 hover: / active: / disabled: 状态".into()),
        }),
        _ => return Err("暂不支持响应式、dark 或组合变体".into()),
    };
    if state.is_some()
        && !matches!(
            kind,
            Kind::Button | Kind::Input | Kind::Select | Kind::InputNumber
        )
    {
        return Err("状态颜色仅用于 Button/Input/Select/InputNumber".into());
    }
    let sized_text = props.iter().any(|p| p.property == "font-size");
    for prop in props {
        let p = prop.property.as_str();
        let v = prop.value.as_str();
        if let Some(state) = state {
            let key = color_key(p).ok_or("状态变体仅支持背景、文字和边框颜色")?;
            let colors = style
                .entry(state)
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .unwrap();
            colors.insert(key.into(), color(v)?);
            continue;
        }
        if let Some(key) = color_key(p) {
            style.insert(key.into(), color(v)?);
            continue;
        }
        match p {
            "font-size" => {
                style.insert("fontSize".into(), json!(length(v)?));
            }
            // Named text sizes provide only font size in this native subset; egui owns line height.
            "line-height" if sized_text => {}
            "font-weight" if matches!(v, "400" | "700") => {
                style.insert(
                    "fontWeight".into(),
                    json!(if v == "700" { "bold" } else { "normal" }),
                );
            }
            "font-family" if matches!(class, "font-sans" | "font-mono") => {
                style.insert(
                    "fontFamily".into(),
                    json!(if class == "font-mono" {
                        "monospace"
                    } else {
                        "proportional"
                    }),
                );
            }
            "text-align" if matches!(v, "left" | "start" | "center" | "right" | "end") => {
                style.insert(
                    "textAlign".into(),
                    json!(match v {
                        "left" => "start",
                        "right" => "end",
                        _ => v,
                    }),
                );
            }
            "text-wrap" | "white-space" if matches!(v, "wrap" | "normal" | "nowrap") => {
                style.insert("textWrap".into(), json!(v != "nowrap"));
            }
            "border-radius" => {
                style.insert("radius".into(), json!(length(v)?));
            }
            "border-width" => {
                style.insert("borderWidth".into(), json!(length(v)?));
                style.entry("borderColor").or_insert(json!("border"));
            }
            "padding" if kind.is_container() => {
                layout.insert("padding".into(), json!(length(v)?));
            }
            "padding" | "padding-left" | "padding-right" | "padding-top" | "padding-bottom" => {
                if !matches!(kind, Kind::Button | Kind::Badge | Kind::Input)
                    || !(class.starts_with("p-")
                        || class.starts_with("px-")
                        || class.starts_with("py-"))
                {
                    return Err("p-* 用于容器或 Button/Badge/Input；px-*/py-* 仅用于 Button/Badge/Input，暂不支持单边 padding".into());
                }
                let amount = length(v)?;
                if kind == Kind::Input && (amount > 127. || amount.fract() != 0.) {
                    return Err("Input 内边距须为 0..127 的整数 point".into());
                }
                if matches!(p, "padding" | "padding-left" | "padding-right") {
                    style.insert("paddingX".into(), json!(amount));
                }
                if matches!(p, "padding" | "padding-top" | "padding-bottom") {
                    style.insert("paddingY".into(), json!(amount));
                }
            }
            "width" | "height" => {
                layout.insert(
                    p.into(),
                    if v == "auto" {
                        json!({"auto":true})
                    } else {
                        json!({"fixed":length(v)?})
                    },
                );
            }
            "min-width" | "max-width" | "min-height" | "max-height" => {
                let key = match p {
                    "min-width" => "minWidth",
                    "max-width" => "maxWidth",
                    "min-height" => "minHeight",
                    _ => "maxHeight",
                };
                layout.insert(key.into(), json!(length(v)?));
            }
            "gap" | "row-gap" | "column-gap" => {
                let amount = json!(length(v)?);
                if p == "gap" {
                    // A later shorthand resets earlier axis-specific declarations.
                    layout.remove("rowGap");
                    layout.remove("columnGap");
                }
                layout.insert(
                    match p {
                        "row-gap" => "rowGap",
                        "column-gap" => "columnGap",
                        _ => "gap",
                    }
                    .into(),
                    amount,
                );
            }
            "align-items"
                if matches!(
                    v,
                    "start" | "end" | "flex-start" | "flex-end" | "center" | "stretch"
                ) =>
            {
                layout.insert(
                    "align".into(),
                    json!(match v {
                        "flex-start" => "start",
                        "flex-end" => "end",
                        _ => v,
                    }),
                );
            }
            "justify-content"
                if matches!(
                    v,
                    "start"
                        | "end"
                        | "flex-start"
                        | "flex-end"
                        | "center"
                        | "space-between"
                        | "space-around"
                        | "space-evenly"
                ) =>
            {
                layout.insert(
                    "justify".into(),
                    json!(match v {
                        "flex-start" => "start",
                        "flex-end" => "end",
                        "space-between" => "spaceBetween",
                        "space-around" => "spaceAround",
                        "space-evenly" => "spaceEvenly",
                        _ => v,
                    }),
                );
            }
            "flex-wrap" if matches!(v, "wrap" | "nowrap") => {
                layout.insert("wrap".into(), json!(v == "wrap"));
            }
            "overflow" if matches!(v, "hidden" | "clip" | "scroll" | "auto") => {
                layout.insert(
                    "overflow".into(),
                    json!(if matches!(v, "scroll" | "auto") {
                        "scroll"
                    } else {
                        "clip"
                    }),
                );
            }
            _ => return Err(format!("暂不支持属性 {p}={v}")),
        }
    }
    Ok(())
}

fn color_key(property: &str) -> Option<&'static str> {
    match property {
        "background-color" => Some("background"),
        "color" => Some("color"),
        "border-color" => Some("borderColor"),
        _ => None,
    }
}

fn length(value: &str) -> Result<f32, String> {
    let (number, scale) = if let Some(n) = value.strip_suffix("rem") {
        (n, 16.)
    } else if let Some(n) = value.strip_suffix("px") {
        (n, 1.)
    } else if value == "0" {
        (value, 1.)
    } else {
        return Err(format!("尺寸 {value:?} 仅支持 px/rem（1rem=16 point）"));
    };
    let result = number
        .parse::<f32>()
        .map_err(|_| format!("非法尺寸 {value:?}"))?
        * scale;
    if !result.is_finite() || !(0. ..=1_000_000.).contains(&result) {
        return Err(format!("尺寸 {value:?} 超出 0..1000000"));
    }
    Ok(result)
}

fn color(value: &str) -> Result<Value, String> {
    let normalized = if value == "transparent" {
        "#00000000".into()
    } else if let Some(hex) = value
        .strip_prefix('#')
        .filter(|s| matches!(s.len(), 3 | 4) && s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        format!("#{}", hex.chars().flat_map(|c| [c, c]).collect::<String>())
    } else if let Some(rgba) = value
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let values = rgba
            .split(',')
            .map(|s| s.trim().parse::<f32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| format!("非法颜色 {value:?}"))?;
        if values.len() != 4
            || values.iter().any(|v| !v.is_finite())
            || values[..3].iter().any(|v| !(0. ..=255.).contains(v))
            || !(0. ..=1.).contains(&values[3])
        {
            return Err(format!("非法颜色 {value:?}"));
        }
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            values[0].round() as u8,
            values[1].round() as u8,
            values[2].round() as u8,
            (values[3] * 255.).round() as u8
        )
    } else {
        value.into()
    };
    serde_json::from_value::<Token>(json!(normalized))
        .map_err(|_| format!("不支持颜色 {value:?}，请使用颜色表或十六进制颜色"))?;
    Ok(json!(normalized))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_classes_and_explicit_fields_merge_without_losing_zero_or_states() {
        let mut node = json!({"class":"p-4 px-2 bg-blue-500 rounded-lg border-2 hover:bg-blue-600 hover:text-white w-40",
            "style":{"radius":0,"borderWidth":0,"paddingY":0,"hover":{"color":"#112233"}},
            "layout":{"width":{"auto":true}}});
        normalize(&mut node, Kind::Button).unwrap();
        assert_eq!(
            node,
            json!({"style":{"paddingX":8.,"paddingY":0,"background":"#3b82f6","radius":0,"borderWidth":0,"borderColor":"border","hover":{"background":"#2563eb","color":"#112233"}},"layout":{"width":{"auto":true}}})
        );
    }

    #[test]
    fn shorthand_gap_resets_axes_and_explicit_gap_overrides_class_axes() {
        let mut node = json!({"class":"p-3 gap-x-8 gap-y-4 gap-2 items-center justify-between"});
        normalize(&mut node, Kind::Row).unwrap();
        assert_eq!(
            node["layout"],
            json!({"padding":12.,"gap":8.,"align":"center","justify":"spaceBetween"})
        );
        let mut node = json!({"class":"gap-2 gap-x-8 gap-y-4","layout":{"gap":0,"rowGap":3}});
        normalize(&mut node, Kind::Row).unwrap();
        assert_eq!(node["layout"], json!({"gap":0,"rowGap":3}));
    }

    #[test]
    fn alignment_classes_use_native_start_and_end_values() {
        for (class, expected) in [
            (
                "items-start justify-end",
                json!({"align":"start","justify":"end"}),
            ),
            (
                "items-end justify-start",
                json!({"align":"end","justify":"start"}),
            ),
            (
                "items-stretch justify-evenly flex-wrap overflow-auto",
                json!({"align":"stretch","justify":"spaceEvenly","wrap":true,"overflow":"scroll"}),
            ),
        ] {
            let mut node = json!({"class":class});
            normalize(&mut node, Kind::Row).unwrap();
            assert_eq!(node["layout"], expected);
        }
    }

    #[test]
    fn native_units_font_and_alpha_are_lowered() {
        let mut node = json!({"class":"text-xl font-bold text-right whitespace-nowrap bg-black/50 border-[#abc] rounded-none w-[12.5rem] min-h-[40px]"});
        normalize(&mut node, Kind::Text).unwrap();
        assert_eq!(
            node["style"],
            json!({"fontSize":20.,"fontWeight":"bold","textAlign":"end","textWrap":false,"background":"#00000080","borderColor":"#aabbcc","radius":0.})
        );
        assert_eq!(
            node["layout"],
            json!({"width":{"fixed":200.},"minHeight":40.})
        );
        let mut node = json!({"class":"text-wrap bg-transparent text-[#abcd]"});
        normalize(&mut node, Kind::Text).unwrap();
        assert_eq!(node["style"]["color"], "#aabbccdd");
        assert_eq!(node["style"]["textWrap"], true);
    }

    #[test]
    fn unsupported_classes_fail_even_when_explicit_fields_would_override_them() {
        for class in [
            "typo",
            "md:p-4",
            "dark:bg-black",
            "hover:active:bg-black",
            "focus:bg-black",
            "!p-4",
            "shadow-lg",
            "w-full",
            "w-1/2",
            "w-[NaNpx]",
            "p-[-1px]",
            "bg-[var(--color)]",
            "font-semibold",
            "leading-6",
            "rounded-t-lg",
            "pt-2",
            "hover:p-2",
        ] {
            let mut node = json!({"class":class,"style":{"background":"#ffffff"},"layout":{"width":{"fixed":10}}});
            assert!(
                normalize(&mut node, Kind::Button).is_err(),
                "accepted {class}"
            );
        }
        for (kind, class) in [
            (Kind::Column, "px-4"),
            (Kind::Text, "hover:bg-black"),
            (Kind::Input, "py-[0.5px]"),
            (Kind::Input, "py-96"),
        ] {
            assert!(normalize(&mut json!({"class":class}), kind).is_err());
        }
        for value in [
            json!([]),
            json!({"kind":"state","key":"classes"}),
            json!(null),
            json!(42),
        ] {
            assert!(normalize(&mut json!({"class":value}), Kind::Text).is_err());
        }
    }

    #[test]
    fn child_classes_parse_recursively_but_business_data_stays_literal() {
        let description = super::super::parse(&json!({"schemaVersion":"0.2","app":{"name":"Class tests","window":{"width":640,"height":480}},"state":{"data":{"class":"not-a-class"}},"actions":{},"root":{"type":"column","id":"root","class":"p-4 gap-3","children":[{"type":"button","id":"button","class":"px-4 py-2 rounded-none border-0 active:bg-blue-700 disabled:text-gray-400","text":{"kind":"literal","value":"Run"}}]}}).to_string()).unwrap();
        assert_eq!(description.root.layout.padding, 16.);
        let child = &description.root.children[0].0;
        assert_eq!(child.style.padding_x, Some(16.));
        assert_eq!(child.style.radius, 0.);
        assert_eq!(child.style.border_width, 0.);
        assert!(child.style.pressed.is_some());
        assert_eq!(description.state["data"]["class"], "not-a-class");
    }
}
