use serde_json::from_slice;
use tokio_util::bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use crate::config::{ConfigReceiver, Mode, TcpClientConfig};
use crate::com::{CommandRequest, CommandResponse, Error, Emitter, MpscEmitter};
use crate::com::commands::CommandDispatch;
use super::util::{encode, decode_frame};

pub struct TcpClientCom
{
    dispatch: CommandDispatch,
    config_receiver: ConfigReceiver,
    response_receiver: tokio::sync::mpsc::Receiver<CommandResponse>,
    config: TcpClientConfig,
    buffer: BytesMut,
}

impl TcpClientCom
{
    pub fn new(config_receiver: ConfigReceiver, config: TcpClientConfig) -> Self
    {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let emitter = Emitter::Mpsc(MpscEmitter::new(&tx));
        let dispatch = CommandDispatch::new(emitter, config_receiver.clone());
        TcpClientCom { dispatch, config_receiver, response_receiver: rx, config, buffer: BytesMut::with_capacity(4096) }
    }

    fn send_command(&self, command: CommandRequest)
        -> Result<(), Error>
    {
        let dispatch = self.dispatch.clone();
        tauri::async_runtime::spawn(async move
        {
            if let Err(e) = dispatch.send_command(command).await
            {
                eprintln!("Failed to send tcp client command: {e}");
            }
        });
        Ok(())
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = TcpStream::connect((self.config.host.address.as_str(), self.config.host.port)) => {
                    let socket = result?;
                    println!("Connected to {}:{}", self.config.host.address, self.config.host.port);
                    self.buffer.clear();
                    self.handle_connection(socket).await?;
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

    async fn handle_connection(&mut self, mut socket: TcpStream)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = socket.read_buf(&mut self.buffer) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            self.handle_incoming_data().await?;
                        }
                        Err(e) => return Err(Error::IoError(e)),
                    }
                    Ok(())
                }
                res = self.response_receiver.recv() => {
                    if let Some(response) = res
                    {
                        let data = encode(0, &response)?;
                        socket.try_write(&data)?;
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
        Ok(())
    }

    async fn handle_incoming_data(&mut self)
        -> Result<(), Error>
    {
        while let Some((cmd, payload)) = decode_frame(&mut self.buffer)
        {
            match cmd
            {
                0 => {
                    let command = from_slice::<CommandRequest>(&payload)?;
                    self.send_command(command)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_config(&self) -> Result<(), Error>
    {
        let result = self.config_receiver.read_config();
        if let Ok(config) = result
        {
            if let Mode::TcpClient(remote_config) = &config.mode
            {
                if self.config.host != remote_config.host
                {
                    return Err(Error::Other("Configuration changed".to_string()));
                }
            }
            else
            {
                return Err(Error::Other("Configuration changed to non-tcp-client mode".to_string()));
            }
        }
        Ok(())
    }
}
