// Rust JSON-RPC Library
// Written in 2014 by
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

//! # Error handling
//!
//! Some useful methods for creating Error objects
//!

use serialize::json;

use {JsonResult, Response};

/// Standard error responses, as described at at
/// http://www.jsonrpc.org/specification#error_object
///
/// # Documentation Copyright
/// Copyright (C) 2007-2010 by the JSON-RPC Working Group
/// 
/// This document and translations of it may be used to implement JSON-RPC, it may be copied and furnished to others, and derivative works that comment on or otherwise explain it or assist in its implementation may be prepared, copied, published and distributed, in whole or in part, without restriction of any kind, provided that the above copyright notice and this paragraph are included on all such copies and derivative works. However, this document itself may not bemodified in any way.
/// 
/// The limited permissions granted above are perpetual and will not be revoked.
/// 
/// This document and the information contained herein is provided "AS IS" and ALL WARRANTIES, EXPRESS OR IMPLIED are DISCLAIMED, INCLUDING BUT NOT LIMITED TO ANY WARRANTY THAT THE USE OF THE INFORMATION HEREIN WILL NOT INFRINGE ANY RIGHTS OR ANY IMPLIED WARRANTIES OF MERCHANTABILITY OR FITNESS FOR A PARTICULAR PURPOSE.
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

#[deriving(Clone, Show, Encodable)]
/// A JSONRPC error object
pub struct Error {
  code: int,
  message: String,
  data: Option<json::Json>
}

/// Create a standard error responses
pub fn standard_error(code: StandardError, data: Option<json::Json>) -> Error {
  match code {
    ParseError => Error {
      code: -32700,
      message: "Parse error".to_string(),
      data: data
    },
    InvalidRequest => Error {
      code: -32600,
      message: "Invalid Request".to_string(),
      data: data
    },
    MethodNotFound => Error {
      code: -32601,
      message: "Method not found".to_string(),
      data: data
    },
    InvalidParams => Error {
      code: -32602,
      message: "Invalid params".to_string(),
      data: data
    },
    InternalError => Error {
      code: -32603,
      message: "Internal error".to_string(),
      data: data
    },
  }
}

/// Converts a Rust `Result` to a JSONRPC response object
pub fn result_to_response(result: JsonResult<json::Json>, id: json::Json) -> Response {
  match result {
    Ok(data) => Response { result: Some(data), error: None, id: id },
    Err(err) => Response { result: None, error: Some(err), id: id }
  }
}

