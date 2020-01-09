# 'make' will build:
# - css from scss
# - wasm from rust
# - js from typescript
#
# 'make release' will build the release version of the wasm
#

all: css wasm index
.PHONY: all

# client
#
# todo: also add sketch to all

CSS_SRC=$(wildcard stylesheets/scss/*.scss)
CLIENT_RUST_SRC=$(wildcard client/src/*.rs)
CORE_RUST_SRC=$(wildcard core/src/*.rs) $(wildcard core/src/geometry/*.rs)
TYPESCRIPT_SRC=$(wildcard typescript/src/*)

release: css wasm-release index
css: www/stylesheet.css
wasm: www/client_bg.wasm
index: www/index.js
sketch: www/sketch.js

www/stylesheet.css: $(CSS_SRC)
	cargo run --manifest-path stylesheets/Cargo.toml -- stylesheets/scss/seni.scss www/stylesheet.css

www/client_bg.wasm: $(CLIENT_RUST_SRC) $(CORE_RUST_SRC)
	cargo build --manifest-path client/Cargo.toml --target wasm32-unknown-unknown
	wasm-bindgen client/target/wasm32-unknown-unknown/debug/client.wasm --out-dir www --no-typescript --no-modules

www/index.js: $(TYPESCRIPT_SRC) typescript/tsconfig-main.json
	tsc --project typescript/tsconfig-main.json

www/sketch.js: $(TYPESCRIPT_SRC) typescript/tsconfig-sketch.json
	tsc --project typescript/tsconfig-sketch.json

wasm-release:
	cargo build --manifest-path client/Cargo.toml --release --target wasm32-unknown-unknown
	wasm-bindgen client/target/wasm32-unknown-unknown/release/client.wasm --out-dir www --no-typescript --no-modules

# clean
#
clean:
	rm -f www/stylesheet.css
	rm -f www/client_bg.wasm
	rm -f www/client.js
	rm -f www/index.js
	rm -f www/sketch.js
