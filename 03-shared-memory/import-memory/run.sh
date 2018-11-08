#!/bin/bash

pushd $(dirname "$0")
cargo +nightly run --release
wasm-gc wasm_with_mem.wasm
popd
