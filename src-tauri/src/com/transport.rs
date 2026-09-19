use crate::error::Error;
use tokio_util::bytes::BytesMut;
use super::transport_tcp_client::TransportTcpClient;
use super::transport_tcp_server::TransportTcpServer;

pub enum Transport
{
    TcpServer(TransportTcpServer),
    TcpClient(TransportTcpClient),
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
        }
    }
}
