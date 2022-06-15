#!/bin/sh -ex

FEATURES=""

# Use toolchain if explicitly specified
if [ -n "$TOOLCHAIN" ]
then
    alias cargo="cargo +$TOOLCHAIN"
fi

# Test without any features first
cargo test --verbose

# Test each feature
for feature in ${FEATURES}
do
    cargo test --verbose --features="$feature"
done

