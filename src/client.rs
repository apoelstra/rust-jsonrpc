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

use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::sync::{Arc, Mutex};

use hyper;
use hyper::client::Client as HyperClient;
use hyper::header::{Authorization, Basic, ContentType, Headers};
use serde;
use serde_json;

use super::{Request, Response};
use util::HashableValue;
use error::Error;

/// A handle to a remote JSONRPC server
pub struct Client {
    url: String,
    user: Option<String>,
    pass: Option<String>,
    client: HyperClient,
    nonce: Arc<Mutex<u64>>,
}

impl Client {
    /// Creates a new client with a specific HyperClient
    /// Use this to create a socks5 client, etc
    pub fn with_client(url: String, user: Option<String>, pass: Option<String>, client: HyperClient) -> Client {
        // Check that if we have a password, we have a username; other way around is ok
        debug_assert!(pass.is_none() || user.is_some());

        Client {
            url: url,
            user: user,
            pass: pass,
            client: client,
            nonce: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a new client
    pub fn new(url: String, user: Option<String>, pass: Option<String>) -> Client {
        Client::with_client(url, user, pass, HyperClient::new())
    }

    /// Make a request and deserialize the response
    pub fn do_rpc<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        rpc_name: &str,
        args: &[serde_json::value::Value],
    ) -> Result<T, Error> {
        let request = self.build_request(rpc_name, args);
        let response = self.send_request(&request)?;

        Ok(response.into_result()?)
    }

    /// The actual send logic used by both [send_request] and [send_batch].
    fn send_raw<B, R>(&self, body: &B) -> Result<R, Error>
    where
        B: serde::ser::Serialize,
        R: for<'de> serde::de::Deserialize<'de>,
    {
        // Build request
        let request_raw = serde_json::to_vec(body)?;

        // Setup connection
        let mut headers = Headers::new();
        headers.set(ContentType::json());
        if let Some(ref user) = self.user {
            headers.set(Authorization(Basic {
                username: user.clone(),
                password: self.pass.clone(),
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
                if e.kind() == io::ErrorKind::BrokenPipe
                    || e.kind() == io::ErrorKind::ConnectionAborted
                {
                    try!(self
                        .client
                        .post(&self.url)
                        .headers(retry_headers)
                        .body(&request_raw[..])
                        .send()
                        .map_err(Error::Hyper))
                } else {
                    return Err(Error::Hyper(hyper::error::Error::Io(e)));
                }
            }
            Err(e) => {
                return Err(Error::Hyper(e));
            }
        };

        // nb we ignore stream.status since we expect the body
        // to contain information about any error
        let response: R = serde_json::from_reader(&mut stream)?;
        stream.bytes().count(); // Drain the stream so it can be reused
        Ok(response)
    }

    /// Sends a request to a client
    pub fn send_request(&self, request: &Request) -> Result<Response, Error> {
        let response: Response = self.send_raw(&request)?;
        if response.jsonrpc != None && response.jsonrpc != Some(From::from("2.0")) {
            return Err(Error::VersionMismatch);
        }
        if response.id != request.id {
            return Err(Error::NonceMismatch);
        }
        Ok(response)
    }

    /// Sends a batch of requests to the client.  The return vector holds the response
    /// for the request at the corresponding index.  If no response was provided, it's [None].
    ///
    /// Note that the requests need to have valid IDs, so it is advised to create the requests
    /// with [build_request].
    pub fn send_batch(&self, requests: &[Request]) -> Result<Vec<Option<Response>>, Error> {
        if requests.len() < 1 {
            return Err(Error::EmptyBatch);
        }

        // If the request body is invalid JSON, the response is a single response object.
        // We ignore this case since we are confident we are producing valid JSON.
        let responses: Vec<Response> = self.send_raw(&requests)?;
        if responses.len() > requests.len() {
            return Err(Error::WrongBatchResponseSize);
        }

        // To prevent having to clone responses, we first copy all the IDs so we can reference
        // them easily. IDs can only be of JSON type String or Number (or Null), so cloning
        // should be inexpensive and require no allocations as Numbers are more common.
        let ids: Vec<serde_json::Value> = responses.iter().map(|r| r.id.clone()).collect();
        // First index responses by ID and catch duplicate IDs.
        let mut resp_by_id = HashMap::new();
        for (id, resp) in ids.iter().zip(responses.into_iter()) {
            if let Some(dup) = resp_by_id.insert(HashableValue(&id), resp) {
                return Err(Error::BatchDuplicateResponseId(dup.id));
            }
        }
        // Match responses to the requests.
        let results =
            requests.into_iter().map(|r| resp_by_id.remove(&HashableValue(&r.id))).collect();

        // Since we're also just producing the first duplicate ID, we can also just produce the
        // first incorrect ID in case there are multiple.
        if let Some(incorrect) = resp_by_id.into_iter().nth(0) {
            return Err(Error::WrongBatchResponseId(incorrect.1.id));
        }

        Ok(results)
    }

    /// Builds a request
    pub fn build_request<'a, 'b>(
        &self,
        name: &'a str,
        params: &'b [serde_json::Value],
    ) -> Request<'a, 'b> {
        let mut nonce = self.nonce.lock().unwrap();
        *nonce += 1;
        Request {
            method: name,
            params: params,
            id: From::from(*nonce),
            jsonrpc: Some("2.0"),
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
        let req1 = client.build_request("test", &[]);
        assert_eq!(client.last_nonce(), 1);
        let req2 = client.build_request("test", &[]);
        assert_eq!(client.last_nonce(), 2);
        assert!(req1 != req2);
    }
}
