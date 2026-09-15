use serde::Deserialize;
use crate::com::Error;
use crate::com::commands::CommandContext;

#[derive(Debug, Deserialize)]
struct CancelCommandParams
{
    pub uuid: String,
}

pub async fn cmd_cancel(com: &CommandContext, id: &String, params: &serde_json::Value)
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
