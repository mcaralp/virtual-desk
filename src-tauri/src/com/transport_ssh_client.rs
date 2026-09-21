use std::sync::Arc;
use tokio_util::bytes::BytesMut;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use russh::client::{self, Msg};
use russh::keys::{load_public_key, load_secret_key, PrivateKeyWithHashAlg, ssh_key};
use russh::ChannelStream;
use crate::error::Error;

struct ClientHandler
{
    expected_server_key: Option<ssh_key::Fingerprint>,
}

impl client::Handler for ClientHandler
{
    type Error = russh::Error;

    async fn check_server_key(&mut self, server_public_key: &russh::keys::PublicKeyOrCertificate)
        -> Result<bool, Self::Error>
    {
        let Some(expected) = &self.expected_server_key else { return Ok(true) };
        let fingerprint = server_public_key.public_key().fingerprint(ssh_key::HashAlg::Sha256);
        Ok(&fingerprint == expected)
    }
}

pub struct TransportSshClient
{
    address: String,
    port: u16,
    config_path: String,
    private_key_path: Option<String>,
    server_public_key_path: Option<String>,
    tcp_stream: Option<TcpStream>,
    stream: Option<ChannelStream<Msg>>,
}

impl TransportSshClient
{
    pub fn new(address: &str, port: u16, config_path: &str, private_key_path: Option<&str>, server_public_key_path: Option<&str>) -> Self
    {
        Self
        {
            address: address.to_string(),
            port,
            config_path: config_path.to_string(),
            private_key_path: private_key_path.map(|s| s.to_string()),
            server_public_key_path: server_public_key_path.map(|s| s.to_string()),
            tcp_stream: None,
            stream: None,
        }
    }

    fn normalize_path(&self, path: &std::path::Path) -> std::path::PathBuf
    {
        if path.is_relative()
        {
            let config_path = std::path::Path::new(&self.config_path).parent().unwrap_or(std::path::Path::new(""));
            config_path.join(path)
        }
        else
        {
            path.to_path_buf()
        }
    }

    async fn authenticate_publickey(&self, session: &mut client::Handle<ClientHandler>, private_key: &str)
        -> Result<(), Error>
    {
        let key = self.load_private_key(private_key).await?;
        let best_hash = session.best_supported_rsa_hash().await?.flatten();
        let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key), best_hash);

        let auth = session.authenticate_publickey("virtualdesk", key_with_hash).await?;

        if !auth.success()
        {
            return Err(Error::SshAuthenticationFailed);
        }

        Ok(())
    }

    async fn authenticate_none(&self, session: &mut client::Handle<ClientHandler>)
        -> Result<(), Error>
    {
        let auth = session.authenticate_none("virtualdesk").await?;

        if !auth.success()
        {
            return Err(Error::SshAuthenticationFailed);
        }

        Ok(())
    }

    pub async fn load_private_key(&self, private_key_path: &str)
        -> Result<ssh_key::PrivateKey, Error>
    {
        let private_key_path = self.normalize_path(&std::path::Path::new(&private_key_path));
        Ok(load_secret_key(&private_key_path, None)?)
    }

    pub async fn load_server_public_key(&self) -> Result<Option<ssh_key::Fingerprint>, Error>
    {
        if let Some(path) = &self.server_public_key_path
        {
            let path = self.normalize_path(&std::path::Path::new(&path));
            let key = load_public_key(path)?;
            Ok(Some(key.fingerprint(ssh_key::HashAlg::Sha256)))
        }
        else
        {
            Ok(None)
        }
    }

    pub async fn connect(&mut self)
        -> Result<(), Error>
    {
        if self.stream.is_some() || self.tcp_stream.is_some()
        {
            return Ok(());
        }
    
        let res = TcpStream::connect((self.address.as_str(), self.port)).await;
        match res{
            Ok(socket) => {
                self.tcp_stream = Some(socket);
            }
            Err(e) => {
                return Err(Error::from(e));
            }
        }
        Ok(())
    }

    pub async fn post_connect(&mut self)
        -> Result<(), Error>
    {
        if let Some(tcp_stream) = self.tcp_stream.take()
        {
            let expected_server_key = self.load_server_public_key().await?;

            let config = Arc::new(client::Config::default());
            let handler = ClientHandler { expected_server_key };
            let mut session = client::connect_stream(config, tcp_stream, handler).await?;
        
            if let Some(private_key) = &self.private_key_path
            {
                self.authenticate_publickey(&mut session, private_key).await?;
            }
            else
            {
                self.authenticate_none(&mut session).await?;
            }

            let channel = session.channel_open_session().await?;
            self.stream = Some(channel.into_stream());
            Ok(())
        }
        else
        {
            Err(Error::NoClientConnected)
        }

    }

    pub async fn read(&mut self, buffer: &mut BytesMut)
        -> Result<usize, Error>
    {
        if let Some(stream) = &mut self.stream
        {
            let res = stream.read_buf(buffer).await;
            match res
            {
                Ok(0) => {
                    self.stream = None;
                    Err(Error::ClientDisconnected)
                },
                Ok(n) => Ok(n),
                Err(e) => {
                    self.stream = None;
                    Err(Error::from(e))
                }
            }
        }
        else
        {
            Err(Error::NoClientConnected)
        }
    }

    pub async fn write(&mut self, data: &[u8])
        -> Result<(), Error>
    {
        if let Some(stream) = &mut self.stream
        {
            let res = stream.write_all(data).await;
            match res
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    self.stream = None;
                    Err(Error::from(e))
                }
            }
        }
        else
        {
            Err(Error::NoClientConnected)
        }
    }

    pub async fn stop(&mut self)
        -> Result<(), Error>
    {
        if let Some(mut tcp_stream) = self.tcp_stream.take()
        {
            let _ = tcp_stream.shutdown().await;
        }
        if let Some(mut stream) = self.stream.take()
        {
            let _ = stream.shutdown().await;
        }
    
        Ok(())
    }
}
