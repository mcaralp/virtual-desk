use serde::{Deserialize, Serialize};
use super::{WindowConfig, PageConfig, WidgetConfig};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "mode", content = "settings", rename_all = "lowercase")]
pub enum Mode
{
    Local(LocalConfig),
    TcpServer(TcpServerConfig),
    TcpClient(TcpClientConfig)
}

impl From<TcpClientConfig> for Mode {
    fn from(config: TcpClientConfig) -> Self {
        Mode::TcpClient(config)
    }
}

impl From<TcpServerConfig> for Mode {
    fn from(config: TcpServerConfig) -> Self {
        Mode::TcpServer(config)
    }
}

impl From<LocalConfig> for Mode {
    fn from(config: LocalConfig) -> Self {
        Mode::Local(config)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocalConfig
{
    pub window: WindowConfig,
    pub pages: Vec<PageConfig>,
    pub widgets: Vec<WidgetConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Host
{
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for Host
{
    fn default() -> Self
    {
        Host { address: default_address(), port: default_port() }
    }
}

fn default_address() -> String
{
    "0.0.0.0".to_string()
}

fn default_port() -> u16
{
    8080
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TcpServerConfig
{
    #[serde(default)]
    pub host: Host,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TcpClientConfig
{
    #[serde(default)]
    pub host: Host,
    pub window: WindowConfig,
    pub pages: Vec<PageConfig>,
    pub widgets: Vec<WidgetConfig>
}
