use crate::com::{CommandResponse, CommandResponseData, Error};

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
        self.sender.send(response).await?;
        Ok(())
    }

    pub async fn emit_error(&self, id: &String, error: &String)
        -> Result<(), Error>
    {
        let response = CommandResponse {
            uuid: id.clone(),
            result: Err(error.clone())
        };
        self.sender.send(response).await?;
        Ok(())
    }
}
