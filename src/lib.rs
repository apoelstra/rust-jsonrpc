// Rust JSON-RPC Library
// Written in 2015 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
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

//! # Rust JSON-RPC Library
//!
//! Rust support for the JSON-RPC 2.0 protocol.
//!

#![crate_type = "lib"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![crate_name = "jsonrpc"]

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

extern crate hyper;
extern crate serde;
extern crate serde_json as json;

#[macro_use] mod macros;
pub mod client;
pub mod error;

// Re-export error type
pub use error::Error;

#[derive(Clone, Debug, PartialEq)]
/// A JSONRPC request object
pub struct Request {
    /// The name of the RPC call
    pub method: String,
    /// Parameters to the RPC call
    pub params: Vec<json::Value>,
    /// Identifier for this Request, which should appear in the response
    pub id: json::Value
}

#[derive(Clone, Debug, PartialEq)]
/// A JSONRPC response object
pub struct Response {
    /// A result if there is one, or null
    pub result: Option<json::Value>,
    /// An error if there is one, or null
    pub error: Option<error::RpcError>,
    /// Identifier from the request
    pub id: json::Value
}

impl Response {
    /// Extract the result from a response
    pub fn result<T: serde::Deserialize>(&self) -> Result<T, Error> {
        if let Some(ref e) = self.error {
            return Err(Error::Rpc(e.clone()));
        }
        match self.result {
            Some(ref res) => json::from_value(res.clone()).map_err(Error::Json),
            None => Err(Error::NoErrorOrResult)
        }
    }

    /// Extract the result from a response, consuming the response
    pub fn into_result<T: serde::Deserialize>(self) -> Result<T, Error> {
        if let Some(e) = self.error {
            return Err(Error::Rpc(e));
        }
        match self.result {
            Some(res) => json::from_value(res).map_err(Error::Json),
            None => Err(Error::NoErrorOrResult)
        }
    }
}

serde_struct_serialize!(
    Request,
    RequestMapVisitor,
    method => 0,
    params => 1,
    id => 2
);

serde_struct_deserialize!(
    Request,
    RequestVisitor,
    RequestField,
    RequestFieldVisitor,
    method => Method,
    params => Params,
    id => Id
);

serde_struct_serialize!(
    Response,
    ResponseMapVisitor,
    result => 0,
    error => 1,
    id => 2
);

serde_struct_deserialize!(
    Response,
    ResponseVisitor,
    ResponseField,
    ResponseFieldVisitor,
    result => Result,
    error => Error,
    id => Id
);

#[cfg(test)]
mod tests {
    use super::{Request, Response};
    use super::error::RpcError;
    use json;
    use json::value::Value as JsonValue;

    #[test]
    fn request_serialize_round_trip() {
        let original = Request {
            method: "test".to_owned(),
            params: vec![JsonValue::Null,
                         JsonValue::Bool(false),
                         JsonValue::Bool(true),
                         JsonValue::String("test2".to_owned())],
            id: JsonValue::U64(69)
        };

        let ser = json::to_string(&original).unwrap();
        let des = json::from_str(&ser).unwrap();

        assert_eq!(original, des);
    }

    #[test]
    fn response_serialize_round_trip() {
        let original_err = RpcError {
            code: -77,
            message: "test4".to_string(),
            data: Some(JsonValue::Bool(true))
        };

        let original = Response {
            result: Some(JsonValue::Array(vec![JsonValue::Null,
                                               JsonValue::Bool(false),
                                               JsonValue::Bool(true),
                                               JsonValue::String("test2".to_owned())])),
            error: Some(original_err),
            id: JsonValue::U64(101)
        };

        let ser = json::to_string(&original).unwrap();
        let des = json::from_str(&ser).unwrap();

        assert_eq!(original, des);
    }

    #[test]
    fn response_extract() {
        let obj = vec!["Mary", "had", "a", "little", "lamb"];
        let response = Response {
            result: Some(json::to_value(&obj)),
            error: None,
            id: JsonValue::Null
        };
        let recovered1: Vec<String> = response.result().unwrap();
        let recovered2: Vec<String> = response.into_result().unwrap();
        assert_eq!(obj, recovered1);
        assert_eq!(obj, recovered2);
    }
}

