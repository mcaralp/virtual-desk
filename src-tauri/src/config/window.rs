use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrNumber
{
    String(String),
    Int(i64),
    Float(f64),
}

// Accepts either a YAML string or a number (e.g. `100%` or `0`) and normalizes it to a String.
fn string_or_number<'de, D>(deserializer: D)
    -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let val = StringOrNumber::deserialize(deserializer)?;
    let result = match val {
        StringOrNumber::String(s) => s,
        StringOrNumber::Int(i) => i.to_string(),
        StringOrNumber::Float(f) => f.to_string(),
    };
    Ok(result)
}

fn default_decorations() -> bool
{
    true
}

fn default_position_value() -> String
{
    "0".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PositionConfig
{
    #[serde(deserialize_with = "string_or_number", default = "default_position_value")]
    pub x: String,

    #[serde(deserialize_with = "string_or_number", default = "default_position_value")]
    pub y: String,
}

impl Default for PositionConfig
{
    fn default() -> Self
    {
        Self {
            x: default_position_value(),
            y: default_position_value(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WindowConfig
{
    #[serde(default)]
    pub screen: u32,

    #[serde(deserialize_with = "string_or_number")]
    pub width: String,

    #[serde(deserialize_with = "string_or_number")]
    pub height: String,

    #[serde(default)]
    pub position: PositionConfig,

    #[serde(default)]
    pub origin: PositionConfig,

    #[serde(default)]
    pub rotation: i32,

    #[serde(default)]
    pub transparent: bool,

    #[serde(default = "default_decorations")]
    pub decorations: bool,

    #[serde(default)]
    pub pinned: bool,
}
