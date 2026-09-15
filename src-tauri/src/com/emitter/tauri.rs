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
        TauriEmitter { app: app }
    }

    pub async fn emit(&self, id: &String, last: bool, data: &serde_json::Value)
        -> Result<(), Error>
    {
        let response = CommandResponse {
            uuid: id.clone(),
            result: Ok(CommandResponseData {
                last: last,
                data: data.clone()
            })
        };
        self.app.emit("command-response", response)?;
        Ok(())
    }

    pub async fn emit_error(&self, id: &String, error: &String)
        -> Result<(), Error>
    {
        let response = CommandResponse {
            uuid: id.clone(),
            result: Err(error.clone())
        };
        self.app.emit("command-response", response)?;
        Ok(())
    }
}
