use serde::Serialize;
use postcard::{to_allocvec, from_bytes};
use tokio_util::bytes::BytesMut;
use tokio::net::{TcpListener, TcpStream};
use crate::com::{Error, CommandResponse, Emitter, TauriEmitter, WindowGuard};
use crate::config::{ConfigReceiver, Mode, TcpServerConfig};
use crate::command::CommandReceiver;

#[derive(Clone)]
pub struct TcpServerCom
{
    emitter: Emitter,
    config_receiver: ConfigReceiver,
    command_receiver: CommandReceiver,
    config: TcpServerConfig,
    buffer: BytesMut,
}

impl TcpServerCom
{
    pub fn new(config_receiver: ConfigReceiver, command_receiver: CommandReceiver, config: TcpServerConfig) -> Self
    {
        let emitter = Emitter::Tauri(TauriEmitter::new(command_receiver.app()));
        Self { emitter, config_receiver, command_receiver, config, buffer: BytesMut::with_capacity(4096) }
    }

    pub async fn run(&mut self)
        -> Result<(), Error>
    {
        self.command_receiver.register_state(self.clone());

        let _window = WindowGuard::new(self.command_receiver.app())?;

        let listener = TcpListener::bind(("0.0.0.0", self.config.port)).await?;

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

    async fn handle_connection(&mut self, socket: TcpStream)
        -> Result<(), Error>
    {
        loop
        {
            let result: Result<(), Error> = tokio::select! {
                result = socket.readable() => {
                    result?;
                    match socket.try_read_buf(&mut self.buffer) {
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
        loop
        {
            const HEADER_SIZE: usize = 8;
            if self.buffer.len() < HEADER_SIZE
            {
                break;
            }

            let cmd = u32::from_le_bytes([self.buffer[0], self.buffer[1], self.buffer[2], self.buffer[3]]);
            let len = u32::from_le_bytes([self.buffer[4], self.buffer[5], self.buffer[6], self.buffer[7]]) as usize;

            if self.buffer.len() < HEADER_SIZE + len
            {
                break;
            }

            let frame = self.buffer.split_to(HEADER_SIZE + len);
            let payload =  &frame[HEADER_SIZE..];

            match cmd
            {
                0 => {
                    let res = from_bytes::<CommandResponse>(payload)?;
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
                if self.config != *remote_config
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

fn encode<T: Serialize>(cmd: u32, value: &T)
    -> Result<Vec<u8>, Error>
{
    let payload = to_allocvec(value)?;
    let size = u32::try_from(payload.len())?;
    let mut frame = Vec::with_capacity(4 + 4 + payload.len());

    frame.extend_from_slice(&cmd.to_le_bytes());
    frame.extend_from_slice(&size.to_le_bytes());
    frame.extend_from_slice(&payload);

    Ok(frame)
}
