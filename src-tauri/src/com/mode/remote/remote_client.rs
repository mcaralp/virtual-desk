use serde_json::from_slice;
use tokio_util::bytes::BytesMut;
use tokio::time::{sleep, Duration};
use crate::config::{ConfigReceiver, TransportConfig};
use crate::com::mode::remote::TransportType;
use crate::com::{CommandRequest, CommandResponse, Error, Emitter, MpscEmitter};
use crate::com::commands::CommandDispatch;
use super::util::{encode, decode_frame};

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub struct RemoteClient
{
    dispatch: CommandDispatch,
    config_receiver: ConfigReceiver,
    response_receiver: tokio::sync::mpsc::Receiver<CommandResponse>,
    buffer: BytesMut,
    transport: TransportType,
    config: TransportConfig,
}

impl RemoteClient
{
    pub fn new(config_receiver: ConfigReceiver, transport: TransportType, config: TransportConfig) -> Self
    {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let emitter = Emitter::Mpsc(MpscEmitter::new(&tx));
        let dispatch = CommandDispatch::new(&emitter, &config_receiver);
        RemoteClient { dispatch, config_receiver, response_receiver: rx, buffer: BytesMut::with_capacity(4096), transport, config }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let res = self.handle_connection().await;
        res
    }

    async fn handle_connection(&mut self)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = self.transport.connect() => {
                    match result {
                        Ok(_) => {
                            println!("Connected to remote server");
                            self.buffer.clear();
                            self.handle_socket().await
                        }
                        Err(e) => {
                            eprintln!("Failed to connect to remote server: {e}");
                            Ok(())
                        }
                    }
                }
                _ = self.config_receiver.recv() => {
                    self.check_config()
                }
            };

            self.dispatch.cancel_all().await;
            // Clear any pending responses from the response receiver
            //to avoid processing stale responses after a reconnect.
            while self.response_receiver.try_recv().is_ok() {}

            // Only a config change tears the mode down; any other error just triggers a reconnect.
            match result
            {
                Ok(()) => {}
                Err(Error::ConfigChanged) => return Err(Error::ConfigChanged),
                Err(e) => eprintln!("Connection error, reconnecting: {e}"),
            }

            sleep(RECONNECT_DELAY).await;
        }
    }

    async fn handle_socket(&mut self)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = self.transport.read(&mut self.buffer) => {
                    match result {
                        Ok(_) => {
                            self.handle_incoming_data().await?;
                        }
                        Err(e) => return Err(e)
                    }
                    Ok(())
                }
                res = self.response_receiver.recv() => {
                    if let Some(response) = res
                    {
                        let data = encode(0, &response)?;
                        self.transport.write(&data).await?;
                    }
                    Ok(())
                }
                _ = self.config_receiver.recv() => {
                    self.check_config()?;
                    Ok(())
                }
            };
            result?;
        }
    }

    async fn handle_incoming_data(&mut self)
        -> Result<(), Error>
    {
        loop
        {
            let Some((cmd, payload)) = decode_frame(&mut self.buffer)? else { break };
            match cmd
            {
                0 => {
                    let command = from_slice::<CommandRequest>(&payload)?;
                    self.dispatch.send_command(&command);
                }
                _ => eprintln!("Ignoring frame with unknown command id: {cmd}"),
            }
        }
        Ok(())
    }

    fn check_config(&self) -> Result<(), Error>
    {
        let result = self.config_receiver.read_config();
        if let Ok(config) = result
        {
            let transport_config = TransportConfig::from(config.mode);
            if self.config != transport_config
            {
                return Err(Error::ConfigChanged);
            }
        }
        Ok(())
    }
}
