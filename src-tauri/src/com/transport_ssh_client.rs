use std::sync::Arc;
use tokio_util::bytes::BytesMut;
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
        let key = load_secret_key(private_key, None)?;
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

    pub async fn connect(&mut self)
        -> Result<(), Error>
    {
        if self.stream.is_some()
        {
            return Ok(());
        }

        let expected_server_key = match &self.server_public_key_path
        {

            Some(path) =>
            {
                let path = self.normalize_path(&std::path::Path::new(&path));
                let key = load_public_key(path)?;
                Some(key.fingerprint(ssh_key::HashAlg::Sha256))
            }
            None => None,
        };

        let config = Arc::new(client::Config::default());
        let handler = ClientHandler { expected_server_key };
        let mut session = client::connect(config, (self.address.as_str(), self.port), handler).await?;
    
        if let Some(private_key) = &self.private_key_path
        {
            if self.authenticate_publickey(&mut session, private_key).await.is_err()
            {
                self.authenticate_none(&mut session).await?;
            }
        }
        else
        {
            self.authenticate_none(&mut session).await?;
        }

        let channel = session.channel_open_session().await?;
        self.stream = Some(channel.into_stream());
        Ok(())
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
        println!("Writing data: {:?}", data);
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
        if let Some(stream) = &mut self.stream
        {
            let _ = stream.shutdown().await;
        }
        self.stream = None;
        Ok(())
    }
}
