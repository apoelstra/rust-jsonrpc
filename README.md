[![Status](https://travis-ci.org/apoelstra/rust-jsonrpc.png?branch=master)](https://travis-ci.org/apoelstra/rust-jsonrpc)

# Rust Version compatibility

This library is compatible with Rust **1.41.1** or higher.

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

