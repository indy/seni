Seni
====

### Note: Development happens at [git.indy.io/indy/seni](https://git.indy.io/indy/seni)

## Overview

Seni is a Scheme-like graphical language that runs on modern web browsers.

It's scripts can be annotated so that genetic algorithms can generate variations and the user can select which of the generated images should be used in future generations.

## Prerequisites
```sh
$ rustup update
$ rustup target add wasm32-unknown-unknown
$ cargo install -f wasm-bindgen-cli
```

## Building
build:
```sh
$ cd client
$ ./build.sh
```

launch a server:
```sh
$ cd server
$ cargo run
```

You can now useseni at [127.0.0.1:8080/index.html](http://127.0.0.1:8080/index.html)

## License
GNU Affero General Public License
