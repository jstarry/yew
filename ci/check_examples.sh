#!/usr/bin/env bash
echo "$(rustup default)" | grep -q "stable"
is_stable=$?
set -euxo pipefail # https://vaneyckt.io/posts/safer_bash_scripts_with_set_euxo_pipefail/

# Showcase includes all other examples
cd examples/showcase

# TODO Can't build some demos with release, need fix

if [ "$is_stable" == "0" ]; then
  # TODO - Emscripten builds are broken on beta/nightly
  # cargo web build --target asmjs-unknown-emscripten --use-system-emscripten
  cargo web build --target wasm32-unknown-emscripten --use-system-emscripten
fi

# TODO showcase doesn't support wasm-bindgen yet
cargo web build --target wasm32-unknown-unknown --use-system-emscripten

# Reset cwd
cd ../..
