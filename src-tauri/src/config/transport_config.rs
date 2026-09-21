use super::{HostTcp, HostSshClient, HostSshServer, Mode};

#[derive(Debug, Clone, PartialEq)]
pub enum TransportConfig
{
    TcpClient(HostTcp),
    TcpServer(HostTcp),
    SshClient(HostSshClient),
    SshServer(HostSshServer),
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
            Mode::SshClient(client) => TransportConfig::SshClient(client.host),
            Mode::SshServer(server) => TransportConfig::SshServer(server.host),
        }
    }
}
