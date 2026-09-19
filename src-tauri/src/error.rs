use tokio::sync::broadcast;
use crate::command::{CommandRequest, CommandResponse};

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serde YAML error: {0}")]
    SerdeYamlError(#[from] serde_yaml_ng::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Notify error: {0}")]
    NotifyError(#[from] notify::Error),
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
    #[error("Configuration changed")]
    ConfigChanged,
    #[error("Unknown command type: {0}")]
    UnknownCommandType(u32),
    #[error("Unknown cancellation token: {0}")]
    UnknownCancellationToken(String),
    #[error("Child process has no stdout")]
    ChildProcessNoStdout,
    #[error("Frame too large: {size} bytes (max {max})")]
    FrameTooLarge { size: usize, max: usize },
    #[error("Client disconnected")]
    ClientDisconnected,
    #[error("No client connected")]
    NoClientConnected,
    #[error("{0}")]
    Other(String),
}

impl From<&str> for Error
{
    fn from(err: &str) -> Self
    {
        Error::Other(err.into())
    }
}

impl From<String> for Error
{
    fn from(err: String) -> Self
    {
        Error::Other(err)
    }
}
