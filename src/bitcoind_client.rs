//! This module implements a minimal and non standard conforming HTTP client that works with
//! the bitcoind RPC server. This client can be used if minimal dependencies are a goal.

use ::HttpRoundTripper;

use http;

use std;
use std::io::{BufRead, BufReader, Cursor, Write};
use std::net::TcpStream;
use std::time::{Instant, Duration};

/// Simple bitcoind JSON RPC client that implements the necessary subset of HTTP
#[derive(Copy, Clone, Debug)]
pub struct Tripper {
    default_port: u16,
    timeout: Duration,
}

/// Builder for simple bitcoind `Tripper`s
#[derive(Clone, Debug)]
pub struct Builder {
    tripper: Tripper,
}

impl Default for Tripper {
    fn default() -> Self {
        Tripper {
            default_port: 8332,
            timeout: Duration::from_secs(15),
        }
    }
}


impl Builder {
    /// Construct new `Builder` with default configuration
    pub fn new() -> Builder {
        Builder {
            tripper: Tripper::new(),
        }
    }

    /// Sets the port that the tripper will connect to in case none was specified in the URL of the
    /// request.
    pub fn default_port(mut self, port: u16) -> Self {
        self.tripper.default_port = port;
        self
    }

    /// Sets the timeout after which requests will abort if they aren't finished
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.tripper.timeout = timeout;
        self
    }

    /// Builds the final `Tripper`
    pub fn build(self) -> Tripper {
        self.tripper
    }
}

impl Tripper {
    /// Construct a new `Tripper` with default parameters
    pub fn new() -> Self {
        Tripper::default()
    }

    /// Returns a builder for `Tripper`
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// Try to read a line from a buffered reader. If no line can be read till the deadline is reached
/// return a timeout error.
fn get_line<R: BufRead>(reader: &mut R, deadline: Instant) -> Result<String, Error> {
    let mut line = String::new();
    while deadline > Instant::now() {
        match reader.read_line(&mut line) {
            // EOF reached for now, try again later
            Ok(0) => std::thread::yield_now(),
            // received useful data, return it
            Ok(_) => return Ok(line),
            // io error occurred, abort
            Err(e) => return Err(Error::SocketError(e)),
        }
    }
    Err(Error::Timeout)
}

impl HttpRoundTripper for Tripper {
    type ResponseBody = Cursor<Vec<u8>>;
    type Err = Error;

    fn post(&self, request: http::Request<&[u8]>) -> Result<http::Response<Self::ResponseBody>, Self::Err> {
        // Parse request
        let server = match request
            .uri()
            .authority_part()
            .map(|authority|{
                (
                    authority.host(),
                    authority.port_part().map(|p| p.as_u16()).unwrap_or(self.default_port)
                )
            }) {
            Some(s) => s,
            None => return Err(Error::NoHost),
        };
        let method = request.method();
        let uri = request.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");

        // Open connection
        let request_deadline = Instant::now() + self.timeout;
        let mut sock = TcpStream::connect(server)?;

        // Send HTTP request
        sock.write_all(format!("{} {} HTTP/1.0\r\n", method, uri).as_bytes())?;
        sock.write_all("Content-Type: application/json\r\n".as_bytes())?;
        sock.write_all(format!("Content-Length: {}\r\n", request.body().len()).as_bytes())?;
        for (key, value) in request.headers() {
            sock.write_all(key.as_ref())?;
            sock.write_all(": ".as_bytes())?;
            sock.write_all(value.as_ref())?;
            sock.write_all("\r\n".as_bytes())?;
        }
        sock.write_all("\r\n".as_bytes())?;
        sock.write_all(request.body())?;

        // Receive response
        let mut reader = BufReader::new(sock);

        // Parse first HTTP response header line
        let http_response = get_line(&mut reader, request_deadline)?;
        if http_response.len() < 12 || !http_response.starts_with("HTTP/1.0 ") {
            return Err(Error::HttpParseError);
        }
        match http_response[9..12].parse::<u16>() {
            Ok(200) => {},
            Ok(e) => return Err(Error::ErrorCode(e)),
            Err(_) => return Err(Error::HttpParseError),
        };

        // Skip response header fields
        while get_line(&mut reader, request_deadline)? != "\r\n" {}

        // Read and return actual response line
        get_line(&mut reader, request_deadline)
            .map(|response| http::Response::new(Cursor::new(response.into_bytes())))
    }
}

/// Error that can happen when sending requests
#[derive(Debug)]
pub enum Error {
    /// The request didn't specify a host to connect to
    NoHost,
    /// An error occurred on the socket layer
    SocketError(std::io::Error),
    /// The HTTP header of the response couldn't be parsed
    HttpParseError,
    /// The server responded with a non-200 HTTP code
    ErrorCode(u16),
    /// We didn't receive a complete response till the deadline ran out
    Timeout,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Error::NoHost => f.write_str("No host was given in the URL."),
            Error::SocketError(ref e) => write!(f, "Couldn't connect to host: {}", e),
            Error::HttpParseError => f.write_str("Couldn't parse response header."),
            Error::ErrorCode(e) => write!(f, "HTTP error {}", e),
            Error::Timeout => f.write_str("Didn't receive response data in time, timed out."),
        }.expect("writing the error message should work");

        Ok(())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::SocketError(e)
    }
}

#[cfg(test)]
mod tests {
    use Client;
    use super::*;

    #[test]
    fn construct() {
        let rtt = Builder::new()
            .timeout(Duration::from_millis(100))
            .build();
        let client = Client::new(rtt, "localhost:22".to_owned(), None, None);
        drop(client);
    }
}

