use tokio::sync::broadcast;
use crate::error::Error;
use super::CommandRequest;

pub struct CommandBusReceiver
{
    rx: broadcast::Receiver<CommandRequest>,
}

impl CommandBusReceiver
{
    pub(super) fn new(rx: broadcast::Receiver<CommandRequest>) -> Self
    {
        Self { rx }
    }

    /// Receives the next command, skipping (and logging) dropped messages instead of failing.
    pub async fn recv(&mut self) -> Result<CommandRequest, Error>
    {
        loop
        {
            match self.rx.recv().await
            {
                Ok(command) => return Ok(command),
                Err(broadcast::error::RecvError::Lagged(n)) =>
                {
                    eprintln!("Command receiver lagged, skipped {n} messages");
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}
