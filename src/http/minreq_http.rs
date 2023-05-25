//! This module implements the [`crate::client::Transport`] trait using [`minreq`]
//! as the underlying HTTP transport.
//!
//! [minreq]: <https://github.com/neonmoe/minreq>

use std::time::Duration;
use std::{error, fmt};

use crate::client::Transport;
use crate::{Request, Response};

const DEFAULT_URL: &str = "http://localhost";
const DEFAULT_PORT: u16 = 8332; // the default RPC port for bitcoind.
const DEFAULT_TIMEOUT_SECONDS: u64 = 15;

/// An HTTP transport that uses [`minreq`] and is useful for running a bitcoind RPC client.
#[derive(Clone, Debug)]
pub struct MinreqHttpTransport {
    /// URL of the RPC server.
    url: String,
    /// timeout only supports second granularity.
    timeout: Duration,
    /// The value of the `Authorization` HTTP header, i.e., a base64 encoding of 'user:password'.
    basic_auth: Option<String>,
}

impl Default for MinreqHttpTransport {
    fn default() -> Self {
        MinreqHttpTransport {
            url: format!("{}:{}", DEFAULT_URL, DEFAULT_PORT),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            basic_auth: None,
        }
    }
}

impl MinreqHttpTransport {
    /// Constructs a new [`MinreqHttpTransport`] with default parameters.
    pub fn new() -> Self {
        MinreqHttpTransport::default()
    }

    /// Returns a builder for [`MinreqHttpTransport`].
    pub fn builder() -> Builder {
        Builder::new()
    }

    fn request<R>(&self, req: impl serde::Serialize) -> Result<R, Error>
    where
        R: for<'a> serde::de::Deserialize<'a>,
    {
        let req = match &self.basic_auth {
            Some(auth) => minreq::Request::new(minreq::Method::Post, &self.url)
                .with_timeout(self.timeout.as_secs())
                .with_header("Authorization", auth)
                .with_json(&req)?,
            None => minreq::Request::new(minreq::Method::Post, &self.url)
                .with_timeout(self.timeout.as_secs())
                .with_json(&req)?,
        };

        let resp = req.send()?;
        let json = resp.json()?;
        Ok(json)
    }
}

impl Transport for MinreqHttpTransport {
    fn send_request(&self, req: Request) -> Result<Response, crate::Error> {
        Ok(self.request(req)?)
    }

    fn send_batch(&self, reqs: &[Request]) -> Result<Vec<Response>, crate::Error> {
        Ok(self.request(reqs)?)
    }

    fn fmt_target(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// Builder for simple bitcoind [`MinreqHttpTransport`].
#[derive(Clone, Debug)]
pub struct Builder {
    tp: MinreqHttpTransport,
}

impl Builder {
    /// Constructs a new [`Builder`] with default configuration and the URL to use.
    pub fn new() -> Builder {
        Builder {
            tp: MinreqHttpTransport::new(),
        }
    }

    /// Sets the timeout after which requests will abort if they aren't finished.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.tp.timeout = timeout;
        self
    }

    /// Sets the URL of the server to the transport.
    pub fn url(mut self, url: &str) -> Result<Self, Error> {
        self.tp.url = url.to_owned();
        Ok(self)
    }

    /// Adds authentication information to the transport.
    pub fn basic_auth(mut self, user: String, pass: Option<String>) -> Self {
        let mut s = user;
        s.push(':');
        if let Some(ref pass) = pass {
            s.push_str(pass.as_ref());
        }
        self.tp.basic_auth = Some(format!("Basic {}", &base64::encode(s.as_bytes())));
        self
    }

    /// Adds authentication information to the transport using a cookie string ('user:pass').
    ///
    /// Does no checking on the format of the cookie string, just base64 encodes whatever is passed in.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use jsonrpc::minreq_http::MinreqHttpTransport;
    /// # use std::fs::{self, File};
    /// # use std::path::Path;
    /// # let cookie_file = Path::new("~/.bitcoind/.cookie");
    /// let mut file = File::open(cookie_file).expect("couldn't open cookie file");
    /// let mut cookie = String::new();
    /// fs::read_to_string(&mut cookie).expect("couldn't read cookie file");
    /// let client = MinreqHttpTransport::builder().cookie_auth(cookie);
    /// ```
    pub fn cookie_auth<S: AsRef<str>>(mut self, cookie: S) -> Self {
        self.tp.basic_auth = Some(format!("Basic {}", &base64::encode(cookie.as_ref().as_bytes())));
        self
    }

    /// Builds the final [`MinreqHttpTransport`].
    pub fn build(self) -> MinreqHttpTransport {
        self.tp
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

/// Error that can happen when sending requests.
#[derive(Debug)]
pub enum Error {
    /// JSON parsing error.
    Json(serde_json::Error),
    /// Minreq error.
    Minreq(minreq::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Json(ref e) => write!(f, "parsing JSON failed: {}", e),
            Error::Minreq(ref e) => write!(f, "minreq: {}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use self::Error::*;

        match *self {
            Json(ref e) => Some(e),
            Minreq(ref e) => Some(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<minreq::Error> for Error {
    fn from(e: minreq::Error) -> Self {
        Error::Minreq(e)
    }
}

impl From<Error> for crate::Error {
    fn from(e: Error) -> crate::Error {
        match e {
            Error::Json(e) => crate::Error::Json(e),
            e => crate::Error::Transport(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Client;

    #[test]
    fn construct() {
        let tp = Builder::new()
            .timeout(Duration::from_millis(100))
            .url("http://localhost:22")
            .unwrap()
            .basic_auth("user".to_string(), None)
            .build();
        let _ = Client::with_transport(tp);
    }
}
