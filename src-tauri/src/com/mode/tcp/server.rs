use serde_json::from_slice;
use tokio_util::bytes::BytesMut;
use tokio::io::AsyncReadExt;
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
}

impl TcpServerCom
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, config: TcpServerConfig, app: tauri::AppHandle) -> Self
    {
        let emitter = Emitter::Tauri(TauriEmitter::new(app.clone()));
        Self { emitter, config_receiver, command_receiver, config, buffer: BytesMut::with_capacity(4096), app }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        let _window = WindowGuard::new(self.app.clone())?;
        let listener = TcpListener::bind((self.config.host.address.as_str(), self.config.host.port)).await?;

        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = listener.accept() => {
                    let (socket, addr) = result?;
                    println!("Accepted connection from {}", addr);
                    self.buffer.clear();
                    self.handle_connection(socket).await?;
                    Ok(())
                }
                res = self.command_receiver.recv() => {
                    let command = res?;
                    self.emitter.emit_error(&command.uuid, &"Server not connected".to_string()).await?;
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
                res = self.command_receiver.recv() => {
                    let command = res?;
                    let data = encode(0, &command)?;
                    socket.try_write(&data)?;
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
                    let res = from_slice::<CommandResponse>(&payload)?;
                    match res.result {
                        Ok(data) => self.emitter.emit(&res.uuid, data.last, &data.data).await?,
                        Err(err) => self.emitter.emit_error(&res.uuid, &err).await?,
                    }
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
            if let Mode::TcpServer(remote_config) = &config.mode
            {
                if self.config.host != remote_config.host
                {
                    return Err(Error::Other("Configuration changed".to_string()));
                }
            }
            else
            {
                return Err(Error::Other("Configuration changed to non-remote mode".to_string()));
            }
        }
        Ok(())
    }
}
