mod commands;
mod mode;
mod error;
mod emitter;
mod window_guard;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::{ConfigReceiver, ConfigWatcher, Mode, AppConfig};
use crate::command::{CommandReceiver, CommandState};

pub use mode::{LocalCom, TcpServerCom, TcpClientCom};
pub use error::Error;
pub use emitter::{Emitter, TauriEmitter, MpscEmitter};
pub use window_guard::WindowGuard;


#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CommandRequest
{
    #[serde(default)]
    pub cmd: u32,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResponseData
{
    pub last: bool,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResponse
{
    pub uuid: String,
    pub result: Result<CommandResponseData, String>,
}

pub async fn run(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, app: tauri::AppHandle, config: AppConfig)
    -> Result<(), Error>
{
    match config.mode
    {
        Mode::Local(_) =>
        {
            let mut com = LocalCom::new(config_receiver, command_receiver, app);
            com.run().await
        },
        Mode::TcpServer(remote_config) =>
        {
            let mut com = TcpServerCom::new(config_receiver, command_receiver, remote_config, app);
            com.run().await
        },
        Mode::TcpClient(tcp_client_config) =>
        {
            let mut com = TcpClientCom::new(config_receiver, command_receiver, tcp_client_config);
            com.run().await
        },
        _ => Err(Error::Other("Unknown mode".to_string()))
    }
}

pub fn setup(config_path: String, app: tauri::AppHandle)
    -> Result<(), Error>
{
    tauri::async_runtime::spawn(async move 
    { 
        let config_watcher = ConfigWatcher::new(&config_path);
        let res = config_watcher.start();
        if let Err(e) = res {
            eprintln!("Error starting config watcher: {:?}", e);
            return;
        }
        let command_state = CommandState::new(app.clone());
        command_state.start();

        loop
        {
            match config_watcher.read_config()
            {
                Ok(config) => 
                {
                    let err = run(config_watcher.subscribe(), command_state.subscribe(), app.clone(), config ).await;
                    if let Err(e) = err
                    {
                        eprintln!("Error running com: {:?}", e);
                    }
                },
                Err(e) =>
                {
                    eprintln!("Error reading config: {:?}", e);
                    return;
                },
            }
        }
    });
    Ok(())
}
