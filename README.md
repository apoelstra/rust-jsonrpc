[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-json)

# Rust JSONRPC Client

Rudimentary support for sending JSONRPC 2.0 requests and receiving responses.

## Serde Support

This includes a pair of macros to enable serialization/deserialization of
structures without using stable or nightly. They can be used as follows:
```rust
#[macro_use] extern crate jsonrpc;
extern crate serde;

struct MyStruct {
    elem1: bool,
    elem2: String,
    elem3: Vec<usize>
}

serde_struct_serialize!(
    MyStruct,
    MyStructMapVisitor,
    elem1 => 0,
    elem2 => 1,
    elem3 => 2
);

serde_struct_deserialize!(
    MyStruct,
    MyStructVisitor,
    MyStructField,
    MyStructFieldVisitor,
    elem1 => Elem1,
    elem2 => Elem2,
    elem3 => Elem3
);
```
The important parts in the above are that the name of the structure and names
of the fields match those of the actual struct; every other identifier is used
internally to the macros and can be made up.

(If anyone has ideas for how to clean up this external interface, please let
me know.)

## JSONRPC

To send a request which should retrieve the above structure, consider the following
example code

```rust
#[macro_use] extern crate jsonrpc;
extern crate serde;

struct MyStruct {
    elem1: bool,
    elem2: String,
    elem3: Vec<usize>
}

serde_struct_deserialize!(
    MyStruct,
    MyStructVisitor,
    MyStructField,
    MyStructFieldVisitor,
    elem1 => Elem1,
    elem2 => Elem2,
    elem3 => Elem3
);

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

