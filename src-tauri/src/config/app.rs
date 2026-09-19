use serde::{Deserialize, Serialize};
use super::Mode;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig
{
    pub version: u32,

    #[serde(flatten)]
    pub mode: Mode
}
