## Setup

```shell
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup update stable
cargo install --git https://github.com/alexcrichton/wasm-gc
```

## Run

1. Run `build.sh` in `to-wasm-new`. It creates a `to_wasm_new.wasm` which soft linked into `run-wasm` folder.
2. Run `run.sh` in `run-wasm`

