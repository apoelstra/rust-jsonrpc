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

use std::sync::{Arc, Mutex};

use serde_json::{self, Value};

use reqwest::{self, Body, Method, Url};
use reqwest::header::{Authorization, Basic};

use error::Error;
use {Request, Response};

/// A handle to a remote JSONRPC server
pub struct Client {
    url: Url,
    user: Option<String>,
    pass: Option<String>,
    client: reqwest::Client,
    nonce: Arc<Mutex<u64>>
}

impl Client {
    /// Creates a new client
    pub fn new<U, P>(url: &str, user: U, pass: P) -> Result<Client, Error>
        where U: Into<Option<String>>,
              P: Into<Option<String>>,
    {
        let (user, pass) = (user.into(), pass.into());

        // Check that if we have a password, we have a username; other way around is ok
        debug_assert!(pass.is_none() || user.is_some());

        Ok(Client {
            url: Url::parse(url)?,
            user,
            pass,
            client: reqwest::Client::new(),
            nonce: Arc::new(Mutex::new(0))
        })
    }

    /// Sends a request to a client
    pub fn execute(&self, request: Request) -> Result<Response, Error> {
        let mut reqwest_request = reqwest::Request::new(Method::Post, self.url.clone());

        // Setup connection
        if let Some(ref user) = self.user {
            let headers = reqwest_request.headers_mut();
            headers.set(Authorization(Basic {
                username: user.clone(),
                password: self.pass.clone()
            }));
        }

        {
            let body = reqwest_request.body_mut();
            *body = Some(Body::from(serde_json::to_vec(&request)?));
        }

        let response: Response = self.client.execute(reqwest_request)?.json()?;

        if let Some(ref jsonrpc) = response.jsonrpc {
            if &**jsonrpc != "2.0" {
                return Err(Error::VersionMismatch);
            }
        }
        if response.id != request.id {
            return Err(Error::NonceMismatch);
        }

        Ok(response)
    }

    /// Builds a request
    pub fn build_request<N>(&self, name: N, params: Vec<Value>) -> Request
        where N: ToString,
    {
        let mut nonce = self.nonce.lock().unwrap();
        *nonce += 1;
        Request {
            method: name.to_string(),
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
        let client = Client::new("http://localhost", None, None).unwrap();
        assert_eq!(client.last_nonce(), 0);
        let req1 = client.build_request("test", vec![]);
        assert_eq!(client.last_nonce(), 1);
        let req2 = client.build_request("test", vec![]);
        assert_eq!(client.last_nonce(), 2);
        assert!(req1 != req2);
    }
}

