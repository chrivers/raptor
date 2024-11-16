mod frameio;

pub use frameio::{FramedRead, FramedWrite};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Run { arg0: String, argv: Vec<String> },
    Shutdown {},
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Err(String),
    Ok(i32),
}
