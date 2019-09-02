#!/bin/sh

# builds a release build by default

if [ "$1" = "run" ]
then
    cargo +nightly run
else
    cargo +nightly build
fi
