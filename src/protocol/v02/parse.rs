use super::*;
use serde_json::Value;

/// Parse authored JSON only. Resource discovery and semantic validation belong to the compiler.
pub fn parse(source: &str) -> Result<Description, String> {
    let mut raw: Value = serde_json::from_str(source.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("JSON: {e}"))?;
    fn structural(v: &Value, path: &str) -> Result<(), String> {
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    if k == "state"
                        || k == "responseSample"
                        || k == "emits"
                        || k == "eventPayload" && !v.is_null()
                        || k == "value" && o.get("kind").and_then(Value::as_str) == Some("literal")
                    {
                        continue;
                    }
                    let p = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    if v.is_null() {
                        return Err(format!("{p}: 结构字段不得为 null，请省略可选字段"));
                    }
                    structural(v, &p)?;
                }
            }
            Value::Array(a) => {
                for (i, v) in a.iter().enumerate() {
                    structural(v, &format!("{path}[{i}]"))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    structural(&raw, "")?;
    let root = parse_node(raw.get("root").cloned().ok_or("root: 缺少字段")?)
        .map_err(|e| format!("root: {e}"))?;
    raw["root"] = serde_json::json!({"type":"column","id":"__parse_root"});
    let mut description: Description = serde_path_to_error::deserialize(raw)
        .map_err(|e| format!("描述字段 {}: {}", e.path(), e.inner()))?;
    description.root = root;
    Ok(description)
}

pub(super) fn parse_node(mut value: Value) -> Result<Node, String> {
    let kind: Kind = serde_json::from_value(value.get("type").cloned().ok_or("缺少 type")?)
        .map_err(|e| e.to_string())?;
    let specific: &[&str] = match kind {
        Kind::Plugin => &["component", "props"],
        Kind::Module => &["children", "state", "actions", "props", "emits", "on"],
        Kind::Row | Kind::Column => &["children", "state", "actions"],
        Kind::Flex | Kind::Space => &["children", "state", "actions", "vertical"],
        Kind::Layout | Kind::Header | Kind::Sider | Kind::Content | Kind::Footer => {
            &["children", "state", "actions"]
        }
        Kind::Splitter => &[
            "children",
            "state",
            "actions",
            "vertical",
            "value",
            "barSize",
            "resizable",
        ],
        Kind::Masonry => &[
            "children",
            "state",
            "actions",
            "columnCount",
            "minColumnWidth",
        ],
        Kind::Divider => &["text", "vertical", "variant", "titlePlacement"],
        Kind::Grid => &["children", "columns", "state", "actions"],
        Kind::Panel | Kind::Card => &["children", "title", "state", "actions"],
        Kind::Text => &["text"],
        Kind::Badge => &["text", "tone"],
        Kind::Input => &["value", "placeholder"],
        Kind::Select => &["value", "placeholder", "options", "optionsFormat"],
        Kind::Button => &["text"],
        Kind::Metric => &["label", "value", "format"],
        Kind::Progress => &["value", "text"],
        Kind::Table => &["rows", "rowKey", "rowHeight", "columns"],
        Kind::LineChart | Kind::BarChart => {
            &["series", "xAxis", "yAxis", "legend", "title", "zoom", "pan"]
        }
    };
    for key in value.as_object().ok_or("组件必须是对象")?.keys() {
        if ![
            "type", "id", "layout", "style", "visible", "enabled", "events",
        ]
        .contains(&key.as_str())
            && !specific.contains(&key.as_str())
        {
            return Err(format!("组件 {:?}: unknown field {key:?}", value.get("id")));
        }
    }
    if let Some(layout) = value.get("layout").and_then(Value::as_object) {
        let container = kind.is_container();
        for key in layout.keys() {
            if !container
                && [
                    "gap",
                    "rowGap",
                    "columnGap",
                    "align",
                    "justify",
                    "wrap",
                    "overflow",
                ]
                .contains(&key.as_str())
            {
                return Err(format!(
                    "组件 {:?}: layout.{key} 仅用于容器",
                    value.get("id")
                ));
            }
            if matches!(kind, Kind::Grid | Kind::Masonry) && key == "justify" {
                return Err("grid/masonry 不接受 layout.justify（轨道定义列宽）".into());
            }
            if kind == Kind::Splitter
                && ["gap", "rowGap", "columnGap", "wrap", "align", "justify"]
                    .contains(&key.as_str())
            {
                return Err("splitter 的间距由 barSize 控制，面板填满交叉轴；不接受 gap/rowGap/columnGap/wrap/align/justify".into());
            }
        }
    }
    // Apply semantic defaults once so measurement and rendering share the same values.
    if matches!(
        kind,
        Kind::Space | Kind::Layout | Kind::Header | Kind::Sider | Kind::Content | Kind::Footer
    ) {
        let layout = value
            .as_object_mut()
            .unwrap()
            .entry("layout")
            .or_insert_with(|| serde_json::json!({}));
        let layout = layout.as_object_mut().ok_or("layout 必须是对象")?;
        if kind == Kind::Space {
            layout.entry("gap").or_insert(serde_json::json!(8));
            layout.entry("align").or_insert(serde_json::json!("center"));
        } else {
            let width = if kind == Kind::Sider {
                serde_json::json!({"fixed":200})
            } else {
                serde_json::json!({"fr":1})
            };
            let height = match kind {
                Kind::Header => serde_json::json!({"fixed":64}),
                Kind::Footer => serde_json::json!({"fixed":48}),
                _ => serde_json::json!({"fr":1}),
            };
            layout.entry("width").or_insert(width);
            layout.entry("height").or_insert(height);
        }
    }
    serde_json::from_value(value).map_err(|e| e.to_string())
}
