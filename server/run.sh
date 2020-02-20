#!/bin/sh

if [ "$1" = "release" ]
then
    cargo run --bin seni_server --release
else
    cargo run --bin seni_server
fi
