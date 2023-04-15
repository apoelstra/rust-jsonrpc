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

use std::fmt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::atomic;

use async_trait::async_trait;
use serde;
use serde_json::Value;
use serde_json::value::RawValue;

use crate::json;
use crate::error::Error;

/// Error type of converter methods.
pub type ConverterError = Box<dyn StdError + Send + Sync>;

/// Conversion method that parses the JSON into a serde object.
pub fn convert_parse<T>(raw: Box<RawValue>) -> Result<T, ConverterError>
where
    T: for<'a> serde::Deserialize<'a>,
{
    Ok(serde_json::from_str(raw.get())?)
}

/// Trivial conversion method that actually doesn't do a conversion and keeps the [RawValue].
pub fn convert_raw(raw: Box<RawValue>) -> Result<Box<RawValue>, ConverterError> {
    Ok(raw)
}

/// An interface for a transport over which to use the JSONRPC protocol.
pub trait SyncTransport {
    /// Send an RPC request over the transport.
    fn send_request(&self, request: &json::Request) -> Result<json::Response, Error>;

    /// Send a batch of RPC requests over the transport.
    fn send_batch(&self, requests: &[json::Request]) -> Result<Vec<json::Response>, Error>;
}

/// NB It is advised to also (usually trivially) implement SyncTransport
/// for AsyncTransports by blocking on the future.
#[async_trait]
pub trait AsyncTransport {
    /// Send an RPC request over the transport.
    async fn send_request(
        &self,
        request: &json::Request<'_>,
    ) -> Result<json::Response, Error>;

    /// Send a batch of RPC requests over the transport.
    async fn send_batch(
        &self,
        requests: &[json::Request<'_>],
    ) -> Result<Vec<json::Response>, Error>;
}

/// A single parameter used in [Params].
pub enum Param<'a> {
    /// A [serde_json::Value] parameter.
    Value(Value),
    /// A serializable object by reference.
    ByRef(&'a (dyn erased_serde::Serialize + Sync)),
    /// A boxed serializable object.
    InBox(Box<dyn erased_serde::Serialize + Sync>),
    /// A boxed [serde_json::value::RawValue].
    Raw(Box<RawValue>),
}

impl<'a> serde::Serialize for Param<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Param::Value(ref v) => serde::Serialize::serialize(v, serializer),
            Param::ByRef(r) => serde::Serialize::serialize(r, serializer),
            Param::InBox(b) => serde::Serialize::serialize(b, serializer),
            Param::Raw(r) => serde::Serialize::serialize(r, serializer),
        }
    }
}

impl From<Value> for Param<'static> {
    fn from(v: Value) -> Param<'static> {
        Param::Value(v)
    }
}

impl<'a, T: serde::Serialize + Sync> From<&'a T> for Param<'a> {
    fn from(v: &'a T) -> Param<'a> {
        Param::ByRef(v)
    }
}

impl<T: serde::Serialize + Sync + 'static> From<Box<T>> for Param<'static> {
    fn from(v: Box<T>) -> Param<'static> {
        Param::InBox(v)
    }
}

/// A list that can be either borrowed or owned.
///
/// NB This enum is non-exhaustive and should not be matched over.
pub enum List<'a, T> {
    /// A borrowed list in the form of a slice.
    Slice(&'a [T]),
    /// An owned list in the form of a boxed slice.
    Boxed(Box<[T]>),
    //TODO(stevenroose) add smallvec type or a N-size array maybe support different-N as features
}

impl<'a, T> List<'a, T> {
    /// Represent the list as a slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            List::Slice(s) => s,
            List::Boxed(v) => &v[..],
        }
    }
}

/// Parameters passed into a RPC request.
pub enum Params<'a> {
    /// Positional arguments.
    ByPosition(List<'a, Param<'a>>),
    /// Named arguments.
    ByName(List<'a, (&'a str, Param<'a>)>),
}

impl<'a> serde::Serialize for Params<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Params::ByPosition(params) => params.as_slice().serialize(serializer),
            Params::ByName(params) => {
                let params = params.as_slice();
                let mut map = serializer.serialize_map(Some(params.len()))?;
                for (key, value) in params.iter() {
                    serde::ser::SerializeMap::serialize_entry(&mut map, key, value)?;
                }
                serde::ser::SerializeMap::end(map)
            },
        }
    }
}

impl<'a> From<&'a [Param<'a>]> for Params<'a> {
    fn from(p: &'a [Param<'a>]) -> Params<'a> {
        Params::ByPosition(List::Slice(p))
    }
}

