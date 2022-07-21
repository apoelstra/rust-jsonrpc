#!/bin/sh -ex

FEATURES="simple_http simple_tcp simple_uds"

cargo --version
rustc --version

# Defaults / sanity checks
cargo build --all
cargo test --all

if [ "$DO_FEATURE_MATRIX" = true ]; then
    cargo build --no-default-features
    cargo test --no-default-features

    # All features
    cargo build --no-default-features --features="$FEATURES"
    cargo test --no-default-features --features="$FEATURES"

    # Single features
    for feature in ${FEATURES}
    do
        cargo test --no-default-features --features="$feature"
    done
fi

exit 0
