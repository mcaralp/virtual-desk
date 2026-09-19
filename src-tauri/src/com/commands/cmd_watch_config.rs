use super::{CommandContext, util};
use crate::com::Error;

pub async fn cmd_config(com: &CommandContext, id: &str)
    -> Result<(), Error>
{
    loop
    {
        util::cancellable(com, id, com.watch_config()).await??;
        let config = com.read_config().await?;
        let value = serde_json::to_value(&config)?;
        com.emit(id, false, &value).await?;
    }
}
