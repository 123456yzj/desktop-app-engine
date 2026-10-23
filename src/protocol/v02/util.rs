//! Desktop inspection utilities, shared with authoring tools.
use super::{
    Kind,
    widgets::{self, PropertyType},
};
use serde_json::{Value, json};

/// Inspect extension properties, their exact constraints, and native event names.
/// Common layout/style fields and core component fields are described by the protocol Schema.
pub fn component_api(kind: Kind) -> Value {
    let properties: serde_json::Map<String, Value> = widgets::properties(kind)
        .into_iter()
        .map(|(name, ty)| {
            let schema = match ty {
                PropertyType::Bool => json!({"type":"boolean"}),
                PropertyType::Text => json!({"type":"string","maxLength":crate::MAX_INPUT_CHARS}),
                PropertyType::Number(min, max) => {
                    json!({"type":"number","minimum":min,"maximum":max})
                }
                PropertyType::Integer(min, max) => {
                    json!({"type":"integer","minimum":min,"maximum":max})
                }
                PropertyType::Enum(values) => json!({"enum":values}),
                PropertyType::Array | PropertyType::Items => {
                    json!({"type":"array","maxItems":10000})
                }
            };
            (name.into(), schema)
        })
        .collect();
    let writes: serde_json::Map<String, Value> = widgets::events(kind)
        .iter()
        .filter_map(|event| {
            widgets::event_property(kind, event).map(|property| ((*event).into(), json!(property)))
        })
        .collect();
    json!({"properties":properties,"events":widgets::events(kind),"eventProperties":writes,"primaryProperty":widgets::primary(kind),"container":kind.is_container(),"input":kind.is_input()})
}
