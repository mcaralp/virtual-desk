use crate::com::mode::remote::remote_server::RemoteServer;
use crate::com::mode::remote::transport::{TcpServer, TransportType};
use crate::com::{Error, Emitter, TauriEmitter, WindowGuard};
use crate::config::{ConfigReceiver, Mode, TcpServerConfig, TransportConfig};
use crate::command::CommandReceiver;

pub struct TcpServerCom
{
    app: tauri::AppHandle,
    server: RemoteServer,
}

impl TcpServerCom
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, config: TcpServerConfig, app: &tauri::AppHandle) -> Self
    {
        let emitter = Emitter::Tauri(TauriEmitter::new(app.clone()));
        let transport = TransportType::TcpServer(TcpServer::new(config.host.address.as_str(), config.host.port));
        let mode = Mode::from(config.clone());
        let server = RemoteServer::new(config_receiver, command_receiver, emitter, transport, TransportConfig::from(mode));
        Self { app: app.clone(), server }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let window = WindowGuard::new(&self.app)?;
        let res = self.server.run().await;
        window.stop().await?;

        res
    }
}