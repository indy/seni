#!/bin/sh

# builds a release build by default

if [ "$1" = "debug" ]
then
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/debug/client.wasm --out-dir www --no-typescript --no-modules
else
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/release/client.wasm --out-dir www --no-typescript --no-modules
fi
