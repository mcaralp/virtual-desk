use serde::{Deserialize, Serialize};
use super::{WindowConfig, PageConfig, WidgetConfig};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "mode", content = "settings", rename_all = "lowercase")]
pub enum Mode
{
    Local(LocalConfig),
    TcpServer(TcpServerConfig),
    TcpClient(TcpClientConfig),
    SshClient(SshClientConfig),
    SshServer(SshServerConfig)
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

impl From<SshClientConfig> for Mode {
    fn from(config: SshClientConfig) -> Self {
        Mode::SshClient(config)
    }
}

impl From<SshServerConfig> for Mode {
    fn from(config: SshServerConfig) -> Self {
        Mode::SshServer(config)
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
pub struct HostTcp
{
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for HostTcp
{
    fn default() -> Self
    {
        HostTcp { address: default_address(), port: default_port() }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostSshServer
{
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub private_key_path: Option<String>,
    #[serde(default)]
    pub authorized_client_keys: Option<String>,
}

impl Default for HostSshServer
{
    fn default() -> Self
    {
        HostSshServer { address: default_address(), port: default_port(), private_key_path: None, authorized_client_keys: None }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HostSshClient
{
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub private_key_path: Option<String>,
    #[serde(default)]
    pub server_public_key_path: Option<String>,
}

impl Default for HostSshClient
{
    fn default() -> Self
    {
        HostSshClient { address: default_address(), port: default_port(), private_key_path: None, server_public_key_path: None }
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
    pub host: HostTcp,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TcpClientConfig
{
    #[serde(default)]
    pub host: HostTcp,
    pub window: WindowConfig,
    pub pages: Vec<PageConfig>,
    pub widgets: Vec<WidgetConfig>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SshClientConfig
{
    #[serde(default)]
    pub host: HostSshClient,
    pub window: WindowConfig,
    pub pages: Vec<PageConfig>,
    pub widgets: Vec<WidgetConfig>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SshServerConfig
{
    #[serde(default)]
    pub host: HostSshServer,
}
