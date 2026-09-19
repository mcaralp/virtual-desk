use serde::Serialize;
use serde_json::to_vec;
use tokio_util::bytes::BytesMut;
use crate::com::Error;

const HEADER_SIZE: usize = 8;
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

pub fn encode<T: Serialize>(cmd: u32, value: &T)
    -> Result<Vec<u8>, Error>
{
    let payload = to_vec(value)?;
    let size = u32::try_from(payload.len())?;
    let mut frame = Vec::with_capacity(4 + 4 + payload.len());

    frame.extend_from_slice(&cmd.to_le_bytes());
    frame.extend_from_slice(&size.to_le_bytes());
    frame.extend_from_slice(&payload);

    Ok(frame)
}

/// Pulls the next complete framed message out of `buffer`, if any, returning its command id and payload.
pub fn decode_frame(buffer: &mut BytesMut)
    -> Result<Option<(u32, BytesMut)>, Error>
{
    if buffer.len() < HEADER_SIZE
    {
        return Ok(None);
    }

    let cmd = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
    let len = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]) as usize;

    if len > MAX_FRAME_SIZE
    {
        return Err(Error::Other(format!("Frame too large: {len} bytes (max {MAX_FRAME_SIZE})")));
    }

    if buffer.len() < HEADER_SIZE + len
    {
        return Ok(None);
    }

    let mut frame = buffer.split_to(HEADER_SIZE + len);
    let payload = frame.split_off(HEADER_SIZE);
    Ok(Some((cmd, payload)))
}
