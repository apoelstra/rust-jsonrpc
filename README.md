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

## Githooks

To assist devs in catching errors _before_ running CI we provide some githooks. If you do not
already have locally configured githooks you can use the ones in this repository by running, in the
root directory of the repository:
```
git config --local core.hooksPath githooks/
```

Alternatively add symlinks in your `.git/hooks` directory to any of the githooks we provide.

## Design goals

This library was built with the primary purpose of talking to Bitcoin Core without the need for
additional dependencies. This means we do not, and likely never will, support async. If you are
writing an async application you might be interested in this [alternative JSONRPC
client](https://github.com/thomaseizinger/rust-jsonrpc-client).