impl<'a> From<Box<[Param<'a>]>> for Params<'a> {
    fn from(p: Box<[Param<'a>]>) -> Params<'a> {
        Params::ByPosition(List::Boxed(p))
    }
}

impl<'a> From<Vec<Param<'a>>> for Params<'a> {
    fn from(p: Vec<Param<'a>>) -> Params<'a> {
        p.into_boxed_slice().into()
    }
}

impl<'a> From<&'a [(&'static str, Param<'a>)]> for Params<'a> {
    fn from(p: &'a [(&'static str, Param<'a>)]) -> Params<'a> {
        Params::ByName(List::Slice(p))
    }
}

impl<'a> From<Box<[(&'static str, Param<'a>)]>> for Params<'a> {
    fn from(p: Box<[(&'static str, Param<'a>)]>) -> Params<'a> {
        Params::ByName(List::Boxed(p))
    }
}

impl<'a> From<Vec<(&'static str, Param<'a>)>> for Params<'a> {
    fn from(p: Vec<(&'static str, Param<'a>)>) -> Params<'a> {
        p.into_boxed_slice().into()
    }
}

impl<'a> From<HashMap<&'static str, Param<'a>>> for Params<'a> {
    fn from(p: HashMap<&'static str, Param<'a>>) -> Params<'a> {
        Params::ByName(List::Boxed(p.into_iter().collect()))
    }
}

/// A prepared RPC request ready to be made using a JSON-RPC client.
pub struct Request<'r, R: 'static> {
    /// The RPC call method name.
    pub method: Cow<'r, str>,
    /// The parameters for the RPC call..
    pub params: Params<'r>,
    /// A converter function to convert the resulting JSON response
    /// into the desired response type.
    pub converter: &'r dyn Fn(Box<RawValue>) -> Result<R, ConverterError>,
}

impl<'r, R> Request<'r, R> {
    /// Validate the raw response object.
    fn validate_response(nonce: &Value, response: &json::Response) -> Result<(), Error> {
        if response.jsonrpc != None && response.jsonrpc != Some(From::from("2.0")) {
            return Err(Error::VersionMismatch);
        }
        if response.id != *nonce {
            return Err(Error::NonceMismatch);
        }
        Ok(())
    }

    /// Batch this request
    pub fn batch(self, batch: &mut Batch<'r, R>) -> Result<(), Request<'r, R>> {
        batch.insert_request(self)
    }

    /// Execute this request by blocking.
    pub fn get_sync<T: SyncTransport>(self, client: &Client<T>) -> Result<R, Error> {
        let req = client.create_raw_request_object(&self.method, &self.params);
        let res = SyncTransport::send_request(&client.transport, &req)?;
        Self::validate_response(&req.id, &res)?;
        (self.converter)(res.into_raw_result()?).map_err(Error::ResponseConversion)
    }

    /// Execute this request asynchronously.
    pub async fn get_async<T: AsyncTransport>(self, client: &Client<T>) -> Result<R, Error> {
        let req = client.create_raw_request_object(&self.method, &self.params);
        let res = AsyncTransport::send_request(&client.transport, &req).await?;
        Self::validate_response(&req.id, &res)?;
        (self.converter)(res.into_raw_result()?).map_err(Error::ResponseConversion)
    }
}

/// A batch of multiple JSON-RPC requests.
pub struct Batch<'b, R: 'static> {
    method: Option<Cow<'b, str>>,
    converter: Option<&'b dyn Fn(Box<RawValue>) -> Result<R, ConverterError>>,
    /// List of arguments for the requests.
    batch_args: Vec<Params<'b>>,
}

impl<'b, R> Batch<'b, R> {
    /// Inserts the request into the batch if it is compatible.
    /// If not, it returns the request in the Err variant.
    pub fn insert_request(&mut self, req: Request<'b, R>) -> Result<(), Request<'b, R>> {
        if let Some(method) = self.method.as_ref() {
            if method.as_ref() != req.method.as_ref() || !std::ptr::eq(self.converter.unwrap(), req.converter) {
                return Err(req);
            }
        } else {
            self.method = Some(req.method);
            self.converter = Some(req.converter);
        }
        
        self.batch_args.push(req.params);
        Ok(())
    }
}

/// A JSON-RPC client.
///
/// Create a new Client using one of the transport-specific constructors:
/// - [Client::simple_http] for the built-in bare-minimum HTTP transport
pub struct Client<T> {
    transport: T,
    nonce: atomic::AtomicUsize,
}

impl<T> Client<T> {
    /// Create a new [Client] using the given transport.
    pub fn new(transport: T) -> Client<T> {
        Client {
            transport: transport,
            nonce: atomic::AtomicUsize::new(1),
        }
    }

