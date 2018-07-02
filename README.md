[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-json)

# Rust JSONRPC Client

Rudimentary support for sending JSONRPC 2.0 requests and receiving responses.

## JSONRPC

To send a request which should retrieve the above structure, consider the following
example code

```rust
extern crate jsonrpc;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize, Serialize)]
struct MyStruct {
    elem1: bool,
    elem2: String,
    elem3: Vec<usize>
}

fn main() {
    // The two Nones are for user/pass for authentication
    let mut client = jsonrpc::client::Client::new("example.org", None, None);
    let request = client.build_request("getmystruct", vec![]);
    match client.execute(request).and_then(|res| res.into_result::<MyStruct>()) {
        Ok(mystruct) => {}, // Ok!
        Err(e) => {}, // Not so much.
    }
}

```

