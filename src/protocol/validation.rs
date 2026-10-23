//! Shared value limits for bindings, state and action results.
pub const MAX_STRING_BYTES: usize = 65_536;

pub fn string_size(value: &str, path: &str) -> Result<(), String> {
    if value.len() > MAX_STRING_BYTES {
        Err(format!("{path}: 字符串超过 {MAX_STRING_BYTES} UTF-8 字节"))
    } else {
        Ok(())
    }
}
