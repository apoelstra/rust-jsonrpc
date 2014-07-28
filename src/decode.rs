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

//! # Decoding helpers
//!
//! Decodes specific objects as specific types, returning a generic
//! "Invalid request" JsonResult
//!

use serialize::json;

use error;
use JsonResult;

/// Read a string
pub fn json_decode_string(js: &json::Json) -> JsonResult<String> {
  if !js.is_string() {
    Err(error::standard_error(error::InvalidRequest, None))
  } else {
    Ok(js.as_string().unwrap().to_string())
  }
}

/// Read a list
pub fn json_decode_list(js: &json::Json) -> JsonResult<json::List> {
  if !js.is_list() {
    Err(error::standard_error(error::InvalidRequest, None))
  } else {
    Ok(js.as_list().unwrap().clone())
  }
}

/// Read a field of an object
pub fn json_decode_field(js: &json::Json, key: &str) -> JsonResult<json::Json> {
  if !js.is_object() {
    Err(error::standard_error(error::InvalidRequest, None))
  } else {
    match js.find(&key.to_string()) {
      Some(js) => Ok(js.clone()),
      None => Err(error::standard_error(error::InvalidRequest, None))
    }
  }
}

/// Read a field of an object as a string
pub fn json_decode_field_string(js: &json::Json, key: &str) -> JsonResult<String> {
  json_decode_field(js, key).and_then(|ref js| json_decode_string(js))
}

/// Read a field of an object as a list
pub fn json_decode_field_list(js: &json::Json, key: &str) -> JsonResult<json::List> {
  json_decode_field(js, key).and_then(|ref js| json_decode_list(js))
}

