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

use hyper::client::Client as HyperClient;
use hyper::header::{Headers, Authorization, Basic};
use json;
use json::value::Value as JsonValue;

use super::{Request, Response};
use error::Error;

/// A handle to a remote JSONRPC server
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Client {
    url: String,
    user: Option<String>,
    pass: Option<String>,
    nonce: u64
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
            nonce: 0
        }
    }

    /// Sends a request to a client
    pub fn send_request(&self, request: &Request) -> Result<Response, Error> {
        // Build request
        let request = json::to_string(&request).unwrap();

        // Setup connection
        let mut headers = Headers::new();
        if let Some(ref user) = self.user {
            headers.set(Authorization(Basic {
                username: user.clone(),
                password: self.pass.clone()
            }));
        }

        // Send request
        let client = HyperClient::new();
        let request = client.post(&self.url).headers(headers).body(&request);
        let stream = try!(request.send().map_err(Error::Hyper));
        json::de::from_reader(stream).map_err(Error::Json)

        // TODO check nonces match
    }

    /// Builds a request
    pub fn build_request(&mut self, name: String, params: Vec<JsonValue>) -> Request {
        self.nonce += 1;
        Request {
            method: name,
            params: params,
            id: JsonValue::U64(self.nonce)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;

    #[test]
    fn sanity() {
        let mut client = Client::new("localhost".to_owned(), None, None);
        let req1 = client.build_request("test".to_owned(), vec![]);
        let req2 = client.build_request("test".to_owned(), vec![]);
        assert!(req1 != req2);
    }
}

