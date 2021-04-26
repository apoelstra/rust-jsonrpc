//! This module implements a synchronous transport over a raw TcpListener.

#[cfg(not(windows))]
use std::os::unix::net::UnixStream;
#[cfg(windows)]
use uds_windows::UnixStream;

use std::{fmt, io, path, time};

use serde;
use serde_json;

use client::Transport;
use {Request, Response};

/// Error that can occur while using the UDS transport.
#[derive(Debug)]
pub enum Error {
    /// An error occurred on the socket layer
    SocketError(io::Error),
    /// We didn't receive a complete response till the deadline ran out
    Timeout,
    /// JSON parsing error.
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::SocketError(ref e) => write!(f, "Couldn't connect to host: {}", e),
            Error::Timeout => f.write_str("Didn't receive response data in time, timed out."),
            Error::Json(ref e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::SocketError(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<Error> for ::Error {
    fn from(e: Error) -> ::Error {
        match e {
            Error::Json(e) => ::Error::Json(e),
            e => ::Error::Transport(Box::new(e)),
        }
    }
}

impl ::std::error::Error for Error {}

/// Simple synchronous UDS transport.
#[derive(Debug, Clone)]
pub struct UdsTransport {
    /// The path to the Unix Domain Socket
    pub sockpath: path::PathBuf,
    /// The read and write timeout to use
    pub timeout: Option<time::Duration>,
}

impl UdsTransport {
    /// Create a new UdsTransport without timeouts to use
    pub fn new<P: AsRef<path::Path>>(sockpath: P) -> UdsTransport {
        UdsTransport {
            sockpath: sockpath.as_ref().to_path_buf(),
            timeout: None,
        }
    }

    fn request<R>(&self, req: impl serde::Serialize) -> Result<R, Error>
    where
        R: for<'a> serde::de::Deserialize<'a>,
    {
        let mut sock = UnixStream::connect(&self.sockpath)?;
        sock.set_read_timeout(self.timeout)?;
        sock.set_write_timeout(self.timeout)?;

        serde_json::to_writer(&mut sock, &req)?;

        // NOTE: we don't check the id there, so it *must* be synchronous
        let resp: R = serde_json::Deserializer::from_reader(&mut sock)
            .into_iter()
            .next()
            .ok_or(Error::Timeout)??;
        Ok(resp)
    }
}

impl Transport for UdsTransport {
    fn send_request(&self, req: Request) -> Result<Response, ::Error> {
        Ok(self.request(req)?)
    }

    fn send_batch(&self, reqs: &[Request]) -> Result<Vec<Response>, ::Error> {
        Ok(self.request(reqs)?)
    }

    fn fmt_target(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.sockpath.to_string_lossy())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(windows))]
    use std::os::unix::net::UnixListener;
    #[cfg(windows)]
    use uds_windows::UnixListener;

    use std::{
        fs,
        io::{Read, Write},
        thread,
    };

    use super::*;
    use Client;

    // Test a dummy request / response over an UDS
    #[test]
    fn sanity_check_uds_transport() {
        let socket_path: path::PathBuf = "uds_scratch.socket".into();
        // Any leftover?
        fs::remove_file(&socket_path).unwrap_or_else(|_| ());

        let server = UnixListener::bind(&socket_path).unwrap();
        let dummy_req = Request {
            method: "getinfo".into(),
            params: serde_json::Value::Array(vec![]),
            id: serde_json::Value::Number(111.into()),
            jsonrpc: Some("2.0".into()),
        };
        let dummy_req_ser = serde_json::to_vec(&dummy_req).unwrap();
        let dummy_resp = Response {
            result: None,
            error: None,
            id: serde_json::Value::Number(111.into()),
            jsonrpc: Some("2.0".into()),
        };
        let dummy_resp_ser = serde_json::to_vec(&dummy_resp).unwrap();

        let cli_socket_path = socket_path.clone();
        let client_thread = thread::spawn(move || {
            let transport = UdsTransport::new(cli_socket_path);
            let client = Client::with_transport(transport);

            loop {
                match client.send_request(dummy_req.clone()) {
                    Ok(resp) => return resp,
                    Err(e) => {
                        // Print so should it fail it can be debugged with --nocapture
                        eprintln!("Error while sending request: '{:?}'", e);
                        thread::sleep(time::Duration::from_millis(100));
                    }
                }
            }
        });

        let (mut stream, _) = server.accept().unwrap();
        stream.set_read_timeout(Some(time::Duration::from_secs(5))).unwrap();
        let mut recv_req = vec![0; dummy_req_ser.len()];
        let mut read = 0;
        while read < dummy_req_ser.len() {
            read += stream.read(&mut recv_req[read..]).unwrap();
        }
        assert_eq!(recv_req, dummy_req_ser);

        stream.write(&dummy_resp_ser).unwrap();
        stream.flush().unwrap();
        let recv_resp = client_thread.join().unwrap();
        assert_eq!(serde_json::to_vec(&recv_resp).unwrap(), dummy_resp_ser);

        // Clean up
        drop(server);
        fs::remove_file(&socket_path).unwrap();
    }
}
