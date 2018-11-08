#!/bin/bash

pushd $(dirname "$0")
cargo +nightly run
popd
