use crate::config::{ConfigReceiver, Mode};
use crate::emitter::{Emitter, TauriEmitter};
use crate::command::{CommandBusReceiver, CommandDispatch};
use crate::error::Error;
use super::window_guard::WindowGuard;

pub struct ComLocal
{
    dispatch: CommandDispatch,
    config_receiver: ConfigReceiver,
    command_receiver: CommandBusReceiver,
    app: tauri::AppHandle,
}

impl ComLocal
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandBusReceiver, app: &tauri::AppHandle) -> Self
    {
        let emitter = TauriEmitter::new(app.clone());
        let dispatch = CommandDispatch::new(&Emitter::Tauri(emitter), &config_receiver);
        ComLocal { dispatch, config_receiver, command_receiver, app: app.clone() }
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
                    self.dispatch.send_command(&command);
                    Ok(())
                }
            };
            result?;
        }
    }
    
    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let window = WindowGuard::new(&self.app)?;
        let res = self.monitor().await;
        self.dispatch.cancel_all().await;
        window.stop().await?;
        res
    }
}
