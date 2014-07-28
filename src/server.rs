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

//! # JSON RPC-Server Support
//!

use http::server;
use http::headers::content_type::MediaType;

use std::io::net::ip::{SocketAddr, Ipv4Addr};

use serialize::json;
use time;

use error::{Error, result_to_response};
use Request;
use decode::{json_decode_field, json_decode_field_list, json_decode_field_string};

/// A Server which reacts to JSONRPC requests by passing the request,
/// along with a response channel, across a channel.
#[deriving(Clone)]
pub struct JsonRpcServer {
  req_tx: Sender<(Request, Sender<Result<json::Json, Error>>)>,
}

impl JsonRpcServer {
  /// Constructor: returns a new `JsonRpcServer` along with a `Receiver`
  /// which should be listened on for new requests from peers
  pub fn new() -> (JsonRpcServer, Receiver<(Request, Sender<Result<json::Json, Error>>)>) {
    let (tx, rx) = channel();
    (JsonRpcServer { req_tx: tx }, rx)
  }
}

impl server::Server for JsonRpcServer {
  fn get_config (&self) -> server::Config {
    server::Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8001 } }
  }

  fn handle_request (&self, r: server::Request, w: &mut server::ResponseWriter) {
    w.headers.date = Some (time::now_utc());
    w.headers.content_type = Some (MediaType {
      type_: "application".to_string(),
      subtype: "json".to_string(),
      parameters: vec![("charset".to_string(), "UTF-8".to_string())]
    });
    w.headers.server = Some("rust-bitcoin-rpc".to_string());

    // Decode the request
    let js = json::from_str(r.body.as_slice());
    if js.is_err() { return; }
    let js = js.unwrap();
    let id = json_decode_field(&js, "id");
    let method = json_decode_field_string(&js, "method");
    let params = json_decode_field_list(&js, "params");

    // If it was a valid request, pass it to the main thread to get a response
    let response = if id.is_ok() && method.is_ok() && params.is_ok() {
      let (resp_tx, resp_rx) = channel();
      let request = Request {
        id: id.clone().unwrap(),
        method: method.unwrap(),
        params: params.unwrap()
      };
      // Request response from main thread
      self.req_tx.send((request, resp_tx));
      result_to_response(resp_rx.recv(), id.unwrap())
    // Otherwise use the error as a response
    } else if id.is_err() {
      result_to_response(id, json::Null)
    } else if method.is_err() {
      result_to_response(method.map(|meth| json::String(meth)), id.unwrap())
    } else if params.is_err() {
      result_to_response(params.map(|parms| json::List(parms)), id.unwrap())
    // Romy says, `else` as code for `else if [last thing i can think of]` is a
    // subtle bug waiting to happen. So actually use `else if` and add an unreachable
    // clause to satisfy the code-path checker
    } else {
      unreachable!()
    };

    // Pass the response to the peer
    let reply_str = json::encode(&response);
    let reply_bytes = reply_str.as_bytes();

    w.headers.content_length = Some(reply_bytes.len());
    if w.write(reply_bytes).is_err() {
      return;
    }
  }
}

