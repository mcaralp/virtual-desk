use super::{Host, Mode};

#[derive(Debug, Clone, PartialEq)]
pub enum TransportConfig
{
    TcpClient(Host),
    TcpServer(Host),
    Local
}

impl From<Mode> for TransportConfig
{
    fn from(config: Mode) -> Self
    {
        match config
        {
            Mode::Local(_) => TransportConfig::Local,
            Mode::TcpServer(server) => TransportConfig::TcpServer(server.host),
            Mode::TcpClient(client) => TransportConfig::TcpClient(client.host),
        }
    }
}
