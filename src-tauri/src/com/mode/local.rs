use crate::config::{ConfigReceiver, Mode};
use crate::com::{Error, Emitter, TauriEmitter, WindowGuard};
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
                    self.dispatch.send_command(command);
                    Ok(())
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
        self.dispatch.cancel_all().await;
        window.stop().await?;
        res
    }
}
