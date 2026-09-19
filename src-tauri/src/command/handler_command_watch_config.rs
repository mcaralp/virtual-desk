use super::{HandlerContext, util};
use crate::error::Error;

pub async fn handler_watch_config(com: &HandlerContext, id: &str)
    -> Result<(), Error>
{
    loop
    {
        util::cancellable(com, id, com.watch_config()).await??;
        let config = com.read_config().await?;
        let value = serde_json::to_value(&config)?;
        com.emit(id, false, value).await?;
    }
}
