use tokio::net::{TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::bytes::BytesMut;
use crate::com::Error;

pub struct TcpClient
{
    client: Option<TcpStream>,
    port: u16,
    address: String,
}

impl TcpClient
{
    pub fn new(address: &str, port: u16) -> Self
    {
        Self { client: None, port, address: address.to_string() }
    }

    pub async fn connect(&mut self)
        -> Result<(), Error>
    {
        if self.client.is_none()
        {
            let res = TcpStream::connect((self.address.as_str(), self.port)).await;
            match res {
                Ok(socket) => {
                    println!("Successfully connected to server at {}:{}", self.address, self.port);
                    self.client = Some(socket);
                }
                Err(e) => {
                    eprintln!("Failed to connect to server: {e}");
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
            match res {
                Ok(0) => {
                    eprintln!("Client disconnected");
                    self.client = None;
                    return Err(Error::Other("Client disconnected".to_string()));
                },
                Ok(n) => return Ok(n),
                Err(e) => {
                    eprintln!("Failed to read data: {e}");
                    self.client = None;
                    return Err(Error::from(e));
                }
            }
        }
        Err(Error::Other("No client connected".to_string()))
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
        Err(Error::Other("No client connected".to_string()))
    }
}
