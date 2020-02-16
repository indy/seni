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

# Make doesn't come with a recursive wildcard function so we have to use this
# complicated thing which was copy/pasted from StackOverflow.
# Fucking hell, why isn't there a built-in function to recursively traverse
# a directory and select files that match a wildcard?
#
# https://stackoverflow.com/questions/2483182/recursive-wildcards-in-gnu-make/18258352#18258352
#
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

CSS_SRC=$(wildcard stylesheets/scss/*.scss)
CLIENT_RUST_SRC=$(wildcard client/src/*.rs)
CORE_RUST_SRC=$(call rwildcard, core/src, *.rs)
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
