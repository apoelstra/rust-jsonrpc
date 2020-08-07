[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-jsonrpc)

# Rust Version compatibility

This library is compatible with Rust **1.29.0** or higher. However, because some
dependencies have increased their Rust versions in minor/patch revisions, a bit
of work is required for users who wish to use older versions of the compiler.
In particular,

For compatibility with Rust **1.31.0** or higher, run
```
cargo update --package 'unicode-normalization' --precise '0.1.9'
```

For compatibility with Rust **1.27.0** or higher, additionally run
```
cargo update --package 'cfg-if' --precise '0.1.9'
cargo update --package 'serde_json' --precise '1.0.39'
cargo update --package 'serde_derive' --precise '1.0.98'
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
    let mut client = jsonrpc::client::Client::new("example.org".to_owned(), None, None);
    let request = client.build_request("getmystruct".to_owned(), vec![]);
    match client.send_request(&request).and_then(|res| res.into_result::<MyStruct>()) {
        Ok(mystruct) => // Ok!
        Err(e) => // Not so much.
    }
}

```

