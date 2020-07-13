#!/usr/bin/env bash

echo "$(rustup default)" | grep -q "stable"
if [ "$?" != "0" ]; then
  # only run checks on stable
  exit 0
fi

set -euxo pipefail

cargo fmt --all -- --check
cargo clippy --all -- --deny=warnings
cargo check --all

# Enable all optional features
(cd yew \
  && cargo check --features cbor,msgpack,toml,yaml \
  && cargo clippy --features cbor,msgpack,toml,yaml -- --deny=warnings)

(cd yewtil \
  && cargo check --features all \
  && cargo clippy --features all)

(cd yew-router \
  && cargo check --features service \
  && cargo clippy --features service \
  && cargo check --no-default-features --features std_web,core,service \
  && cargo clippy --no-default-features --features std_web,core,service)

# Check stdweb
pushd yew-stdweb
cargo fmt --all -- --check
cargo clippy --all -- --deny=warnings
cargo check --all --target wasm32-unknown-unknown

# webgl_stdweb doesn't play nice with wasm-bindgen
(cd examples/webgl && cargo web check --target wasm32-unknown-unknown)
popd