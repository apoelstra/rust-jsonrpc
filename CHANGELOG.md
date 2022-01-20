# 0.12.1 - 2022-01-20

## Features

* A new set of transports were added for JSONRPC over raw TCP sockets (one using `SocketAddr`, and
  one UNIX-only using Unix Domain Sockets)

## Bug fixes

* The `Content-Type` HTTP header is now correctly set to `application/json`
* The `Connection: Close` HTTP header is now sent for requests

# 0.12.0 - 2020-12-16

* Remove `http` and `hyper` dependencies
* Implement our own simple HTTP transport for Bitcoin Core
* But allow use of generic transports

# 0.11.0 - 2019-04-05

* [Clean up the API](https://github.com/apoelstra/rust-jsonrpc/pull/19)
* [Set the content-type header to json]((https://github.com/apoelstra/rust-jsonrpc/pull/21)
* [Allow no `result` field in responses](https://github.com/apoelstra/rust-jsonrpc/pull/16)
* [Add batch request support](https://github.com/apoelstra/rust-jsonrpc/pull/24)

