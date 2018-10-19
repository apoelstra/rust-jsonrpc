// Rust JSON-RPC Library
// Written in 2015 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Client support
//!
//! Support for connecting to JSONRPC servers over HTTP, sending requests,
//! and parsing responses
//!

use std::io;
use std::io::Read;
use std::sync::{Arc, Mutex};

use hyper::client::Client as HyperClient;
use hyper::header::{Headers, Authorization, Basic};
use hyper;
use serde_json;

use super::{Request, Response};
use error::Error;

/// A handle to a remote JSONRPC server
pub struct Client {
    url: String,
    user: Option<String>,
    pass: Option<String>,
    client: HyperClient,
    nonce: Arc<Mutex<u64>>
}

impl Client {
    /// Creates a new client
    pub fn new(url: String, user: Option<String>, pass: Option<String>) -> Client {
        // Check that if we have a password, we have a username; other way around is ok
        debug_assert!(pass.is_none() || user.is_some());

        Client {
            url: url,
            user: user,
            pass: pass,
            client: HyperClient::new(),
            nonce: Arc::new(Mutex::new(0))
        }
    }

    /// Sends a request to a client
    pub fn send_request(&self, request: &Request) -> Result<Response, Error> {
        // Build request
        let request_raw = serde_json::to_vec(request)?;

        // Setup connection
        let mut headers = Headers::new();
        if let Some(ref user) = self.user {
            headers.set(Authorization(Basic {
                username: user.clone(),
                password: self.pass.clone()
            }));
        }

        // Send request
        let retry_headers = headers.clone();
        let hyper_request = self.client.post(&self.url).headers(headers).body(&request_raw[..]);
        let mut stream = match hyper_request.send() {
            Ok(s) => s,
            // Hyper maintains a pool of TCP connections to its various clients,
            // and when one drops it cannot tell until it tries sending. In this
            // case the appropriate thing is to re-send, which will cause hyper
            // to open a new connection. Jonathan Reem explained this to me on
            // IRC, citing vague technical reasons that the library itself cannot
            // do the retry transparently.
            Err(hyper::error::Error::Io(e)) => {
                if e.kind() == io::ErrorKind::BrokenPipe ||
                   e.kind() == io::ErrorKind::ConnectionAborted {
                    try!(self.client.post(&self.url).headers(retry_headers)
                                                    .body(&request_raw[..])
                                                    .send().map_err(Error::Hyper))
                } else {
                    return Err(Error::Hyper(hyper::error::Error::Io(e)));
                }
            }
            Err(e) => { return Err(Error::Hyper(e)); }
        };

        // nb we ignore stream.status since we expect the body
        // to contain information about any error
        let response: Response = serde_json::from_reader(&mut stream)?;
        stream.bytes().count();  // Drain the stream so it can be reused
        if response.jsonrpc != None &&
           response.jsonrpc != Some(From::from("2.0")) {
            return Err(Error::VersionMismatch);
        }
        if response.id != request.id {
            return Err(Error::NonceMismatch);
        }
        Ok(response)
    }

    /// Builds a request
    pub fn build_request(&self, name: String, params: Vec<serde_json::Value>) -> Request {
        let mut nonce = self.nonce.lock().unwrap();
        *nonce += 1;
        Request {
            method: name,
            params: params,
            id: From::from(*nonce),
            jsonrpc: Some(String::from("2.0"))
        }
    }

    /// Accessor for the last-used nonce
    pub fn last_nonce(&self) -> u64 {
        *self.nonce.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let client = Client::new("localhost".to_owned(), None, None);
        assert_eq!(client.last_nonce(), 0);
        let req1 = client.build_request("test".to_owned(), vec![]);
        assert_eq!(client.last_nonce(), 1);
        let req2 = client.build_request("test".to_owned(), vec![]);
        assert_eq!(client.last_nonce(), 2);
        assert!(req1 != req2);
    }
}

