rem cargo +nightly build --target wasm32-unknown-unknown
rem wasm-bindgen target/wasm32-unknown-unknown/debug/sen_client.wasm --out-dir www --no-typescript --no-modules

rem TODO: sed hack to enable acces to memory

@echo off

setlocal EnableDelayedExpansion


if "%1" == "release" (
    cargo +nightly build --release --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/release/sen_client.wasm --out-dir www --no-typescript --no-modules

rem isg note: in the early part of the Rust port, we'll have to use the js renderer, this requires access to the wasm memory, hence this hack
rem
rem    sed -i "s/__exports.BridgeConfig = BridgeConfig;/__exports.BridgeConfig = BridgeConfig;\n\/\/ ISG HACK\n__exports.wasm = wasm;/g" www/sokoban_client.js

) else (
    cargo +nightly build --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/debug/sen_client.wasm --out-dir www --no-typescript --no-modules

rem isg note: in the early part of the Rust port, we'll have to use the js renderer, this requires access to the wasm memory, hence this hack
rem
rem    sed -i "s/__exports.BridgeConfig = BridgeConfig;/__exports.BridgeConfig = BridgeConfig;\n\/\/ ISG HACK\n__exports.wasm = wasm;/g" www/sokoban_client.js
)
