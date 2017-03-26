#!/bin/bash

if [ "$1" == "wasm" ]; then

    mkdir build_wasm    
    pushd build_wasm
    emcc -o seni-wasm.js ../src/wasm.c ../src/gl-matrix/*.c ../src/seni.c ../src/seni_*.c -O3 -s WASM=1
    popd

else

    # build for the native platform and run tests
    mkdir build_osx
    pushd build_osx    
    # cc -o test -std=c99 -Wall -Wextra ../src/test.c ../src/unity/unity.c ../src/gl-matrix/*.c ../src/seni.c ../src/seni_*.c
    cc -o test -std=c99 ../src/test.c ../src/unity/unity.c ../src/gl-matrix/*.c ../src/seni.c ../src/seni_*.c
    popd

    ./build_osx/test

fi
