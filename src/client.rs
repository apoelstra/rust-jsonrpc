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

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic;

use serde;
// use base64;
use serde_json;

use super::{Request, Response};
use util::HashableValue;
use error::Error;

/// An interface for a transport over which to use the JSONRPC protocol.
pub trait Transport {
    /// The Error type for this transport.
    /// Errors will get converted into Box<std::error::Error> so the
    /// type here is not use any further.
    type Err: std::error::Error;

    /// Make an RPC call over the transport.
    fn call<R>(&self, impl serde::Serialize) -> Result<R, Self::Err>
        where R: for<'a> serde::de::Deserialize<'a>;
}

/// A JSON-RPC client.
pub struct Client<T: Transport> {
    transport: T,
    nonce: atomic::AtomicUsize,
}

impl<T: Transport + 'static> Client<T> {
    /// Creates a new client with the given transport.
    pub fn with_transport(transport: T) -> Client<T> {
        Client {
            transport: transport,
            nonce: atomic::AtomicUsize::new(1),
        }
    }

    /// Builds a request
    pub fn build_request<'a>(
        &self,
        method: &'a str,
        params: &'a [Box<serde_json::value::RawValue>],
    ) -> Request<'a> {
        let nonce = self.nonce.fetch_add(1, atomic::Ordering::Relaxed);
        Request {
            method: method,
            params: params,
            id: serde_json::Value::from(nonce),
            jsonrpc: Some("2.0"),
        }
    }

    /// Sends a request to a client
    pub fn send_request(&self, request: Request) -> Result<Response, Error> {
        let res: Result<Response, _> = self.transport.call(request);
        res.map_err(|e| Error::Transport(e.into()))
    }

    /// Sends a batch of requests to the client.  The return vector holds the response
    /// for the request at the corresponding index.  If no response was provided, it's [None].
    ///
    /// Note that the requests need to have valid IDs, so it is advised to create the requests
    /// with [build_request].
    pub fn send_batch(&self, requests: &[Request]) -> Result<Vec<Option<Response>>, Error> {
        if requests.is_empty() {
            return Err(Error::EmptyBatch);
        }

        // If the request body is invalid JSON, the response is a single response object.
        // We ignore this case since we are confident we are producing valid JSON.
        let responses: Vec<Response> = self.transport.call(requests)
            .map_err(|e| Error::Transport(e.into()))?;
        if responses.len() > requests.len() {
            return Err(Error::WrongBatchResponseSize);
        }

        //TODO(stevenroose) check if the server preserved order to avoid doing the mapping

        // First index responses by ID and catch duplicate IDs.
        let mut by_id = HashMap::with_capacity(requests.len());
        for resp in responses.into_iter() {
            let id = HashableValue(Cow::Owned(resp.id.clone()));
            if let Some(dup) = by_id.insert(id, resp) {
                return Err(Error::BatchDuplicateResponseId(dup.id));
            }
        }
        // Match responses to the requests.
        let results = requests.into_iter().map(|r| {
            by_id.remove(&HashableValue(Cow::Borrowed(&r.id)))
        }).collect();

        // Since we're also just producing the first duplicate ID, we can also just produce the
        // first incorrect ID in case there are multiple.
        if let Some((id, _)) = by_id.into_iter().nth(0) {
            return Err(Error::WrongBatchResponseId(id.0.into_owned()));
        }

        Ok(results)
    }

    /// Make a request and deserialize the response
    pub fn call<R: for<'a> serde::de::Deserialize<'a>>(
        &self,
        method: &str,
        args: &[Box<serde_json::value::RawValue>],
    ) -> Result<R, Error> {
        let request = self.build_request(method, args);
        let id = request.id.clone();

        let response = self.send_request(request)?;
        if response.jsonrpc != None && response.jsonrpc != Some(From::from("2.0")) {
            return Err(Error::VersionMismatch);
        }
        if response.id != id {
            return Err(Error::NonceMismatch);
        }

        Ok(response.result()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use serde;

    struct DummyTransport;
    impl Transport for DummyTransport {
        type Err = io::Error;

        fn call<R>(&self, _req: impl serde::Serialize) -> Result<R, Self::Err>
            where R: for<'a> serde::de::Deserialize<'a>
        {
            Err(io::Error::new(io::ErrorKind::Other, ""))
        }
    }

    #[test]
    fn sanity() {
        let client = Client::with_transport(DummyTransport);
        assert_eq!(client.nonce.load(std::sync::atomic::Ordering::Relaxed), 1);
        let req1 = client.build_request("test", &[]);
        assert_eq!(client.nonce.load(std::sync::atomic::Ordering::Relaxed), 2);
        let req2 = client.build_request("test", &[]);
        assert_eq!(client.nonce.load(std::sync::atomic::Ordering::Relaxed), 3);
        assert!(req1.id != req2.id);
    }
}
