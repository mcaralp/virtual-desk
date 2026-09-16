use serde::Serialize;
use postcard::to_allocvec;
use tokio_util::bytes::BytesMut;
use crate::com::Error;

const HEADER_SIZE: usize = 8;

pub fn encode<T: Serialize>(cmd: u32, value: &T)
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

/// Pulls the next complete framed message out of `buffer`, if any, returning its command id and payload.
pub fn decode_frame(buffer: &mut BytesMut)
    -> Option<(u32, BytesMut)>
{
    if buffer.len() < HEADER_SIZE
    {
        return None;
    }

    let cmd = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let len = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;

    if buffer.len() < HEADER_SIZE + len
    {
        return None;
    }

    let mut frame = buffer.split_to(HEADER_SIZE + len);
    let payload = frame.split_off(HEADER_SIZE);
    Some((cmd, payload))
}
