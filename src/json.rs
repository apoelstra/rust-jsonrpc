//! Type definitions for the JSON objects described in the JSONRPC specification.

use std::fmt;

use erased_serde;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::Error;

/// A JSONRPC request object.
#[derive(Serialize)]
pub struct Request<'a> {
    /// The name of the RPC call.
    pub method: &'a str,
    /// Parameters to the RPC call.
    pub params: &'a (dyn erased_serde::Serialize + Sync),
    /// Identifier for this Request, which should appear in the response.
    pub id: serde_json::Value,
    /// jsonrpc field, MUST be "2.0".
    pub jsonrpc: Option<&'a str>,
}

impl<'a> fmt::Debug for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
        //TODO(stevenroose) remove if unneeded
        // let mut ret = Vec::<u8>::new();
        // let mut ser = serde_json::Serializer::new(&mut ret);
        // erased_serde::serialize(&self, &mut ser).unwrap();
        // f.write_str(str::from_utf8(&ret).unwrap())
    }
}

/// A JSONRPC response object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    /// A result if there is one, or [`None`].
    pub result: Option<Box<RawValue>>,
    /// An error if there is one, or [`None`].
    pub error: Option<RpcError>,
    /// Identifier for this Request, which should match that of the request.
    pub id: serde_json::Value,
    /// jsonrpc field, MUST be "2.0".
    pub jsonrpc: Option<String>,
}

impl Response {
    /// Convert response into a raw result.
    pub fn into_raw_result(self) -> Result<Box<RawValue>, Error> {
        if let Some(e) = self.error {
            return Err(Error::Rpc(e));
        }

        if let Some(res) = self.result {
            Ok(res)
        } else {
            Ok(RawValue::from_string(serde_json::to_string(&serde_json::Value::Null).unwrap()).unwrap())
        }
    }

    /// Returns whether or not the `result` field is empty
    pub fn is_none(&self) -> bool {
        self.result.is_none()
    }
}

/// Standard error responses, as described at at
/// <http://www.jsonrpc.org/specification#error_object>
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
#[derive(Debug)]
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
    InternalError,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A JSONRPC error object
pub struct RpcError {
    /// The integer identifier of the error
    pub code: i32,
    /// A string describing the error
    pub message: String,
    /// Additional data specific to the error
    pub data: Option<Box<serde_json::value::RawValue>>,
}

/// Create a standard error responses
pub fn standard_error(
    code: StandardError,
    data: Option<Box<serde_json::value::RawValue>>,
) -> RpcError {
    match code {
        StandardError::ParseError => RpcError {
            code: -32700,
            message: "Parse error".to_string(),
            data,
        },
        StandardError::InvalidRequest => RpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data,
        },
        StandardError::MethodNotFound => RpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data,
        },
        StandardError::InvalidParams => RpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data,
        },
        StandardError::InternalError => RpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data,
        },
    }
}

/// Converts a Rust `Result` to a JSONRPC response object
pub fn result_to_response(
    result: Result<serde_json::Value, RpcError>,
    id: serde_json::Value,
) -> Response {
    match result {
        Ok(data) => Response {
            result: Some(
                serde_json::value::RawValue::from_string(serde_json::to_string(&data).unwrap())
                    .unwrap(),
            ),
            error: None,
            id,
            jsonrpc: Some(String::from("2.0")),
        },
        Err(err) => Response {
            result: None,
            error: Some(err),
            id,
            jsonrpc: Some(String::from("2.0")),
        },
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use serde_json::value::RawValue;

    use super::{Response, result_to_response, standard_error};
    use super::StandardError::{
        InternalError, InvalidParams, InvalidRequest, MethodNotFound, ParseError,
    };

    #[test]
    fn response_is_none() {
        let joanna = Response {
            result: Some(RawValue::from_string(serde_json::to_string(&true).unwrap()).unwrap()),
            error: None,
            id: From::from(81),
            jsonrpc: Some(String::from("2.0")),
        };

        let bill = Response {
            result: None,
            error: None,
            id: From::from(66),
            jsonrpc: Some(String::from("2.0")),
        };

        assert!(!joanna.is_none());
        assert!(bill.is_none());
    }

    #[test]
    fn response_extract() {
        let obj = vec!["Mary", "had", "a", "little", "lamb"];
        let response = Response {
            result: Some(RawValue::from_string(serde_json::to_string(&obj).unwrap()).unwrap()),
            error: None,
            id: serde_json::Value::Null,
            jsonrpc: Some(String::from("2.0")),
        };
        let recovered1: Vec<String> = response.result().unwrap();
        assert!(response.clone().check_error().is_ok());
        let recovered2: Vec<String> = response.result().unwrap();
        assert_eq!(obj, recovered1);
        assert_eq!(obj, recovered2);
    }

    #[test]
    fn null_result() {
        let s = r#"{"result":null,"error":null,"id":"test"}"#;
        let response: Response = serde_json::from_str(s).unwrap();
        let recovered1: Result<(), _> = response.result();
        let recovered2: Result<(), _> = response.result();
        assert!(recovered1.is_ok());
        assert!(recovered2.is_ok());

        let recovered1: Result<String, _> = response.result();
        let recovered2: Result<String, _> = response.result();
        assert!(recovered1.is_err());
        assert!(recovered2.is_err());
    }

    #[test]
    fn batch_response() {
        // from the jsonrpc.org spec example
        let s = r#"[
            {"jsonrpc": "2.0", "result": 7, "id": "1"},
            {"jsonrpc": "2.0", "result": 19, "id": "2"},
            {"jsonrpc": "2.0", "error": {"code": -32600, "message": "Invalid Request"}, "id": null},
            {"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": "5"},
            {"jsonrpc": "2.0", "result": ["hello", 5], "id": "9"}
        ]"#;
        let batch_response: Vec<Response> = serde_json::from_str(s).unwrap();
        assert_eq!(batch_response.len(), 5);
    }

    #[test]
    fn test_arg() {
        macro_rules! test_arg {
            ($val:expr, $t:ty) => {{
                let val1: $t = $val;
                let arg = super::arg(val1.clone());
                let val2: $t = serde_json::from_str(arg.get()).expect(stringify!($val));
                assert_eq!(val1, val2, "failed test for {}", stringify!($val));
            }};
        }

        test_arg!(true, bool);
        test_arg!(42, u8);
        test_arg!(42, usize);
        test_arg!(42, isize);
        test_arg!(vec![42, 35], Vec<u8>);
        test_arg!(String::from("test"), String);

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct Test {
            v: String,
        }
        test_arg!(
            Test {
                v: String::from("test"),
            },
            Test
        );
    }

    #[test]
    fn test_parse_error() {
        let resp = result_to_response(Err(standard_error(ParseError, None)), From::from(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, serde_json::Value::from(1));
        assert_eq!(resp.error.unwrap().code, -32700);
    }

    #[test]
    fn test_invalid_request() {
        let resp = result_to_response(Err(standard_error(InvalidRequest, None)), From::from(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, serde_json::Value::from(1));
        assert_eq!(resp.error.unwrap().code, -32600);
    }

    #[test]
    fn test_method_not_found() {
        let resp = result_to_response(Err(standard_error(MethodNotFound, None)), From::from(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, serde_json::Value::from(1));
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_invalid_params() {
        let resp = result_to_response(Err(standard_error(InvalidParams, None)), From::from("123"));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, serde_json::Value::from("123"));
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[test]
    fn test_internal_error() {
        let resp = result_to_response(Err(standard_error(InternalError, None)), From::from(-1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, serde_json::Value::from(-1));
        assert_eq!(resp.error.unwrap().code, -32603);
    }
}
