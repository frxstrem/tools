use std::io;
use std::marker::PhantomData;

use bytes::{BufMut, BytesMut};
use bytes::{ByteOrder, LittleEndian};
use serde::{de::DeserializeOwned, Serialize};
use tokio::codec::{Decoder, Encoder};

pub struct JsonCodec<T> {
    _phantom: PhantomData<T>,
}

impl<T> JsonCodec<T> {
    pub fn new() -> JsonCodec<T> {
        JsonCodec {
            _phantom: PhantomData,
        }
    }
}

impl<T> Decoder for JsonCodec<T>
where
    T: DeserializeOwned,
{
    type Item = T;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<T>> {
        if buf.len() < 4 {
            return Ok(None);
        }

        let length = LittleEndian::read_u32(&buf[0..4]) as usize;
        if buf.len() < 4 + length {
            return Ok(None);
        }

        buf.advance(4);
        let buf = buf.split_to(length);

        let message: T = serde_json::from_slice(&buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        Ok(Some(message))
    }
}

impl<T> Encoder for JsonCodec<T>
where
    T: Serialize,
{
    type Item = T;
    type Error = io::Error;

    fn encode(&mut self, item: T, buf: &mut BytesMut) -> io::Result<()> {
        let data = serde_json::to_vec(&item)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        if data.len() > std::u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Message too large",
            ));
        }

        buf.put_u32_le(data.len() as u32);
        buf.put(data);

        Ok(())
    }
}
