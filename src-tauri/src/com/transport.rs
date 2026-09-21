use crate::error::Error;
use tokio_util::bytes::BytesMut;
use super::transport_tcp_client::TransportTcpClient;
use super::transport_tcp_server::TransportTcpServer;
use super::transport_ssh_client::TransportSshClient;
use super::transport_ssh_server::TransportSshServer;

pub enum Transport
{
    TcpServer(TransportTcpServer),
    TcpClient(TransportTcpClient),
    SshClient(TransportSshClient),
    SshServer(TransportSshServer),
}

impl Transport
{
    pub async fn connect(&mut self) -> Result<(), Error>
    {
        match self {
            Transport::TcpServer(server) =>
            {
                server.connect().await
            }
            Transport::TcpClient(client) =>
            {
                client.connect().await
            }
            Transport::SshClient(client) =>
            {
                client.connect().await
            }
            Transport::SshServer(server) =>
            {
                server.connect().await
            }
        }
    }

    pub async fn post_connect(&mut self) -> Result<(), Error>
    {
        match self {
            Transport::SshClient(client) =>
            {
                client.post_connect().await
            }
            Transport::SshServer(server) =>
            {
                server.post_connect().await
            },
            _ => Ok(()),
        }
    }

    pub async fn read(&mut self, buffer: &mut BytesMut) -> Result<usize, Error>
    {
        match self {
            Transport::TcpServer(server) =>
            {
                server.read(buffer).await
            }
            Transport::TcpClient(client) =>
            {
                client.read(buffer).await
            }
            Transport::SshClient(client) =>
            {
                client.read(buffer).await
            }
            Transport::SshServer(server) =>
            {
                server.read(buffer).await
            }
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error>
    {
        match self {
            Transport::TcpServer(server) =>
            {
                server.write(data).await
            }
            Transport::TcpClient(client) =>
            {
                client.write(data).await
            }
            Transport::SshClient(client) =>
            {
                client.write(data).await
            }
            Transport::SshServer(server) =>
            {
                server.write(data).await
            }
        }
    }

    pub async fn stop(&mut self) -> Result<(), Error>
    {
        match self {
            Transport::TcpServer(server) =>
            {
                server.stop().await?;
            }
            Transport::TcpClient(client) =>
            {
                client.stop().await?;
            }
            Transport::SshClient(client) =>
            {
                client.stop().await?;
            }
            Transport::SshServer(server) =>
            {
                server.stop().await?;
            }
        }
        Ok(())
    }
}
