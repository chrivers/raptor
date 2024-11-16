use std::io::{Read, Write};

use serde::{de::DeserializeOwned, Serialize};

use crate::RaptorResult;

pub trait FramedRead: Read {
    fn read_framed<T: DeserializeOwned>(&mut self) -> RaptorResult<T> {
        let mut len_bytes: [u8; 4] = [0; 4];
        self.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes);

        let mut req = vec![0; len as usize];
        self.read_exact(&mut req)?;

        Ok(bincode::deserialize(&req)?)
    }
}

pub trait FramedWrite: Write {
    #[allow(clippy::cast_possible_truncation)]
    fn write_framed(&mut self, value: impl Serialize) -> RaptorResult<()> {
        let buf = bincode::serialize(&value)?;
        let len_bytes = (buf.len() as u32).to_be_bytes();
        self.write_all(&len_bytes)?;
        self.write_all(&buf)?;

        Ok(())
    }
}

impl<R: Read> FramedRead for R {}
impl<W: Write> FramedWrite for W {}
