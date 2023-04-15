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

use std::fmt;

use serde_json;

use crate::json::RpcError;

/// A library error
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The transport used doesn't support the sync/async method.
    NoTransportSupport,
    /// A transport error
    Transport(Box<dyn std::error::Error + Send + Sync>),
    /// Json error
    Json(serde_json::Error),
    /// Error response
    Rpc(RpcError),
    /// Response to a request did not have the expected nonce
    NonceMismatch,
    /// Response to a request had a jsonrpc field other than "2.0"
    VersionMismatch,
    /// Batches can't be empty
    EmptyBatch,
    /// Too many responses returned in batch
    WrongBatchResponseSize,
    /// Batch response contained a duplicate ID
    BatchDuplicateResponseId(serde_json::Value),
    /// Batch response contained an ID that didn't correspond to any request ID
    WrongBatchResponseId(serde_json::Value),
    /// Error occurred in converting the response value into the return type.
    ResponseConversion(Box<dyn std::error::Error + Send + Sync>),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Json(e)
    }
}

impl From<RpcError> for Error {
    fn from(e: RpcError) -> Error {
        Error::Rpc(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoTransportSupport => write!(f, "used transport doesn't support the sync/async method"),
            Error::Transport(ref e) => write!(f, "transport error: {}", e),
            Error::Json(ref e) => write!(f, "JSON decode error: {}", e),
            Error::Rpc(ref r) => write!(f, "RPC error response: {:?}", r),
            Error::BatchDuplicateResponseId(ref v) => {
                write!(f, "duplicate RPC batch response ID: {}", v)
            }
            Error::WrongBatchResponseId(ref v) => write!(f, "wrong RPC batch response ID: {}", v),
            Error::NonceMismatch => write!(f, "Nonce of response did not match nonce of request"),
            Error::VersionMismatch => write!(f, "`jsonrpc` field set to non-\"2.0\""),
            Error::EmptyBatch => write!(f, "batches can't be empty"),
            Error::WrongBatchResponseSize => write!(f, "too many responses returned in batch"),
            Error::ResponseConversion(ref e) => write!(f, "response conversion error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use self::Error::*;

        match *self {
            Rpc(_)
            | NoTransportSupport
            | NonceMismatch
            | VersionMismatch
            | EmptyBatch
            | WrongBatchResponseSize
            | BatchDuplicateResponseId(_)
            | WrongBatchResponseId(_) => None,
            Transport(ref e) => Some(&**e),
            Json(ref e) => Some(e),
            ResponseConversion(ref e) => Some(&**e),
        }
    }
}

