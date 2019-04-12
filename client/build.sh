#!/bin/sh

if [ "$1" = "release" ]
then
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/release/client.wasm --out-dir www --no-typescript --no-modules

    # isg note: in the early part of the Rust port, we'll have to use the js renderer, this requires access to the wasm memory, hence this hack
    #
#    sed -i "s/__exports.BridgeConfig = BridgeConfig;/__exports.BridgeConfig = BridgeConfig;\n\/\/ ISG HACK\n__exports.wasm = wasm;/g" www/sokoban_client.js
else
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen target/wasm32-unknown-unknown/debug/client.wasm --out-dir www --no-typescript --no-modules

    # isg note: in the early part of the Rust port, we'll have to use the js renderer, this requires access to the wasm memory, hence this hack
    #
    # sed -i "s/__exports.BridgeConfig = BridgeConfig;/__exports.BridgeConfig = BridgeConfig;\n\/\/ ISG HACK\n__exports.wasm = wasm;/g" www/sokoban_client.js
fi
