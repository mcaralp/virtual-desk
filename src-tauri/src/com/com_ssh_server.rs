use super::session_server::SessionServer;
use super::transport::Transport;
use super::transport_ssh_server::TransportSshServer;
use super::window_guard::WindowGuard;
use crate::emitter::{Emitter, TauriEmitter};
use crate::config::{ConfigReceiver, Mode, SshServerConfig, TransportConfig};
use crate::command::CommandBusReceiver;
use crate::error::Error;

pub struct ComSshServer
{
    app: tauri::AppHandle,
    server: SessionServer,
}

impl ComSshServer
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandBusReceiver, config: SshServerConfig, app: &tauri::AppHandle) -> Self
    {
        let emitter = Emitter::Tauri(TauriEmitter::new(app.clone()));
        let transport = Transport::SshServer(TransportSshServer::new(
            &config.host.address,
            config.host.port,
            config_receiver.config_path(),
            config.host.private_key_path.as_deref(),
            config.host.authorized_client_keys.as_deref()
        ));
        let mode = Mode::from(config);
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
