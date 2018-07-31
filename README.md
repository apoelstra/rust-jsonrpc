<p align="center">
  <a href="https://travis-ci.org/apoelstra/rust-jsonrpc">
    <img src="https://travis-ci.org/apoelstra/rust-jsonrpc.svg?branch=master" alt="Build Status">
    </img>
  </a>

  <a href="https://crates.io/crates/rust-jsonrpc">
    <img src="https://img.shields.io/crates/v/jsonrpc.svg" alt="Crates.io Version">
    </img>
  </a>

  <br/>

   <strong>
     <a href="https://docs.rs/jsonrpc">
       Documentation
     </a>
   </strong>
</p>

# Rust JSON-RPC Client

Rudimentary support for sending JSONRPC 2.0 requests and receiving responses.

## Example: sending a basic request

To send a request which should retrieve the above structure, consider the
following example code:

```rust
extern crate jsonrpc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct MyStruct {
    elem1: bool,
    elem2: String,
    elem3: Vec<usize>
}

fn main() {
    // The two Nones are for user/pass for authentication
    let mut client = jsonrpc::client::Client::new("example.org", None, None);
    let request = client.build_request("getmystruct", vec![]);
    match client.send_request(&request).and_then(|res| res.into_result::<MyStruct>()) {
        Ok(mystruct) => // Ok!
        Err(e) => // Not so much.
    }
}

```
