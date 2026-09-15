use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::com::{CommandRequest, Error};

pub struct CommandReceiver
{
    receiver: broadcast::Receiver<CommandRequest>,
    app: AppHandle,
}

impl CommandReceiver
{
    pub fn new(receiver: broadcast::Receiver<CommandRequest>, app: AppHandle) -> Self
    {
        Self { receiver, app }
    }

    pub fn register_state<T>(&self, handler: T)
    where
        T: Send + Sync + 'static,
    {
        if let Some(state) = self.app.try_state::<Mutex<Option<T>>>()
        {
            let mut state = state.lock().unwrap();
            *state = Some(handler);
        }
        else
        {
            self.app.manage(Mutex::new(Some(handler)));
        }
    }

    pub async fn recv(&mut self) -> Result<CommandRequest, broadcast::error::RecvError>
    {
        self.receiver.recv().await
    }

    pub fn app(&self) -> AppHandle
    {
        self.app.clone()
    }
}

impl Clone for CommandReceiver
{
    fn clone(&self) -> Self
    {
        Self {
            receiver: self.receiver.resubscribe(),
            app: self.app.clone(),
        }
    }
}

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
        -> CommandReceiver
    {
        let receiver = self.subscribe();
        self.app.manage(self.clone());
        CommandReceiver::new(receiver, self.app.clone())
    }

    pub fn get_receiver(&self) -> CommandReceiver
    {
        CommandReceiver::new(self.sender.subscribe(), self.app.clone())
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
