# Seni

Seni is a Scheme-like graphical language that runs on modern web browsers.

It's scripts can be annotated so that genetic algorithms can generate variations and the user can select which of the generated images should be used in future generations.

## Build


### Prerequisites

wasm-bindgen:
`cargo install -f wasm-bindgen-cli`

### Building web client:

1. `cd client`
2. `./build.sh`

### Run the server:

1. `cd server`
2. `cargo run`

can now view the page at localhost:8080/index.html
