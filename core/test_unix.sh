#!/bin/bash

# run tests
cc -o test -std=c99 src/test.c ../core/src/lib/unity/unity.c ../core/src/seni/*.c -lm -O2 -I ../core/src
./test
