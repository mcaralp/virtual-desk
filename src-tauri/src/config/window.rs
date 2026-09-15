use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Int(i64),
    Float(f64),
}

// Accepts either a YAML string or a number (e.g. `100%` or `0`) and normalizes it to a String.
fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s,
        StringOrNumber::Int(i) => i.to_string(),
        StringOrNumber::Float(f) => f.to_string(),
    })
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WindowConfig {
    pub screen: u32,

    #[serde(deserialize_with = "string_or_number")]
    pub width: String,

    #[serde(deserialize_with = "string_or_number")]
    pub height: String,

    pub position: PositionConfig,
    pub origin: PositionConfig,

    pub transparent: bool,
    pub decorations: bool,
    pub pinned: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PositionConfig {
    #[serde(deserialize_with = "string_or_number")]
    pub x: String,

    #[serde(deserialize_with = "string_or_number")]
    pub y: String,
}

