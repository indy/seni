@echo off

setlocal EnableDelayedExpansion

if "%1" == "release" (
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/release/client.wasm --out-dir www --no-typescript --no-modules
) else (
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/debug/client.wasm --out-dir www --no-typescript --no-modules
)
