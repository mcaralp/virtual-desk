use crate::command::{CommandResponse, CommandResponseData};
use crate::error::Error;

#[derive(Clone)]
pub struct MpscEmitter
{
    pub sender: tokio::sync::mpsc::Sender<CommandResponse>
}

impl MpscEmitter
{
    pub fn new(sender: &tokio::sync::mpsc::Sender<CommandResponse>) -> Self
    {
        MpscEmitter { sender: sender.clone() }
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
        self.sender.send(response).await?;
        Ok(())
    }

    pub async fn emit_error(&self, id: &str, error: &str)
        -> Result<(), Error>
    {
        let response = CommandResponse {
            uuid: id.to_string(),
            result: Err(error.to_string())
        };
        self.sender.send(response).await?;
        Ok(())
    }
}
