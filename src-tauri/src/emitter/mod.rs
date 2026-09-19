mod mpsc;
mod tauri;

use serde_json::Value;
use crate::error::Error;

pub use tauri::TauriEmitter;
pub use mpsc::MpscEmitter;

#[derive(Clone)]
pub enum Emitter
{
    Tauri(tauri::TauriEmitter),
    Mpsc(mpsc::MpscEmitter)
}

impl Emitter
{
    pub async fn emit(&self, id: &str, last: bool, data: Value)
        -> Result<(), Error>
    {
        match self
        {
            Emitter::Tauri(emitter) =>
            {
                emitter.emit(id, last, data).await?;
            }
            Emitter::Mpsc(emitter) =>
            {
                emitter.emit(id, last, data).await?;
            }
        }
        Ok(())
    }

    pub async fn emit_error(&self, id: &str, error: &str)
        -> Result<(), Error>
    {
        match self
        {
            Emitter::Tauri(emitter) =>
            {
                emitter.emit_error(id, error).await?;
            }
            Emitter::Mpsc(emitter) =>
            {
                emitter.emit_error(id, error).await?;
            }
        }
        Ok(())
    }
}
