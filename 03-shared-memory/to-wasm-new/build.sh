#!/bin/bash

pushd $(dirname "$0")
cargo +nightly build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/to_wasm_new.wasm .
wasm-gc to_wasm_new.wasm
popd
