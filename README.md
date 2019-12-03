Seni
====

### Note: The canonical home page for this project is [git.indy.io/indy/seni](https://git.indy.io/indy/seni)

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

You can now useseni at [127.0.0.1:3210/](http://127.0.0.1:3210/)

## License
GNU Affero General Public License
