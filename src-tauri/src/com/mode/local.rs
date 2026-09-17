use crate::config::{ConfigReceiver, Mode};
use crate::com::{CommandRequest, Error, Emitter, TauriEmitter, WindowGuard};
use crate::command::{CommandReceiver};
use crate::com::commands::CommandDispatch;

pub struct LocalCom
{
    dispatch: CommandDispatch,
    config_receiver: ConfigReceiver,
    command_receiver: CommandReceiver,
    app: tauri::AppHandle,
}

impl LocalCom
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, app: tauri::AppHandle) -> Self
    {
        let emitter = TauriEmitter::new(app.clone());
        let dispatch = CommandDispatch::new(Emitter::Tauri(emitter), config_receiver.clone());
        LocalCom { dispatch, config_receiver, command_receiver, app }
    }

    fn send_command(&self, command: CommandRequest)
        -> Result<(), Error>
    {
        let dispatch = self.dispatch.clone();
        tauri::async_runtime::spawn(async move
        {
            if let Err(e) = dispatch.send_command(command).await
            {
                eprintln!("Failed to send local command: {e}");
            }
        });
        Ok(())
    }

    async fn monitor(&mut self)
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
                        if !matches!(config.mode, Mode::Local(_))
                        {
                            return Err(Error::ConfigChanged);
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
    
    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let window = WindowGuard::new(self.app.clone())?;
        let res = self.monitor().await;
        self.dispatch.cancel_all();
        window.stop().await?;
        res
    }
}
