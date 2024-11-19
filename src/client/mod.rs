mod frameio;

use camino::Utf8PathBuf;
pub use frameio::{FramedRead, FramedWrite};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Account {
    Id(u32),
    Name(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestRun {
    pub arg0: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestCreateFile {
    pub path: Utf8PathBuf,
    pub user: Option<Account>,
    pub group: Option<Account>,
    pub mode: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestWriteFd {
    pub fd: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestCloseFd {
    pub fd: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Run(RequestRun),
    CreateFile(RequestCreateFile),
    WriteFd(RequestWriteFd),
    CloseFd(RequestCloseFd),
    Shutdown {},
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Err(String),
    Ok(i32),
}
