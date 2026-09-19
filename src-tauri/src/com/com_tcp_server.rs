use super::session_server::SessionServer;
use super::transport::Transport;
use super::transport_tcp_server::TransportTcpServer;
use super::window_guard::WindowGuard;
use crate::emitter::{Emitter, TauriEmitter};
use crate::config::{ConfigReceiver, Mode, TcpServerConfig, TransportConfig};
use crate::command::CommandBusReceiver;
use crate::error::Error;

pub struct ComTcpServer
{
    app: tauri::AppHandle,
    server: SessionServer,
}

impl ComTcpServer
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandBusReceiver, config: TcpServerConfig, app: &tauri::AppHandle) -> Self
    {
        let emitter = Emitter::Tauri(TauriEmitter::new(app.clone()));
        let transport = Transport::TcpServer(TransportTcpServer::new(config.host.address.as_str(), config.host.port));
        let mode = Mode::from(config.clone());
        let server = SessionServer::new(config_receiver, command_receiver, emitter, transport, TransportConfig::from(mode));
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
