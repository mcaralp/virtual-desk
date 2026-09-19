use crate::config::ConfigReceiver;
use crate::emitter::Emitter;
use crate::error::Error;
use super::{CommandRequest, HandlerContext};
use super::command_type::{convert_command_type, CommandType};
use super::{handler_command_cancel, handler_command_read_config, handler_command_shell, handler_command_watch_config};

#[derive(Clone)]
pub struct CommandDispatch
{
    context: HandlerContext

}

impl CommandDispatch
{
    pub fn new(emitter: &Emitter, config_receiver: &ConfigReceiver) -> Self
    {
        Self
        {
            context: HandlerContext::new(emitter.clone(), config_receiver.clone()),
        }
    }

    async fn dispatch(&self, command: &CommandRequest)
        -> Result<(), Error>
    {
        let cmd = convert_command_type(command.cmd)
            .ok_or(Error::UnknownCommandType(command.cmd))?;
        println!("Command: {:?}", command);
        match cmd
        {
            CommandType::ShellCmd =>
            {
                handler_command_shell::handler_shell(&self.context, &command.uuid, &command.params).await?;
            }
            CommandType::ReadConfigCmd =>
            {
                handler_command_read_config::handler_read_config(&self.context, &command.uuid).await?;
            }
            CommandType::WatchConfigCmd =>
            {
                handler_command_watch_config::handler_watch_config(&self.context, &command.uuid).await?;
            }
            CommandType::CancelCmd =>
            {
                handler_command_cancel::handler_cancel(&self.context, &command.uuid, &command.params).await?;
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
