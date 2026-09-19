
use super::CommandContext;
use crate::com::Error;

pub async fn cmd_config(com: &CommandContext, id: &str)
    -> Result<(), Error>
{
    let config = com.read_config().await?;
    let value = serde_json::to_value(&config)?;
    com.emit(id, true, &value).await?;
    Ok(())
}
