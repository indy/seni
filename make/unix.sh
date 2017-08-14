#!/bin/bash

# build for the native platform and run tests
mkdir build_unix

if [ "$1" == "test" ]; then
    pushd build_unix    
    cc -o test -std=c99 ../app/c/main_test.c ../app/c/unity/unity.c ../app/c/seni_*.c -lm -O2
    popd
    ./build_unix/test
fi

if [ "$1" == "native" ]; then
    pushd build_unix    
    cc -o native -std=c99 ../app/c/main_native.c ../app/c/seni_*.c -lm -O2
    popd
    ./build_unix/native $2
fi


# clang -isystem /Users/indy/local/clang+llvm-4.0.1-x86_64-apple-macosx10.9.0/include/c++/v1 -I/Users/indy/local/clang+llvm-4.0.1-x86_64-apple-macosx10.9.0/include/c++/v1 -o test -std=c99 ../app/c/main_test.c ../app/c/unity/unity.c ../app/c/seni_*.c -lm -O2
