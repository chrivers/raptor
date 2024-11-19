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
pub enum Request {
    Run {
        arg0: String,
        argv: Vec<String>,
    },
    CreateFile {
        path: Utf8PathBuf,
        user: Option<Account>,
        group: Option<Account>,
        mode: Option<u16>,
    },
    WriteFd {
        fd: i32,
        data: Vec<u8>,
    },
    CloseFd {
        fd: i32,
    },
    Shutdown {},
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Err(String),
    Ok(i32),
}
