//! This module implements a minimal and non standard conforming HTTP 1.0
//! round-tripper that works with the bitcoind RPC server. This can be used
//! if minimal dependencies are a goal and synchronous communication is ok.

use std::{fmt, io, net, thread};
use std::io::{BufRead, BufReader, Write};
use std::net::{ToSocketAddrs, TcpStream};
use std::time::{Instant, Duration};

use base64;
use serde;
use serde_json;

use ::client::Transport;

/// The default TCP port to use for connections.
/// Set to 8332, the default RPC port for bitcoind.
pub const DEFAULT_PORT: u16 = 8332;

/// Simple HTTP transport that implements the necessary subset of HTTP for
/// running a bitcoind RPC client.
#[derive(Clone, Debug)]
pub struct SimpleHttpTransport {
    addr: net::SocketAddr,
    url: String,
    timeout: Duration,
    /// The value of the `Authorization` HTTP header.
    basic_auth: Option<String>,
}

impl Default for SimpleHttpTransport {
    fn default() -> Self {
        SimpleHttpTransport {
            addr: net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), DEFAULT_PORT),
            url: format!("http://127.0.0.1:{}/", DEFAULT_PORT).parse().unwrap(),
            timeout: Duration::from_secs(15),
            basic_auth: None,
        }
    }
}

impl SimpleHttpTransport {
    /// Construct a new `SimpleHttpTransport` with default parameters
    pub fn new() -> Self {
        SimpleHttpTransport::default()
    }

    /// Returns a builder for `SimpleHttpTransport`
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// Error that can happen when sending requests
#[derive(Debug)]
pub enum Error {
    /// An invalid URL was passed.
    InvalidUrl(String),
    /// An error occurred on the socket layer
    SocketError(io::Error),
    /// The HTTP header of the response couldn't be parsed
    HttpParseError,
    /// Unexpected HTTP error code (non-200)
    HttpErrorCode(u16),
    /// We didn't receive a complete response till the deadline ran out
    Timeout,
    /// JSON parsing error.
    Json(serde_json::Error),
}

impl ::std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::InvalidUrl(ref u) => write!(f, "invalid URL: {}", u),
            Error::SocketError(ref e) => write!(f, "Couldn't connect to host: {}", e),
            Error::HttpParseError => f.write_str("Couldn't parse response header."),
            Error::HttpErrorCode(c) => write!(f, "unexpected HTTP code: {}", c),
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

/// Try to read a line from a buffered reader. If no line can be read till the deadline is reached
/// return a timeout error.
fn get_line<R: BufRead>(reader: &mut R, deadline: Instant) -> Result<String, Error> {
    let mut line = String::new();
    while deadline > Instant::now() {
        match reader.read_line(&mut line) {
            // EOF reached for now, try again later
            Ok(0) => thread::sleep(Duration::from_millis(5)),
            // received useful data, return it
            Ok(_) => return Ok(line),
            // io error occurred, abort
            Err(e) => return Err(Error::SocketError(e)),
        }
    }
    Err(Error::Timeout)
}

impl Transport for SimpleHttpTransport {
    type Err = Error;

    fn call<R>(&self, req: impl serde::Serialize) -> Result<R, Self::Err>
        where R: for<'a> serde::de::Deserialize<'a>
    {
        // Open connection
        let request_deadline = Instant::now() + self.timeout;
        let mut sock = TcpStream::connect_timeout(&self.addr, self.timeout)?;

        // Send HTTP request
        sock.write_all(format!("POST {} HTTP/1.1\r\n", self.url).as_bytes())?;
        // Write headers
        sock.write_all(b"Content-Type: application/json-rpc\r\n")?;
        if let Some(ref auth) = self.basic_auth {
            sock.write_all(b"Authentication: ")?;
            sock.write_all(auth.as_ref())?;
            sock.write_all(b"\r\n")?;
        }
        // Write body
        sock.write_all(b"\r\n")?;
        serde_json::to_writer(&mut sock, &req)?;
        sock.flush()?;

        // Receive response
        let mut reader = BufReader::new(sock);

        // Parse first HTTP response header line
        let http_response = get_line(&mut reader, request_deadline)?;
        if http_response.len() < 12 || !http_response.starts_with("HTTP/1.1 ") {
            return Err(Error::HttpParseError);
        }
        let response_code = match http_response[9..12].parse::<u16>() {
            Ok(n) => n,
            Err(_) => return Err(Error::HttpParseError),
        };
        if response_code != 200 {
            return Err(Error::HttpErrorCode(response_code));
        }

        // Skip response header fields
        while get_line(&mut reader, request_deadline)? != "\r\n" {}

        // Read and return actual response line
        let resp = get_line(&mut reader, request_deadline)?;
        //NB this could be serde_json::from_reader but then we don't control the timeout
        Ok(serde_json::from_str(&resp)?)
    }
}

/// Builder for simple bitcoind `SimpleHttpTransport`s
#[derive(Clone, Debug)]
pub struct Builder {
    tp: SimpleHttpTransport,
}


impl Builder {
    /// Construct new `Builder` with default configuration
    pub fn new() -> Builder {
        Builder {
            tp: SimpleHttpTransport::new(),
        }
    }

