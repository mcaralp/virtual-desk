use std::future::Future;

use crate::com::Error;
use crate::com::commands::CommandContext;

pub async fn cancellable<F>(com: &CommandContext, id: &str, future: F)
    -> Result<F::Output, Error>
where
    F: Future,
{
    let token = com.get_token(id)
        .ok_or_else(|| Error::Other(format!("Unknown cancellation token: {id}")))?;
    tokio::select! {
        result = future => Ok(result),
        _ = token.cancelled() => Err(Error::Cancelled),
    }
}

