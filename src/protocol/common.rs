use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_host")]
    pub host: Option<String>,
    pub window: Window,
}

fn deserialize_host<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let host = String::deserialize(deserializer)?;
    if host.is_empty()
        || host.len() > 64
        || !host
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    {
        return Err(serde::de::Error::custom(
            "host 必须是 1..64 位字母、数字、下划线或连字符",
        ));
    }
    Ok(Some(host))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub width: f32,
    pub height: f32,
}
