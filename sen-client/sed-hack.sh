# isg note: in the early part of the Rust port, we'll have to use the js renderer, this requires access to the wasm memory, hence this hack
#
sed -i "s/__exports.BridgeConfig = BridgeConfig;/__exports.BridgeConfig = BridgeConfig;\n\/\/ ISG HACK\n__exports.wasm = wasm;/g" www/sen_client.js
