use std::io::{Read, Write};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::FalconResult;

pub trait FramedRead: Read {
    fn read_framed<T: DeserializeOwned>(&mut self) -> FalconResult<T> {
        let mut len_bytes: [u8; 4] = [0; 4];
        self.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes);

        let mut req = vec![0; len as usize];
        self.read_exact(&mut req)?;

        Ok(bincode::serde::decode_from_slice(&req, bincode::config::standard())?.0)
    }
}

pub trait FramedWrite: Write {
    #[allow(clippy::cast_possible_truncation)]
    fn write_framed(&mut self, value: impl Serialize) -> FalconResult<()> {
        let buf = bincode::serde::encode_to_vec(&value, bincode::config::standard())?;
        let len_bytes = (buf.len() as u32).to_be_bytes();
        self.write_all(&len_bytes)?;
        self.write_all(&buf)?;

        Ok(())
    }
}

impl<R: Read> FramedRead for R {}
impl<W: Write> FramedWrite for W {}
