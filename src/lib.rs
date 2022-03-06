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

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

// Coding conventions
#![warn(missing_docs)]

extern crate serde;
pub extern crate serde_json;
extern crate erased_serde;

#[cfg(feature = "base64-compat")]
pub extern crate base64;

pub mod client;
pub mod error;
pub mod json;
mod util;

#[cfg(feature = "simple_http")]
pub mod simple_http;

#[cfg(feature = "simple_tcp")]
pub mod simple_tcp;

#[cfg(all(feature = "simple_uds", not(windows)))]
pub mod simple_uds;

#[cfg(feature = "tp-hyper")]
pub mod hyper;

// Re-export error type
pub use crate::client::{Client, Request, SyncTransport, AsyncTransport};
pub use crate::error::Error;
pub use crate::json::{RpcError, StandardError};

use serde_json::value::RawValue;

/// Shorthand method to convert an argument into a boxed [`serde_json::value::RawValue`].
///
/// Since serializers rarely fail, it's probably easier to use [`arg`] instead.
pub fn try_arg<T: serde::Serialize>(arg: T) -> Result<Box<RawValue>, serde_json::Error> {
    RawValue::from_string(serde_json::to_string(&arg)?)
}

/// Shorthand method to convert an argument into a boxed [`serde_json::value::RawValue`].
///
/// This conversion should not fail, so to avoid returning a [`Result`],
/// in case of an error, the error is serialized as the return value.
pub fn arg<T: serde::Serialize>(arg: T) -> Box<RawValue> {
    match try_arg(arg) {
        Ok(v) => v,
        Err(e) => RawValue::from_string(format!("<<ERROR SERIALIZING ARGUMENT: {}>>", e))
            .unwrap_or_else(|_| {
                RawValue::from_string("<<ERROR SERIALIZING ARGUMENT>>".to_owned()).unwrap()
            }),
    }
}
