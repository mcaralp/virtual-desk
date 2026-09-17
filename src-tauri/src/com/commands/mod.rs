mod cmd_cancel;
mod cmd_read_config;
mod cmd_watch_config;
mod cmd_shell;
mod util;

use std::collections::HashMap;
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

#[derive(Clone)]
pub struct CommandContext
{
    pub emitter: Emitter,
    pub cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    pub config_receiver: ConfigReceiver,
}

impl CommandContext
{
    pub fn new(emitter: Emitter, config_receiver: ConfigReceiver) -> Self
    {
        Self
        {
            emitter,
            cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
            config_receiver,
        }
    }

    pub fn get_token(&self, uuid: &str) -> Option<CancellationToken>
    {
        self.cancel_tokens.lock().unwrap().get(uuid).cloned()
    }

    pub fn insert_token(&self, uuid: &str)
    {
        self.cancel_tokens.lock().unwrap().insert(uuid.to_string(), CancellationToken::new());
    }

    pub fn remove_token(&self, uuid: &str)
    {
        self.cancel_tokens.lock().unwrap().remove(uuid);
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
    pub context: CommandContext
}

impl CommandDispatch
{
    pub fn new(emitter: Emitter, config_receiver: ConfigReceiver) -> Self
    {
        Self
        {
            context: CommandContext::new(emitter, config_receiver),
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

    pub async fn send_command(&self, command: CommandRequest)
        -> Result<(), Error>
    {
        self.context.insert_token(&command.uuid);
        let res = self.dispatch(&command).await;
        if let Err(e) = res
        {
            eprintln!("Command {} failed: {e}", command.uuid);
            let _ = self.context.emit_error(&command.uuid, &e.to_string()).await;
        }
        self.context.remove_token(&command.uuid);
        Ok(())
    }

    pub fn cancel_all(&self)
    {
        for token in self.context.cancel_tokens.lock().unwrap().values()
        {
            token.cancel();
        }
    }
}
