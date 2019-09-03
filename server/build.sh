#!/bin/sh

# builds a release build by default

if [ "$1" = "run" ]
then
    cargo +nightly run
elif [ "$1" = "release" ]
then
    cargo +nightly build --release
else
    cargo +nightly build
fi
