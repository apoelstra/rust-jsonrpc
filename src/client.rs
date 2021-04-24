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
use std::fmt;
use std::sync::atomic;

use serde;
use serde_json;
use serde_json::value::RawValue;

use super::{Request, Response};
use error::Error;
use util::HashableValue;

/// An interface for a transport over which to use the JSONRPC protocol.
pub trait Transport: Send + Sync + 'static {
    /// Send an RPC request over the transport.
    fn send_request(&self, Request) -> Result<Response, Error>;
    /// Send a batch of RPC requests over the transport.
    fn send_batch(&self, &[Request]) -> Result<Vec<Response>, Error>;
    /// Format the target of this transport.
    /// I.e. the URL/socket/...
    fn fmt_target(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

/// A JSON-RPC client.
///
/// Create a new Client using one of the transport-specific constructors:
/// - [Client::simple_http] for the built-in bare-minimum HTTP transport
pub struct Client {
    pub(crate) transport: Box<dyn Transport>,
    nonce: atomic::AtomicUsize,
}

impl Client {
    /// Creates a new client with the given transport.
    pub fn with_transport<T: Transport>(transport: T) -> Client {
        Client {
            transport: Box::new(transport),
            nonce: atomic::AtomicUsize::new(1),
        }
    }

    /// Builds a request.
    ///
    /// To construct the arguments, one can use one of the shorthand methods
    /// [jsonrpc::arg] or [jsonrpc::try_arg].
    pub fn build_request<'a>(&self, method: &'a str, params: &'a [Box<RawValue>]) -> Request<'a> {
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
        self.transport.send_request(request)
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
        let responses = self.transport.send_batch(requests)?;
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
        let results = requests
            .into_iter()
            .map(|r| by_id.remove(&HashableValue(Cow::Borrowed(&r.id))))
            .collect();

        // Since we're also just producing the first duplicate ID, we can also just produce the
        // first incorrect ID in case there are multiple.
        if let Some((id, _)) = by_id.into_iter().nth(0) {
            return Err(Error::WrongBatchResponseId(id.0.into_owned()));
        }

        Ok(results)
    }

    /// Make a request and deserialize the response.
    ///
    /// To construct the arguments, one can use one of the shorthand methods
    /// [jsonrpc::arg] or [jsonrpc::try_arg].
    pub fn call<R: for<'a> serde::de::Deserialize<'a>>(
        &self,
        method: &str,
        args: &[Box<RawValue>],
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

impl fmt::Debug for ::Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "jsonrpc::Client(")?;
        self.transport.fmt_target(f)?;
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync;

    struct DummyTransport;
    impl Transport for DummyTransport {
        fn send_request(&self, _: Request) -> Result<Response, Error> {
            Err(Error::NonceMismatch)
        }
        fn send_batch(&self, _: &[Request]) -> Result<Vec<Response>, Error> {
            Ok(vec![])
        }
        fn fmt_target(&self, _: &mut fmt::Formatter) -> fmt::Result {
            Ok(())
        }
    }

    #[test]
    fn sanity() {
        let client = Client::with_transport(DummyTransport);
        assert_eq!(client.nonce.load(sync::atomic::Ordering::Relaxed), 1);
        let req1 = client.build_request("test", &[]);
        assert_eq!(client.nonce.load(sync::atomic::Ordering::Relaxed), 2);
        let req2 = client.build_request("test", &[]);
        assert_eq!(client.nonce.load(sync::atomic::Ordering::Relaxed), 3);
        assert!(req1.id != req2.id);
    }
}
