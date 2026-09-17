use tokio::sync::broadcast;
use crate::com::{CommandRequest, CommandResponse};
use crate::config;

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] config::Error),
    #[error("Serde JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Mpsc send error: {0}")]
    MpscError(#[from] tokio::sync::mpsc::error::SendError<CommandResponse>),
    #[error("Broadcast send error: {0}")]
    BroadcastSendError(#[from] broadcast::error::SendError<CommandRequest>),
    #[error("Broadcast recv error: {0}")]
    BroadcastRecvError(#[from] broadcast::error::RecvError),
    #[error("Try from int error: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
    #[error("Operation cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

impl From<String> for Error
{
    fn from(err: String) -> Self
    {
        Error::Other(err)
    }
}
