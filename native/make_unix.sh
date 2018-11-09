#!/bin/bash

# -g flag is for producing debug information
# paths are relative to dist directory
cc -o native -std=c99 src/native.c ../core/src/seni/*.c -lm -O2 -g -I ../core/src
./native $2
