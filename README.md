[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-json)

# Rust JSONRPC Client

Rudimentary support for sending JSONRPC 2.0 requests and receiving responses.

## Serde Support

This includes a of macro to enable serialization/deserialization of
structures without using stable or nightly. They can be used as follows:
```rust
#[macro_use] extern crate jsonrpc;
extern crate serde;

struct MyStruct {
    elem1: bool,
    elem2: String,
    elem3: Vec<usize>
}
serde_struct_impl!(MyStruct, elem1, elem2, elem3 <- "alternate name for elem3");
```

There is also a variant of this for enums representing structures that might
have one of a few possible forms. For example
```
struct Variant1 {
    success: bool,
    success_message: String
}

struct Variant2 {
    success: bool,
    errors: Vec<String>
}

enum Reply {
    Good(Variant1),
    Bad(Variant2)
}
serde_struct_enum_impl!(Reply,
    Good, Variant1, success, success_message;
    Bad, Variant2, success, errors
);
```
Note that this macro works by returning the first variant for which all
fields are present. This means that if one variant is a superset of another,
the larger one should be given first to the macro to prevent the smaller
from always being matched.

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

serde_struct_impl!(MyStruct, elem1, elem2, elem3);

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

