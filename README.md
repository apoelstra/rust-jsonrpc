[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-jsonrpc)

# Rust Version compatibility

This library is compatible with Rust **1.29.0** or higher. However, because some
dependencies have increased their Rust versions in minor/patch revisions, a bit
of work is required for users who wish to use older versions of the compiler.
In particular,

For compatibility with older versions of rustc, use the following commands to
pull your dependencies back down to unbroken versions:
```
cargo +1.29 update -p byteorder --precise "1.3.4"
cargo +1.29 update --package 'serde_json' --precise '1.0.39'
cargo +1.29 update --package 'serde' --precise '1.0.98'
cargo +1.29 update --package 'serde_derive' --precise '1.0.98'
```

# Rust JSONRPC Client

Rudimentary support for sending JSONRPC 2.0 requests and receiving responses.

As an example, hit a local bitcoind JSON-RPC endpoint and call the `uptime` command.

```rust
use jsonrpc::Client;
use jsonrpc::simple_http::{self, SimpleHttpTransport};

fn client(url: &str, user: &str, pass: &str) -> Result<Client, simple_http::Error> {
    let t = SimpleHttpTransport::builder()
        .url(url)?
        .auth(user, Some(pass))
        .build();

    Ok(Client::with_transport(t))
}

// Demonstrate an example JSON-RCP call against bitcoind.
fn main() {
    let client = client("localhost:18443", "user", "pass").expect("failed to create client");
    let request = client.build_request("uptime", &[]);
    let response = client.send_request(request).expect("send_request failed");

    // For other commands this would be a struct matching the returned json.
    let result: u64 = response.result().expect("response is an error, use check_error");
    println!("bitcoind uptime: {}", result);
}
```

