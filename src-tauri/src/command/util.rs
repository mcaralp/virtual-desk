use std::future::Future;

use crate::error::Error;
use super::HandlerContext;

pub async fn cancellable<F>(com: &HandlerContext, id: &str, future: F)
    -> Result<F::Output, Error>
where
    F: Future,
{
    let token = com.get_token(id)
        .ok_or_else(|| Error::UnknownCancellationToken(id.to_string()))?;
    tokio::select! {
        result = future => Ok(result),
        _ = token.cancelled() => Err(Error::Cancelled),
    }
}
