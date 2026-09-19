
use crate::com::mode::remote::remote_client::{RemoteClient};
use crate::com::mode::remote::TransportType;
use crate::com::mode::remote::transport::TcpClient;
use crate::config::{ConfigReceiver, Mode, TcpClientConfig, TransportConfig};
use crate::com::{Error};

pub struct TcpClientCom
{
    client: RemoteClient
}

impl TcpClientCom
{
    pub fn new(config_receiver: ConfigReceiver, config: TcpClientConfig) -> Self
    {
        let transport = TransportType::TcpClient(TcpClient::new(&config.host.address, config.host.port));
        let mode = Mode::from(config.clone());
        let client = RemoteClient::new(config_receiver, transport, TransportConfig::from(mode));
        TcpClientCom { client }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let res = self.client.run().await;
        res
    }
}
