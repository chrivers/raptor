use std::os::unix::net::UnixListener;

use falcon::client::{FramedRead, FramedWrite, Request, RequestRun, Response};
use falcon::error::FalconResult;
use log::{error, info};

fn main() -> FalconResult<()> {
    colog::init();
    let socket_name = std::env::var("FALCON_SOCKET").expect("Must have FALCON_SOCKET set");
    let listen = UnixListener::bind(socket_name)?;

    let (mut stream, _addr) = listen.accept()?;

    let req = Request::Run(RequestRun {
        arg0: String::from("sh"),
        argv: ["/bin/sh", "-c", "id"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect(),
    });

    info!("writing frame: {req:?}");
    stream.write_framed(req)?;

    let resp: Response = stream.read_framed()?;

    match resp {
        Err(err) => error!("Error: {err}"),
        Ok(res) => info!("Success: {res}"),
    }

    Ok(())
}
