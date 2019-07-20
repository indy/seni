#!/bin/sh

# useful rust commands

if [ "$1" = "doc" ]
then
    cargo doc --no-deps --open
fi
