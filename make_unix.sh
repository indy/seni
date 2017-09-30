#!/bin/bash

# build for the native platform and run tests

if [ "$1" == "test" ]; then
    pushd dist    
    cc -o test -std=c99 ../app/c/test.c ../app/c/lib/unity/unity.c ../app/c/seni/*.c -lm -O2
    popd
    ./dist/test
fi

if [ "$1" == "native" ]; then
    pushd dist    
    cc -o native -std=c99 ../app/c/native.c ../app/c/seni/*.c -lm -O2
    popd
    ./dist/native $2
fi


# clang -isystem /Users/indy/local/clang+llvm-4.0.1-x86_64-apple-macosx10.9.0/include/c++/v1 -I/Users/indy/local/clang+llvm-4.0.1-x86_64-apple-macosx10.9.0/include/c++/v1 -o test -std=c99 ../app/c/test.c ../app/c/unity/unity.c ../app/c/seni_*.c -lm -O2
