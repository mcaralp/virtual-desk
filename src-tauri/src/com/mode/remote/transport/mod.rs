mod tcp_server;
mod tcp_client;

use crate::com::Error;
use tokio_util::bytes::BytesMut;

pub use tcp_server::TcpServer;
pub use tcp_client::TcpClient;

pub enum TransportType
{
    TcpServer(TcpServer),
    TcpClient(TcpClient),
}

impl TransportType
{
    pub async fn connect(&mut self) -> Result<(), Error>
    {
        match self {
            TransportType::TcpServer(server) =>
            {
                server.connect().await
            }
            TransportType::TcpClient(client) =>
            {
                client.connect().await
            }
        }
    }

    pub async fn read(&mut self, buffer: &mut BytesMut) -> Result<usize, Error>
    {
        match self {
            TransportType::TcpServer(server) =>
            {
                server.read(buffer).await
            }
            TransportType::TcpClient(client) =>
            {
                client.read(buffer).await
            }
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), Error>
    {
        match self {
            TransportType::TcpServer(server) =>
            {
                server.write(data).await
            }
            TransportType::TcpClient(client) =>
            {
                client.write(data).await
            }
        }
    }
}
