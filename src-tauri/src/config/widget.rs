use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WidgetConfig {
    pub id: String,

    #[serde(rename = "type")]
    pub widget_type: String,

    pub params: Value,
}
