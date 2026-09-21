use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::bytes::BytesMut;
use crate::error::Error;

pub struct TransportTcpServer
{
    listener: Option<TcpListener>,
    client: Option<TcpStream>,
    port: u16,
    address: String,
}

impl TransportTcpServer
{
    pub fn new(address: &str, port: u16) -> Self
    {
        Self { listener: None, client: None, port, address: address.to_string() }
    }

    pub async fn connect(&mut self)
        -> Result<(), Error>
    {
        if self.listener.is_none()
        {
            self.listener = Some(TcpListener::bind((self.address.as_str(), self.port)).await?);
        }

        if self.client.is_none()
        {
            let listener = self.listener.as_mut().unwrap();
            let result = listener.accept().await;
            match result
            {
                Ok((socket, _)) => {
                    self.client = Some(socket);
                    return Ok(())
                }
                Err(e) => {
                    return Err(Error::from(e));
                }
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
            println!("Read result: {:?}", res);
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
        self.listener = None;
        Ok(())
    }
}
