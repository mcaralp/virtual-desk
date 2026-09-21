use crate::config::{ConfigReceiver, Mode, SshClientConfig, TransportConfig};
use crate::error::Error;
use super::session_client::SessionClient;
use super::transport::Transport;
use super::transport_ssh_client::TransportSshClient;

pub struct ComSshClient
{
    client: SessionClient
}

impl ComSshClient
{
    pub fn new(config_receiver: ConfigReceiver, config: SshClientConfig) -> Self
    {
        let transport = Transport::SshClient(TransportSshClient::new(
            &config.host.address,
            config.host.port,
            config_receiver.config_path(),
            config.host.private_key_path.as_deref(),
            config.host.server_public_key_path.as_deref(),
        ));
        let mode = Mode::from(config);
        let client = SessionClient::new(config_receiver, transport, TransportConfig::from(mode));
        ComSshClient { client }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let res = self.client.run().await;
        res
    }
}
