#!/usr/bin/env bash

# `rust-jsonrpc` does not have a std feature.
FEATURES_WITH_STD=""

# So this is the var to use for all tests.
FEATURES_WITHOUT_STD="simple_http minreq_http simple_tcp simple_uds proxy"

# Run these examples.
EXAMPLES=""