    /// Sets the timeout after which requests will abort if they aren't finished
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.tp.timeout = timeout;
        self
    }

    /// Set the URL of the server to the transport.
    pub fn url<U: Into<String>>(mut self, url: U) -> Result<Self, Error> {
        let url = url.into();

        // Do some very basic manual URL parsing because the uri/url crates
        // all have unicode-normalization as a dependency and that's broken.

        { // scope for borrowck on url
            // this clone is here because of 1.29 borrowck
            // remove this when 1.29 is deprecated
            let url_cloned = url.clone();

            // We need to get the hostname and the port.
            // (1) Split scheme
            let after_scheme = {
                let mut split = url_cloned.splitn(2, "://");
                let s = split.next().unwrap();
                split.next().unwrap_or(s)
            };
            // (2) split off path
            let before_path = after_scheme.splitn(2, "/").next().unwrap();
            // (3) split off auth part
            let after_auth = {
                let mut split = before_path.splitn(2, "@");
                let s = split.next().unwrap();
                split.next().unwrap_or(s)
            };
            // so now we should have <hostname>:<port> or just <hostname>
            let mut split = after_auth.split(":");
            let hostname = split.next().unwrap();
            let port: u16 = match split.next() {
                Some(port_str) => match port_str.parse() {
                    Ok(port) => port,
                    Err(_) => return Err(Error::InvalidUrl(url)),
                },
                None => DEFAULT_PORT,
            };
            // make sure we don't have a second colon in this part
            if split.next().is_some() {
                return Err(Error::InvalidUrl(url));
            }

            self.tp.addr = match format!("{}:{}", hostname, port).to_socket_addrs()?.next() {
                Some(a) => a,
                None => return Err(Error::InvalidUrl(url)),
            };
        }

        self.tp.url = url;
        Ok(self)
    }

    /// Add authentication information to the transport.
    pub fn auth<S: AsRef<str>>(mut self, user: S, pass: Option<S>) -> Self {
        let mut auth = user.as_ref().to_owned();
        auth.push(':');
        if let Some(ref pass) = pass {
            auth.push_str(&pass.as_ref()[..]);
        }
        self.tp.basic_auth = Some(format!("Basic {}", &base64::encode(auth.as_bytes())));
        self
    }

    /// Builds the final `SimpleHttpTransport`
    pub fn build(self) -> SimpleHttpTransport {
        self.tp
    }
}

impl ::Client<SimpleHttpTransport> {
    /// Create a new JSON-RPC client using a bare-minimum HTTP transport.
    pub fn simple_http(
        url: String,
        user: Option<String>,
        pass: Option<String>,
    ) -> Result<::Client<SimpleHttpTransport>, Error> {
        let mut builder = Builder::new().url(url)?;
        if let Some(user) = user {
            builder = builder.auth(user, pass);
        }
        Ok(::Client::with_transport(builder.build()))
    }
}

#[cfg(test)]
mod tests {
    use std::net;

    use ::Client;
    use super::*;

    #[test]
    fn test_urls() {
        let addr: net::SocketAddr = "localhost:22".to_socket_addrs().unwrap().next().unwrap();
        let urls = [
            "localhost:22",
            "http://localhost:22/",
            "https://localhost:22/walletname/stuff?it=working",
            "http://me:weak@localhost:22/wallet",
        ];
        for u in &urls {
            let tp = Builder::new().url(*u).unwrap().build();
            assert_eq!(tp.addr, addr);
        }
    }

    #[test]
    fn construct() {
        let tp = Builder::new()
            .timeout(Duration::from_millis(100))
            .url("localhost:22").unwrap()
            .auth("user", None)
            .build();
        let _ = Client::with_transport(tp);

        let _ = Client::simple_http("localhost:22".to_owned(), None, None).unwrap();
    }
}

