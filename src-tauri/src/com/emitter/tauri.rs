use tauri::Emitter;
use crate::com::{CommandResponse, CommandResponseData, Error};

#[derive(Clone)]
pub struct TauriEmitter
{
    pub app: tauri::AppHandle
}

impl TauriEmitter
{
    pub fn new(app: tauri::AppHandle) -> Self
    {
        TauriEmitter { app }
    }

    pub async fn emit(&self, id: &str, last: bool, data: &serde_json::Value)
        -> Result<(), Error>
    {
        let response = CommandResponse {
            uuid: id.to_string(),
            result: Ok(CommandResponseData {
                last: last,
                data: data.clone()
            })
        };
        self.app.emit("command-response", response)?;
        Ok(())
    }

    pub async fn emit_error(&self, id: &str, error: &str)
        -> Result<(), Error>
    {
        let response = CommandResponse {
            uuid: id.to_string(),
            result: Err(error.to_string())
        };
        self.app.emit("command-response", response)?;
        Ok(())
    }
}
