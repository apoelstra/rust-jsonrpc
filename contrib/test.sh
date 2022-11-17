#!/bin/sh -ex

FEATURES="simple_http simple_tcp simple_uds proxy"

cargo --version
rustc --version

# Some tests require certain toolchain types.
NIGHTLY=false
if cargo --version | grep nightly; then
    NIGHTLY=true
fi

# On MacOS on 1.41 we get a link failure with syn
# This is fixed by https://github.com/rust-lang/rust/pull/91604 (I think)
# but implies that we can't do testing on MacOS for now, at least with 1.41.
if cargo --version | grep "1\.41\.0"; then
    if [ "$RUNNER_OS" = "macOS" ]; then
        exit 0
    fi
fi

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

# Build docs if told to, only works with nightly toolchain.
if [ "$DO_DOCS" = true ]; then
    if [ "$NIGHTLY" = false ]; then
        echo "DO_DOCS requires a nightly toolchain (consider using RUSTUP_TOOLCHAIN)"
        exit 1
    fi

    RUSTDOCFLAGS="--cfg docsrs" cargo rustdoc --features="$FEATURES" -- -D rustdoc::broken-intra-doc-links
fi

exit 0
