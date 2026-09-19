use crate::config::{ConfigReceiver, Mode, TcpClientConfig, TransportConfig};
use crate::error::Error;
use super::session_client::SessionClient;
use super::transport::Transport;
use super::transport_tcp_client::TransportTcpClient;

pub struct ComTcpClient
{
    client: SessionClient
}

impl ComTcpClient
{
    pub fn new(config_receiver: ConfigReceiver, config: TcpClientConfig) -> Self
    {
        let transport = Transport::TcpClient(TransportTcpClient::new(&config.host.address, config.host.port));
        let mode = Mode::from(config);
        let client = SessionClient::new(config_receiver, transport, TransportConfig::from(mode));
        ComTcpClient { client }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let res = self.client.run().await;
        res
    }
}
