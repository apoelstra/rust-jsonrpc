[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-jsonrpc)

# Rust Version compatibility

This library is compatible with Rust **1.29.0** or higher. However, because some
dependencies have increased their Rust versions in minor/patch revisions, a bit
of work is required for users who wish to use older versions of the compiler.
In particular,

For compatibility with older versions of rustc, use the following commands to
pull your dependencies back down to unbroken versions:
```
cargo update --package 'serde_json' --precise '1.0.39'
cargo update --package 'serde' --precise '1.0.98'
cargo update --package 'serde_derive' --precise '1.0.98'
cargo update --package 'byteorder' --precise '1.3.4'
```

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

