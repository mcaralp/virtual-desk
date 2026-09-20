use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use russh::server::{self, Msg, RunningSession , Session, run_stream};
use russh::keys::{Algorithm, EcdsaCurve, PrivateKey, load_secret_key, parse_public_key_base64, ssh_key};
use russh::{Channel, ChannelStream, Disconnect};
use crate::error::Error;

#[derive(Clone)]
struct SshServerHandler
{
    authorized_client_keys: Option<Vec<ssh_key::Fingerprint>>,
    channel_tx: mpsc::Sender<Channel<Msg>>,
}

impl server::Server for SshServerHandler
{
    type Handler = Self;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self
    {
        println!("New client connected");
        self.clone()
    }
}

impl server::Handler for SshServerHandler
{
    type Error = russh::Error;

    async fn auth_none(&mut self, user: &str) -> Result<server::Auth, Self::Error> {
        println!("Authenticating none for user: {}", user);
        if  self.authorized_client_keys.is_some()
        {
            Ok(server::Auth::reject())
        }
        else
        {
            Ok(server::Auth::Accept)
        }
    }

    async fn auth_publickey(&mut self, _user: &str, key: &russh::keys::ssh_key::PublicKey)
        -> Result<server::Auth, russh::Error>
    {
        println!("Authenticating public key: {:?}", key.fingerprint(ssh_key::HashAlg::Sha256));
        match &self.authorized_client_keys
        {
            Some(keys) =>
            {
                for key_fingerprint in keys
                {
                    if key_fingerprint == &key.fingerprint(ssh_key::HashAlg::Sha256)
                    {
                        return Ok(server::Auth::Accept);
                    }
                }
                Ok(server::Auth::reject())
            },
            None => Ok(server::Auth::Accept),
        }
    }

    async fn channel_open_session(&mut self, channel: Channel<Msg>, reply: server::ChannelOpenHandle, _session: &mut Session)
        -> Result<(), russh::Error>
    {
        reply.accept().await;
        let _ = self.channel_tx.send(channel).await;
        Ok(())
    }
}

pub struct TransportSshServer
{
    address: String,
    port: u16,
    config_path: String,
    private_key_path: Option<String>,
    authorized_client_keys: Option<String>,
    tcp_stream: Option<TcpStream>,
    stream: Option<ChannelStream<Msg>>,
    listener: Option<TcpListener>,
    running_session: Option<RunningSession<SshServerHandler>>,
}

impl TransportSshServer
{
    pub fn new(address: &str, port: u16, config_path: &str, private_key_path: Option<&str>, authorized_client_keys: Option<&str>) -> Self
    {
        Self
        {
            address: address.to_string(),
            port,
            config_path: config_path.to_string(),
            private_key_path: private_key_path.map(|s| s.to_string()),
            authorized_client_keys: authorized_client_keys.map(|s| s.to_string()),
            tcp_stream: None,
            stream: None,
            listener: None,
            running_session: None,
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

    async fn load_authorized_client_keys(&self) -> Result<Option<Vec<ssh_key::Fingerprint>>, Error>
    {

        if let Some(path) = &self.authorized_client_keys
        {
            let path = self.normalize_path(&self.normalize_path(std::path::Path::new(&path)));

            let content = tokio::fs::read_to_string(path).await?;
            let mut fingerprints = Vec::<ssh_key::Fingerprint>::new();

            for line in content.lines()
            {
                let mut split = line.split_whitespace();
                let res = match (split.next(), split.next())
                {
                    (Some(_), Some(key)) => parse_public_key_base64(key),
                    (Some(key), None) => parse_public_key_base64(key),
                    _ => Err(russh::keys::Error::CouldNotReadKey),
                }?;
                let fingerprint = res.fingerprint(ssh_key::HashAlg::Sha256);
                fingerprints.push(fingerprint);
            }
            Ok(Some(fingerprints))
        }
        else
        {
            Ok(None)
        }
    }

    async fn load_private_key(&self) -> Result<PrivateKey, Error>
    {
        if let Some(private_key_path) = &self.private_key_path
        {
            let private_key_path = self.normalize_path(&std::path::Path::new(&private_key_path));
            Ok(load_secret_key(&private_key_path, None)?)
        }
        else
        {
            let alg = Algorithm::Ecdsa { curve: EcdsaCurve::NistP256 };
            Ok(PrivateKey::random(&mut rand::rng(), alg)?)
        }
    }

    pub async fn connect(&mut self)
        -> Result<(), Error>
    {
        if self.listener.is_none()
        {
            let listener = TcpListener::bind((self.address.as_str(), self.port)).await?;
            self.listener = Some(listener);
        }

        if self.stream.is_none() && self.tcp_stream.is_none()
        {
            let (stream, _) = self.listener.as_mut().unwrap().accept().await?;
            self.tcp_stream = Some(stream);

        }

        Ok(())
    }

    pub async fn post_connect(&mut self)
        -> Result<(), Error>
    {
        if let Some(tcp_stream) = self.tcp_stream.take()
        {
            let authorized_client_keys = self.load_authorized_client_keys().await?;
            let host_key = self.load_private_key().await?;

            let config = Arc::new(server::Config { keys: vec![host_key], ..Default::default() });
            let (tx, mut rx) = mpsc::channel(1);
            let handler = SshServerHandler { authorized_client_keys, channel_tx: tx };

            let session = run_stream(config, tcp_stream, handler).await?;

            let channel = rx.recv().await
                .ok_or(Error::NoClientConnected)?;

            self.running_session = Some(session);
            self.stream = Some(channel.into_stream());
        }
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
        if let Some(session) = self.running_session.take()
        {
            session.handle().disconnect(Disconnect::ByApplication, "Server stopped".into(), "en".into()).await?;
        }
        self.stream = None;
        self.running_session = None;
        Ok(())
    }
}
