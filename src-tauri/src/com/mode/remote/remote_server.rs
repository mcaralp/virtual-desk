use serde_json::from_slice;
use tokio_util::bytes::BytesMut;

use crate::com::mode::remote::{TransportType};
use crate::com::{Error, CommandResponse, Emitter};
use crate::config::{ConfigReceiver, TransportConfig};
use crate::command::CommandReceiver;

use super::util::{encode, decode_frame};

pub struct RemoteServer
{
    emitter: Emitter,
    config_receiver: ConfigReceiver,
    command_receiver: CommandReceiver,
    buffer: BytesMut,
    requests_in_progress: Vec<String>,
    transport: TransportType,
    config: TransportConfig
}

impl RemoteServer
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, emitter: Emitter, transport: TransportType, config: TransportConfig) -> Self
    {
        Self { emitter, config_receiver, command_receiver, buffer: BytesMut::with_capacity(4096), requests_in_progress: Vec::new(), transport, config }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        self.accept_connections().await
    }

    async fn accept_connections(&mut self)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = self.transport.connect() => {
                    match result {
                        Ok(()) => {
                            println!("Accepted connection");
                            self.buffer.clear();
                            self.handle_connection().await
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {e}");
                            Ok(())
                        }
                    }
                }
                res = self.command_receiver.recv() => {
                    let command = res?;
                    self.emitter.emit_error(&command.uuid, "Server not connected").await
                }
                _ = self.config_receiver.recv() => {
                    self.check_config()
                }
            };

            for uuid in &self.requests_in_progress
            {
                let _ = self.emitter.emit_error(uuid, "Connection closed").await;
            }
            self.requests_in_progress.clear();

            // Only a config change tears the mode down; any other error just waits for a new connection.
            match result
            {
                Ok(()) => {}
                Err(Error::ConfigChanged) => return Err(Error::ConfigChanged),
                Err(e) => eprintln!("Connection error, awaiting new connection: {e}"),
            }
        }
    }

    async fn handle_connection(&mut self)
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
                        Err(e) => return Err(e),
                    }
                    Ok(())
                }
                res = self.command_receiver.recv() => {
                    let command = res?;
                    self.requests_in_progress.push(command.uuid.clone());
                    let data = encode(0, &command)?;
                    self.transport.write(&data).await?;
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
                    let res = from_slice::<CommandResponse>(&payload)?;
                    match res.result {
                        Ok(data) => {
                            if data.last
                            {
                                self.requests_in_progress.retain(|uuid| uuid != &res.uuid);
                            }
                            self.emitter.emit(&res.uuid, data.last, &data.data).await?
                        }
                        Err(err) => {
                            self.requests_in_progress.retain(|uuid| uuid != &res.uuid);
                            self.emitter.emit_error(&res.uuid, &err).await?
                        }
                    }
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
