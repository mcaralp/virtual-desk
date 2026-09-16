use crate::config::{ConfigReceiver, Mode, TcpClientConfig};
use crate::com::{CommandRequest, Error, Emitter, MpscEmitter};
use crate::command::CommandReceiver;
use crate::com::commands::CommandDispatch;

pub struct TcpClientCom
{
    dispatch: CommandDispatch,
    config_receiver: ConfigReceiver,
    command_receiver: CommandReceiver,
}

impl TcpClientCom
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, _config: TcpClientConfig) -> Self
    {
        // TODO: forward responses over the TCP connection once the client transport is implemented
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let emitter = Emitter::Mpsc(MpscEmitter::new(&tx));
        let dispatch = CommandDispatch::new(emitter, config_receiver.clone());
        TcpClientCom { dispatch, config_receiver, command_receiver }
    }

    pub fn send_command(&self, command: CommandRequest)
        -> Result<(), Error>
    {
        let dispatch = self.dispatch.clone();
        tauri::async_runtime::spawn(async move
        {
            if let Err(e) = dispatch.send_command(command).await
            {
                eprintln!("Failed to send tcp client command: {e}");
            }
        });
        Ok(())
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select!
            {
                res = self.config_receiver.recv() =>
                {
                    res?;
                    let config = self.config_receiver.read_config();
                    if let Ok(config) = config
                    {
                        if !matches!(config.mode, Mode::TcpClient(_))
                        {
                            return Err(Error::Other("Configuration changed to non-tcp-client mode".to_string()));
                        }
                    }
                    Ok(())
                }
                res = self.command_receiver.recv() =>
                {
                    let command = res?;
                    self.send_command(command)
                }
            };
            result?;
        }
    }
}
