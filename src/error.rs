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

//! # Error handling
//!
//! Some useful methods for creating Error objects
//!

use hyper;
use json;
use json::value::Value as JsonValue;
use serde;

use Response;

/// A library error
#[derive(Debug)]
pub enum Error {
    /// Json error
    Json(json::error::Error),
    /// Client error
    Hyper(hyper::error::Error),
    /// Error response
    Rpc(RpcError),
    /// Response has neither error nor result
    NoErrorOrResult
}

/// Standard error responses, as described at at
/// http://www.jsonrpc.org/specification#error_object
///
/// # Documentation Copyright
/// Copyright (C) 2007-2010 by the JSON-RPC Working Group
/// 
/// This document and translations of it may be used to implement JSON-RPC, it
/// may be copied and furnished to others, and derivative works that comment
/// on or otherwise explain it or assist in its implementation may be prepared,
/// copied, published and distributed, in whole or in part, without restriction
/// of any kind, provided that the above copyright notice and this paragraph
/// are included on all such copies and derivative works. However, this document
/// itself may not be modified in any way.
/// 
/// The limited permissions granted above are perpetual and will not be revoked.
/// 
/// This document and the information contained herein is provided "AS IS" and
/// ALL WARRANTIES, EXPRESS OR IMPLIED are DISCLAIMED, INCLUDING BUT NOT LIMITED
/// TO ANY WARRANTY THAT THE USE OF THE INFORMATION HEREIN WILL NOT INFRINGE ANY
/// RIGHTS OR ANY IMPLIED WARRANTIES OF MERCHANTABILITY OR FITNESS FOR A
/// PARTICULAR PURPOSE.
/// 
pub enum StandardError {
    /// Invalid JSON was received by the server.
    /// An error occurred on the server while parsing the JSON text.
    ParseError,
    /// The JSON sent is not a valid Request object.
    InvalidRequest,
    /// The method does not exist / is not available.
    MethodNotFound,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSON-RPC error.
    InternalError
}

#[derive(Clone, Debug, PartialEq)]
/// A JSONRPC error object
pub struct RpcError {
    /// The integer identifier of the error
    pub code: i32,
    /// A string describing the error
    pub message: String,
    /// Additional data specific to the error
    pub data: Option<JsonValue>
}

/// Create a standard error responses
pub fn standard_error(code: StandardError, data: Option<JsonValue>) -> RpcError {
    match code {
        StandardError::ParseError => RpcError {
            code: -32700,
            message: "Parse error".to_string(),
            data: data
        },
        StandardError::InvalidRequest => RpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: data
        },
        StandardError::MethodNotFound => RpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: data
        },
        StandardError::InvalidParams => RpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: data
        },
        StandardError::InternalError => RpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: data
        },
    }
}

/// Converts a Rust `Result` to a JSONRPC response object
pub fn result_to_response(result: Result<JsonValue, RpcError>, id: JsonValue) -> Response {
    match result {
        Ok(data) => Response { result: Some(data), error: None, id: id },
        Err(err) => Response { result: None, error: Some(err), id: id }
    }
}

serde_struct_serialize!(
    RpcError,
    RpcErrorMapVisitor,
    code => 0,
    message => 1,
    data => 2
);

serde_struct_deserialize!(
    RpcError,
    RpcErrorVisitor,
    RpcErrorField,
    RpcErrorFieldVisitor,
    code => Code,
    message => Message,
    data => Data
);


#[cfg(test)]
mod tests {
    use super::StandardError::{ParseError, InvalidRequest, MethodNotFound, InvalidParams, InternalError};
    use super::{standard_error, result_to_response};

    use json::value::Value as JsonValue;

    #[test]
    fn test_parse_error() {
        let resp = result_to_response(Err(standard_error(ParseError, None)), JsonValue::U64(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, JsonValue::U64(1));
        assert_eq!(resp.error.unwrap().code, -32700);
    }

    #[test]
    fn test_invalid_request() {
        let resp = result_to_response(Err(standard_error(InvalidRequest, None)), JsonValue::I64(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, JsonValue::I64(1));
        assert_eq!(resp.error.unwrap().code, -32600);
    }

    #[test]
    fn test_method_not_found() {
        let resp = result_to_response(Err(standard_error(MethodNotFound, None)), JsonValue::U64(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, JsonValue::U64(1));
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_invalid_params() {
        let resp = result_to_response(Err(standard_error(InvalidParams, None)), JsonValue::String("123".to_string()));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, JsonValue::String("123".to_string()));
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[test]
    fn test_internal_error() {
        let resp = result_to_response(Err(standard_error(InternalError, None)), JsonValue::I64(-1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, JsonValue::I64(-1));
        assert_eq!(resp.error.unwrap().code, -32603);
    }
}

