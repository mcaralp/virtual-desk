mod commands;
mod mode;
mod error;
mod emitter;
mod window_guard;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::{ConfigReceiver, ConfigWatcher, Mode, AppConfig};
use crate::command::{CommandReceiver, CommandState};

pub use mode::LocalCom;
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

pub enum CommandType
{
    ShellCmd,
    ConfigCmd,
    CancelCmd
}

pub fn convert_command_type(cmd: u32) -> Option<CommandType>
{
    match cmd
    {
        1 => Some(CommandType::ShellCmd),
        2 => Some(CommandType::ConfigCmd),
        3 => Some(CommandType::CancelCmd),
        _ => None,
    }
}

pub async fn run(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, config: AppConfig)
    -> Result<(), Error>
{
    match config.mode
    {
        Mode::Local(_) =>
        {
            let mut com = LocalCom::new(config_receiver, command_receiver);
            com.run().await
        },
        Mode::TcpServer(remote_config) =>
        {
            let mut com = mode::tcp_server::TcpServerCom::new(config_receiver, command_receiver, remote_config);
            com.run().await
        },
        Mode::TcpClient(tcp_client_config) =>
        {
            let mut com = mode::tcp_client::TcpClientCom::new(config_receiver, command_receiver, tcp_client_config);
            com.run().await
        },
        _ => Err(Error::Other("Unknown mode".to_string()))
    }
}

pub fn setup(config_watcher: ConfigWatcher, command_state: CommandState)
    -> Result<(), Error>
{
    // let handle: tauri::AppHandle = app.handle().clone();
    // let com = Com::None;
    // let state = Mutex::new(com);
    // app.manage(state);


    tauri::async_runtime::spawn(async move 
    {
        loop
        {
            let config_receiver = config_watcher.subscribe();
            let command_receiver = command_state.get_receiver();

            match config_receiver.read_config()
            {
                Ok(config) => 
                {
                    let err = run(config_receiver.clone(), command_receiver.clone(), config).await;
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
