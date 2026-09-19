use serde::Deserialize;
use super::CommandContext;
use crate::com::Error;

#[derive(Debug, Deserialize)]
struct CancelCommandParams
{
    pub uuid: String,
}

pub async fn handler_cancel(com: &CommandContext, id: &str, params: &serde_json::Value)
    -> Result<(), Error>
{
    let cancel_params: CancelCommandParams = serde_json::from_value(params.clone())?;
    if let Some(token) = com.get_token(&cancel_params.uuid)
    {
        token.cancel();
    }
    println!("Cancelled command with UUID: {}", cancel_params.uuid);
    com.emit(id, true, &serde_json::Value::Null).await?;
    Ok(())
}
