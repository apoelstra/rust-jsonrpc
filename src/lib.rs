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

//! # Rust JSON-RPC Library
//!
//! This library supports the JSONRPC protocol on top of Tcp. In future
//! other transports should be supported.
//!

#![crate_name = "jsonrpc"]
#![crate_type = "dylib"]
#![crate_type = "rlib"]

// Experimental features we need
#![feature(globs)]
#![feature(macro_rules)]
#![feature(overloaded_calls)]
#![feature(unsafe_destructor)]
#![feature(default_type_params)]

#![comment = "Rust Bitcoin Library"]
#![license = "CC0"]

// Coding conventions
#![deny(non_uppercase_pattern_statics)]
#![deny(uppercase_variables)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case_functions)]
#![deny(unused_mut)]
#![warn(missing_doc)]

extern crate http;
extern crate serialize;
extern crate time;

use serialize::json;

pub mod error;
pub mod server;

#[deriving(Clone, Show)]
/// A JSONRPC request object
pub struct Request {
  /// The name of the RPC call
  pub method: String,
  /// Parameters to the RPC call
  pub params: json::List,
  /// Identifier for this Request, which should appear in the response
  pub id: json::Json
}

#[deriving(Clone, Show, Encodable)]
/// A JSONRPC response object
pub struct Response {
  /// A result if there is one, or null
  pub result: Option<json::Json>,
  /// An error if there is one, or null
  pub error: Option<error::Error>,
  /// Identifier from the request
  pub id: json::Json
}

pub type JsonResult<T> = Result<T, error::Error>;

fn json_decode_field(js: &json::Json, key: &str) -> JsonResult<json::Json> {
  if !js.is_object() {
    Err(error::standard_error(error::InvalidRequest, None))
  } else {
    match js.find(&key.to_string()) {
      Some(js) => Ok(js.clone()),
      None => Err(error::standard_error(error::InvalidRequest, None))
    }
  }
}

fn json_decode_field_string(js: &json::Json, key: &str) -> JsonResult<String> {
  if !js.is_object() {
    Err(error::standard_error(error::InvalidRequest, None))
  } else {
    match js.find(&key.to_string()) {
      Some(js) => {
        if !js.is_string() {
          Err(error::standard_error(error::InvalidRequest, None))
        } else {
          Ok(js.as_string().unwrap().to_string())
        }
      }
      None => Err(error::standard_error(error::InvalidRequest, None))
    }
  }
}

fn json_decode_field_list(js: &json::Json, key: &str) -> JsonResult<json::List> {
  if !js.is_object() {
    Err(error::standard_error(error::InvalidRequest, None))
  } else {
    match js.find(&key.to_string()) {
      Some(js) => {
        if !js.is_list() {
          Err(error::standard_error(error::InvalidRequest, None))
        } else {
          Ok(js.as_list().unwrap().clone())
        }
      }
      None => Err(error::standard_error(error::InvalidRequest, None))
    }
  }
}

