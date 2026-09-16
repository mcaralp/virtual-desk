use tauri::{AppHandle, Manager};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::com::{CommandRequest, Error};

pub type CommandReceiver = broadcast::Receiver<CommandRequest>;

#[derive(Clone)]
pub struct CommandState
{
    sender: Arc<broadcast::Sender<CommandRequest>>,
    app: AppHandle
}

impl CommandState
{
    pub fn new(app: AppHandle) -> Self
    {
        let (sender, _) = broadcast::channel(64);
        let sender = Arc::new(sender);
        Self { sender, app }
    }

    pub fn send(&self, item: CommandRequest) -> Result<usize, Error>
    {
        let res = self.sender.send(item)?;
        Ok(res)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CommandRequest>
    {
        self.sender.subscribe()
    }

    pub fn start(&self)
    {
        if self.app.try_state::<CommandState>().is_none()
        {
            self.app.manage(self.clone());
        }
    }
}

#[tauri::command]
pub fn exec_command(command: CommandRequest, app: AppHandle)
    -> Result<(), String>
{
    let state = app.state::<CommandState>();
    // no error if there are currently no subscribers
    let _ = state.send(command);
    Ok(())
}
