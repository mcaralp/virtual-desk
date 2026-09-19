mod cmd_cancel;
mod cmd_read_config;
mod cmd_watch_config;
mod cmd_shell;
mod util;

use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use crate::config::{ConfigReceiver, AppConfig};
use crate::com::{Emitter, CommandRequest, Error};

pub enum CommandType
{
    ShellCmd,
    ReadConfigCmd,
    WatchConfigCmd,
    CancelCmd
}

pub fn convert_command_type(cmd: u32) -> Option<CommandType>
{
    match cmd
    {
        1 => Some(CommandType::ShellCmd),
        2 => Some(CommandType::ReadConfigCmd),
        3 => Some(CommandType::WatchConfigCmd),
        4 => Some(CommandType::CancelCmd),
        _ => None,
    }
}

pub struct TaskData
{
    uuid: String,
    cancel_token: CancellationToken,
    task_handle: tauri::async_runtime::JoinHandle<()>,
}

impl TaskData
{
    pub fn new(uuid: &str, task_handle: tauri::async_runtime::JoinHandle<()>) -> Self
    {
        Self
        {
            uuid: uuid.to_string(),
            cancel_token: CancellationToken::new(),
            task_handle,
        }
    }
}

#[derive(Clone)]
pub struct CommandContext
{
    pub emitter: Emitter,
    pub tasks: Arc<Mutex<Vec<TaskData>>>,
    pub config_receiver: ConfigReceiver,
}

impl CommandContext
{
    pub fn new(emitter: Emitter, config_receiver: ConfigReceiver) -> Self
    {
        Self
        {
            emitter,
            tasks: Arc::new(Mutex::new(Vec::new())),
            config_receiver,
        }
    }

    pub fn get_token(&self, uuid: &str) -> Option<CancellationToken>
    {
        let tasks = self.tasks.lock().unwrap();
        for task in tasks.iter()
        {
            if task.uuid == uuid
            {
                return Some(task.cancel_token.clone());
            }
        }
        None
    }

    pub fn insert_task(&self, uuid: &str, handle: tauri::async_runtime::JoinHandle<()>)
    {
        self.tasks.lock().unwrap().push(TaskData::new(uuid, handle));
    }

    pub fn remove_task(&self, uuid: &str)
    {
        let mut tasks = self.tasks.lock().unwrap();
        for pos in 0..tasks.len()
        {
            if tasks[pos].uuid == uuid
            {
                tasks.remove(pos);
                return;
            }
        }
    }

    pub async fn emit(&self, id: &str, last: bool, data: &serde_json::Value)
        -> Result<(), Error>
    {
        self.emitter.emit(id, last, data).await?;
        Ok(())
    }

    pub async fn emit_error(&self, id: &str, error: &str)
        -> Result<(), Error>
    {
        self.emitter.emit_error(id, error).await?;
        Ok(())
    }

    pub async fn watch_config(&self)
        -> Result<(), Error>
    {
        self.config_receiver.clone().recv().await?;
        Ok(())
    }

    pub async fn read_config(&self)
        -> Result<AppConfig, Error>
    {
        let config = self.config_receiver.read_config()?;
        Ok(config)
    }
}

#[derive(Clone)]
pub struct CommandDispatch
{
    context: CommandContext

}

impl CommandDispatch
{
    pub fn new(emitter: &Emitter, config_receiver: &ConfigReceiver) -> Self
    {
        Self
        {
            context: CommandContext::new(emitter.clone(), config_receiver.clone()),
        }
    }

    async fn dispatch(&self, command: &CommandRequest)
        -> Result<(), Error>
    {
        let cmd = convert_command_type(command.cmd)
            .ok_or(Error::Other(format!("Unknown command type: {}", command.cmd)))?;
        println!("Command: {:?}", command);
        match cmd
        {
            CommandType::ShellCmd =>
            {
                cmd_shell::cmd_shell(&self.context, &command.uuid, &command.params).await?;
            }
            CommandType::ReadConfigCmd =>
            {
                cmd_read_config::cmd_config(&self.context, &command.uuid).await?;
            }
            CommandType::WatchConfigCmd =>
            {
                cmd_watch_config::cmd_config(&self.context, &command.uuid).await?;
            }
            CommandType::CancelCmd =>
            {
                cmd_cancel::cmd_cancel(&self.context, &command.uuid, &command.params).await?;
            }
        }
        Ok(())
    }

    pub fn send_command(&self, command: &CommandRequest)
    {
        let command_copy = command.clone();
        let dispatch = self.clone();
        // Cooperative scheduling: the task only runs once this fn yields, so registering after spawn is safe.
        let handle = tauri::async_runtime::spawn(async move
        {
            let res = dispatch.dispatch(&command_copy).await;
            if let Err(e) = res
            {
                eprintln!("Command {} failed: {e}", command_copy.uuid);
                let _ = dispatch.context.emit_error(&command_copy.uuid, &e.to_string()).await;
            }
            dispatch.context.remove_task(&command_copy.uuid);
        });

        self.context.insert_task(&command.uuid, handle);
    }

    pub async fn cancel_all(&self)
    {
        // Take ownership of the tasks so the lock is released before awaiting the handles.
        let tasks = std::mem::take(&mut *self.context.tasks.lock().unwrap());
        for task in &tasks
        {
            task.cancel_token.cancel();
        }
        for task in tasks
        {
            let _ = task.task_handle.await;
        }
    }
}
