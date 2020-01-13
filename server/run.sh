#!/bin/sh

if [ "$1" = "release" ]
then
    cargo run --release
else
    cargo run
fi
