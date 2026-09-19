use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResponseData
{
    pub last: bool,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResponse
{
    pub uuid: String,
    pub result: Result<CommandResponseData, String>,
}
