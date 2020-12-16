#!/bin/sh -ex

FEATURES=""

# Use toolchain if explicitly specified
if [ -n "$TOOLCHAIN" ]
then
    alias cargo="cargo +$TOOLCHAIN"
fi

cargo update ## create lockfile
if [ "$TRAVIS_RUST_VERSION" = "1.29.0" ]; then
    cargo update --package 'serde_json' --precise '1.0.39'
    cargo update --package 'serde_derive' --precise '1.0.98'
fi

# Test without any features first
cargo test --verbose

# Test each feature
for feature in ${FEATURES}
do
    cargo test --verbose --features="$feature"
done

