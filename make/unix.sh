#!/bin/bash

# build for the native platform and run tests
mkdir build_unix

if [ "$1" == "test" ]; then
    pushd build_unix    
    cc -o test -std=c99 ../app/c/main_test.c ../app/c/unity/unity.c ../app/c/seni_*.c -lm -O1
    popd
    ./build_unix/test
fi

if [ "$1" == "compile" ]; then
    pushd build_unix    
    cc -o compile -std=c99 ../app/c/main_compile.c ../app/c/seni_*.c -lm -O1
    popd
    ./build_unix/compile
fi


