mod window_guard;
mod com_local;
mod com_tcp_client;
mod com_tcp_server;
mod com_ssh_client;
mod com_ssh_server;
mod session_client;
mod session_server;
mod transport;
mod transport_tcp_client;
mod transport_tcp_server;
mod transport_ssh_client;
mod transport_ssh_server;
mod frame;

use tokio::time::{sleep, Duration};
use crate::config::{ConfigReceiver, ConfigWatcher, Mode, AppConfig};
use crate::command::{CommandBusReceiver, CommandBus};
use crate::error::Error;

pub use com_local::ComLocal;
pub use com_tcp_client::ComTcpClient;
pub use com_tcp_server::ComTcpServer;
pub use com_ssh_client::ComSshClient;
pub use com_ssh_server::ComSshServer;

pub async fn run(config_receiver: ConfigReceiver, command_receiver: CommandBusReceiver, app: &tauri::AppHandle, config: AppConfig)
    -> Result<(), Error>
{
    match config.mode
    {
        Mode::Local(_) =>
        {
            let mut com = ComLocal::new(config_receiver, command_receiver, app);
            com.run().await
        },
        Mode::TcpServer(remote_config) =>
        {
            let mut com = ComTcpServer::new(config_receiver, command_receiver, remote_config, app);
            com.run().await
        },
        Mode::TcpClient(tcp_client_config) =>
        {
            let mut com = ComTcpClient::new(config_receiver, tcp_client_config);
            com.run().await
        },
        Mode::SshClient(ssh_client_config) =>
        {
            let mut com = ComSshClient::new(config_receiver, ssh_client_config);
            com.run().await
        },
        Mode::SshServer(ssh_server_config) =>
        {
            let mut com = ComSshServer::new(config_receiver, command_receiver, ssh_server_config, app);
            com.run().await
        }
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
        let command_state = CommandBus::new(&app);
        command_state.start();

        loop
        {
            match config_watcher.read_config()
            {
                Ok(config) => 
                {
                    let err = run(config_watcher.subscribe(), command_state.subscribe(), &app, config ).await;
                    match err
                    {
                        Ok(()) => {},
                        Err(Error::ConfigChanged) => println!("Configuration changed, restarting com"),
                        Err(e) => eprintln!("Error running com: {:?}", e),
                    }
                },
                Err(e) =>
                {
                    eprintln!("Error reading config, waiting for changes: {:?}", e);
                    let mut receiver = config_watcher.subscribe();
                    let _ = receiver.recv().await;
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });
    Ok(())
}
