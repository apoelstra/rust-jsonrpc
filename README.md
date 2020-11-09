[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-jsonrpc)

# Rust JSONRPC Client

Rudimentary support for sending JSONRPC 2.0 requests and receiving responses.

To send a request which should retrieve the above structure, consider the following
example code

```rust
extern crate jsonrpc;
extern crate serde;
#[macro_use] extern crate serde_derive;

#[derive(Deserialize)]
struct MyStruct {
    elem1: bool,
    elem2: String,
    elem3: Vec<usize>
}

fn main() {
    // The two Nones are for user/pass for authentication
    let rtt = jsonrpc::simple_rtt::Tripper::new();
    let client = jsonrpc::client::Client::with_rtt(rtt, "example.org".to_owned(), None, None);
    match client.do_rpc::<MyStruct>(&request) {
        Ok(mystruct) => // Ok!
        Err(e) => // Not so much.
    }
}

```

