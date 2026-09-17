use serde_json::from_slice;
use tokio_util::bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::com::{Error, CommandResponse, Emitter, TauriEmitter, WindowGuard};
use crate::config::{ConfigReceiver, Mode, TcpServerConfig};
use crate::command::CommandReceiver;
use super::util::{encode, decode_frame};

pub struct TcpServerCom
{
    emitter: Emitter,
    config_receiver: ConfigReceiver,
    command_receiver: CommandReceiver,
    config: TcpServerConfig,
    buffer: BytesMut,
    app: tauri::AppHandle,
    requests_in_progress: Vec<String>,
}

impl TcpServerCom
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, config: TcpServerConfig, app: tauri::AppHandle) -> Self
    {
        let emitter = Emitter::Tauri(TauriEmitter::new(app.clone()));
        Self { emitter, config_receiver, command_receiver, config, buffer: BytesMut::with_capacity(4096), app, requests_in_progress: Vec::new() }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let window = WindowGuard::new(self.app.clone())?;
        let res = self.accept_connections().await;
        window.stop().await?;

        res
    }

    async fn accept_connections(&mut self)
        -> Result<(), Error>
    {
        let listener = TcpListener::bind((self.config.host.address.as_str(), self.config.host.port)).await?;
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            println!("Accepted connection from {}", addr);
                            self.buffer.clear();
                            self.handle_connection(socket).await
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
                res = self.command_receiver.recv() => {
                    let command = res?;
                    self.requests_in_progress.push(command.uuid.clone());
                    let data = encode(0, &command)?;
                    socket.write_all(&data).await?;
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
            if let Mode::TcpServer(remote_config) = &config.mode
            {
                if self.config.host != remote_config.host
                {
                    return Err(Error::ConfigChanged);
                }
            }
            else
            {
                return Err(Error::ConfigChanged);
            }
        }
        Ok(())
    }
}