    /// Creates a raw request object.
    ///
    /// To construct the arguments, one can use one of the shorthand methods
    /// [jsonrpc::arg] or [jsonrpc::try_arg].
    pub fn create_raw_request_object<'a>(
        &self,
        method: &'a str,
        params: &'a Params<'a>,
    ) -> json::Request<'a> {
        let nonce = self.nonce.fetch_add(1, atomic::Ordering::Relaxed);
        json::Request {
            method: method,
            params: params,
            id: Value::from(nonce),
            jsonrpc: Some("2.0"),
        }
    }

    pub fn prepare<'r, R>(
        &self,
        method: impl Into<Cow<'r, str>>,
        params: impl Into<Params<'r>>,
        converter: &'r dyn Fn(Box<RawValue>) -> Result<R, ConverterError>,
    ) -> Request<'r, R> {
        Request {
            method: method.into(),
            params: params.into(),
            converter: converter,
        }
    }

    pub fn prepare_raw<'r>(
        &self,
        method: impl Into<Cow<'r, str>>,
        params: impl Into<Params<'r>>,
    ) -> Request<'r, Box<RawValue>> {
        Request {
            method: method.into(),
            params: params.into(),
            converter: &convert_raw,
        }
    }

    pub fn prepare_parse<'r, R: for<'a> serde::de::Deserialize<'a>>(
        &self,
        method: impl Into<Cow<'r, str>>,
        params: impl Into<Params<'r>>,
    ) -> Request<'r, R> {
        Request {
            method: method.into(),
            params: params.into(),
            converter: &convert_parse,
        }
    }

    ///// Sends a batch of requests to the client.  The return vector holds the response
    ///// for the request at the corresponding index.  If no response was provided, it's [None].
    /////
    ///// Note that the requests need to have valid IDs, so it is advised to create the requests
    ///// with [build_request].
    //pub fn send_batch(&self, requests: &[json::Request]) -> Result<Vec<Option<Response>>, Error> {
    //    if requests.is_empty() {
    //        return Err(Error::EmptyBatch);
    //    }

    //    // If the request body is invalid JSON, the response is a single response object.
    //    // We ignore this case since we are confident we are producing valid JSON.
    //    let responses = self.transport.send_batch(requests)?;
    //    if responses.len() > requests.len() {
    //        return Err(Error::WrongBatchResponseSize);
    //    }

    //    //TODO(stevenroose) check if the server preserved order to avoid doing the mapping

    //    // First index responses by ID and catch duplicate IDs.
    //    let mut by_id = HashMap::with_capacity(requests.len());
    //    for resp in responses.into_iter() {
    //        let id = HashableValue(Cow::Owned(resp.id.clone()));
    //        if let Some(dup) = by_id.insert(id, resp) {
    //            return Err(Error::BatchDuplicateResponseId(dup.id));
    //        }
    //    }
    //    // Match responses to the requests.
    //    let results = requests.into_iter().map(|r| {
    //        by_id.remove(&HashableValue(Cow::Borrowed(&r.id)))
    //    }).collect();

    //    // Since we're also just producing the first duplicate ID, we can also just produce the
    //    // first incorrect ID in case there are multiple.
    //    if let Some((id, _)) = by_id.into_iter().nth(0) {
    //        return Err(Error::WrongBatchResponseId(id.0.into_owned()));
    //    }

    //    Ok(results)
    //}
}

impl <T: SyncTransport> Client<T> {
    /// Make a request and deserialize the response.
    ///
    /// To construct the arguments, one can use one of the shorthand methods
    /// [jsonrpc::arg] or [jsonrpc::try_arg].
    pub fn call_sync<'s, R: 'static + for<'a> serde::de::Deserialize<'a>>(
        &'s self,
        method: &'static str,
        params: Vec<Param<'s>>,
    ) -> Result<R, Error> {
        self.prepare_parse(method, params).get_sync(self)
    }
}

impl<T: fmt::Debug> fmt::Debug for Client<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "jsonrpc::Client(nonce: {}; transport: {:?})",
            self.nonce.load(atomic::Ordering::Relaxed), self.transport,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync;
    use json;

    #[derive(Debug)]
    struct DummyTransport;
    impl SyncTransport for DummyTransport {
        fn send_request(&self, _: json::Request) -> Result<json::Response, Error> { Err(Error::NonceMismatch) }
        fn send_batch(&self, _: &[json::Request]) -> Result<Vec<json::Response>, Error> { Ok(vec![]) }
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
