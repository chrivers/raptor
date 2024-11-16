use std::io::ErrorKind;
use std::os::unix::net::UnixStream;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::Command;

use log::{debug, error, info, trace};

use raptor::client::{FramedRead, FramedWrite, Request, Response};
use raptor::{RaptorError, RaptorResult};

fn main() -> RaptorResult<()> {
    colog::init();
    /* let socket_name = std::env::var("RAPTOR_NSPAWN_SOCKET")?; */
    let socket_name = std::env::args().nth(1).unwrap();
    let mut stream = UnixStream::connect(socket_name)?;

    loop {
        let req: Request = match stream.read_framed() {
            Ok(req) => req,
            Err(RaptorError::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(err) => {
                error!("Failed to read request: {err}");
                break;
            }
        };

        trace!("read request: {req:?}");

        match req {
            Request::Run { arg0, argv } => {
                info!("Exec {} {:?}", arg0, &argv);
                let res = Command::new(&argv[0])
                    .arg0(&argv[0])
                    .args(&argv[1..])
                    .status();

                let resp = match res {
                    Ok(code) => Response::Ok(code.into_raw()),
                    Err(err) => {
                        error!("Error: {err}");
                        Response::Err(err.to_string())
                    }
                };

                debug!("writing response: {resp:?}");
                stream.write_framed(resp)?;
            }

            Request::Shutdown {} => {
                break;
            }
        }
    }

    Ok(())
}
