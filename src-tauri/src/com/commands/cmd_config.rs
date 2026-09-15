use crate::com::Error;
use crate::com::commands::{CommandContext, util};

async fn send_config(com: &CommandContext, id: &String)
    -> Result<(), Error>
{
    let config = com.read_config().await?;
    let value = serde_json::to_value(&config)?;
    com.emit(id, false, &value).await?;
    Ok(())
}

pub async fn cmd_config(com: &CommandContext, id: &String)
    -> Result<(), Error>
{
    send_config(com, id).await?;
    loop
    {
        util::cancellable(com, id, com.watch_config()).await??;
        send_config(com, id).await?;
    }
}
