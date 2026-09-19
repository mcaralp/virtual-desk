use tauri::{AppHandle, Manager};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::error::Error;
use super::{CommandRequest, CommandBusReceiver};

#[derive(Clone)]
pub struct CommandBus
{
    sender: Arc<broadcast::Sender<CommandRequest>>,
    app: AppHandle
}

impl CommandBus
{
    pub fn new(app: &AppHandle) -> Self
    {
        let (sender, _) = broadcast::channel(64);
        let sender = Arc::new(sender);
        Self { sender, app: app.clone() }
    }

    pub fn send(&self, item: &CommandRequest) -> Result<usize, Error>
    {
        let res = self.sender.send(item.clone())?;
        Ok(res)
    }

    pub fn subscribe(&self) -> CommandBusReceiver
    {
        CommandBusReceiver::new(self.sender.subscribe())
    }

    pub fn start(&self)
    {
        if self.app.try_state::<CommandBus>().is_none()
        {
            self.app.manage(self.clone());
        }
    }
}
