#!/bin/bash

# build for the native platform and run tests

if [ "$1" == "test" ]; then
    pushd dist
    cc -o test -std=c99 ../src/c/test.c ../src/c/lib/unity/unity.c ../src/c/senie/*.c -lm -O2
    popd
    ./dist/test
fi

# -g flag is for producing debug information

if [ "$1" == "native" ]; then
    pushd dist
    cc -o native -std=c99 ../src/c/native.c ../src/c/senie/*.c -lm -O2 -g
    popd
    ./dist/native $2
fi


# clang -isystem /Users/indy/local/clang+llvm-4.0.1-x86_64-apple-macosx10.9.0/include/c++/v1 -I/Users/indy/local/clang+llvm-4.0.1-x86_64-apple-macosx10.9.0/include/c++/v1 -o test -std=c99 ../src/c/test.c ../src/c/unity/unity.c ../src/c/senie_*.c -lm -O2
