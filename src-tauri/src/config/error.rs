
#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error("Serde YAML error: {0}")]
    SerdeYamlError(#[from] serde_yaml_ng::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Notify error: {0}")]
    NotifyError(#[from] notify::Error),
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] tokio::sync::broadcast::error::RecvError),
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
