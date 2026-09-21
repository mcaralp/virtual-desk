use tokio::net::{TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::bytes::BytesMut;
use crate::error::Error;

pub struct TransportTcpClient
{
    client: Option<TcpStream>,
    port: u16,
    address: String,
}

impl TransportTcpClient
{
    pub fn new(address: &str, port: u16) -> Self
    {
        Self { client: None, port, address: address.to_string() }
    }

    pub async fn connect(&mut self)
        -> Result<(), Error>
    {
        if self.client.is_some()
        {
            return Ok(());
        }
    
        println!("Attempting to connect to server at {}:{}", self.address, self.port);
        let res = TcpStream::connect((self.address.as_str(), self.port)).await;
        match res {
            Ok(socket) => {
                self.client = Some(socket);
            }
            Err(e) => {
                return Err(Error::from(e));
            }
        }
        Ok(())
    }

    pub async fn read(&mut self, buffer: &mut BytesMut)
        -> Result<usize, Error>
    {
        if let Some(client) = &mut self.client
        {
            let res = client.read_buf(buffer).await;
            match res {
                Ok(0) => {
                    eprintln!("Client disconnected");
                    self.client = None;
                    return Err(Error::ClientDisconnected);
                },
                Ok(n) => return Ok(n),
                Err(e) => {
                    eprintln!("Failed to read data: {e}");
                    self.client = None;
                    return Err(Error::from(e));
                }
            }
        }
        Err(Error::NoClientConnected)
    }

    pub async fn write(&mut self, data: &[u8])
        -> Result<(), Error>
    {
        if let Some(client) = &mut self.client
        {
            let res = client.write_all(data).await;
            match res {
                Ok(_) => return Ok(()),
                Err(e) => {
                    eprintln!("Failed to send data: {e}");
                    self.client = None;
                    return Err(Error::from(e));
                }
            }
        }
        Err(Error::NoClientConnected)
    }

    pub async fn stop(&mut self)
        -> Result<(), Error>
    {
        if let Some(mut client) = self.client.take()
        {
            let _ = client.shutdown().await;
        }
        Ok(())
    }
}
