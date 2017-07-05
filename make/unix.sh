#!/bin/bash

# build for the native platform and run tests
mkdir build_unix
pushd build_unix    
cc -o test -std=c99 ../app/c/test.c ../app/c/unity/unity.c ../app/c/seni_*.c -lm -O1
popd

if [ "$1" == "test" ]; then
    ./build_unix/test
fi


