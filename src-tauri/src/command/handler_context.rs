use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;
use crate::config::{ConfigReceiver, AppConfig};
use crate::emitter::Emitter;
use crate::error::Error;

pub struct TaskData
{
    pub(crate) uuid: String,
    pub(crate) cancel_token: CancellationToken,
    pub(crate) task_handle: tauri::async_runtime::JoinHandle<()>,
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
pub struct HandlerContext
{
    pub emitter: Emitter,
    pub tasks: Arc<Mutex<Vec<TaskData>>>,
    pub config_receiver: ConfigReceiver,
}

impl HandlerContext
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

    pub async fn emit(&self, id: &str, last: bool, data: serde_json::Value)
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
