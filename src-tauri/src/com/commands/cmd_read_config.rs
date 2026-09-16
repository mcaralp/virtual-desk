use crate::com::Error;
use crate::com::commands::{CommandContext};

pub async fn cmd_config(com: &CommandContext, id: &String)
    -> Result<(), Error>
{
    let config = com.read_config().await?;
    let value = serde_json::to_value(&config)?;
    com.emit(id, true, &value).await?;
    Ok(())
}
