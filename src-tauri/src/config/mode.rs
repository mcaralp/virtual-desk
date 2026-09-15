use serde::{Deserialize, Serialize};
use crate::config::{WindowConfig, PageConfig, WidgetConfig};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "mode", content = "settings", rename_all = "lowercase")]
pub enum Mode
{
    Local(LocalConfig),
    TcpServer(TcpServerConfig),
    TcpClient(TcpClientConfig),
    None()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocalConfig
{
    pub window: WindowConfig,
    pub pages: Vec<PageConfig>,
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TcpServerConfig
{
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TcpClientConfig
{
    pub port: u16,
}
