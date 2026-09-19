use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CommandRequest
{
    #[serde(default)]
    pub cmd: u32,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub params: Value,
}
