mod frameio;

use camino::Utf8PathBuf;
pub use frameio::{FramedRead, FramedWrite};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Run { arg0: String, argv: Vec<String> },
    CreateFile { path: Utf8PathBuf },
    WriteFd { fd: i32, data: Vec<u8> },
    CloseFd { fd: i32 },
    Shutdown {},
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Err(String),
    Ok(i32),
}
