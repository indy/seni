#!/bin/bash

if [ "$1" == "wasm" ]; then

    pushd app/dist
    emcc -o seni-wasm.js ../c/wasm.c ../c/gl-matrix/*.c ../c/seni.c ../c/seni_*.c -O3 -s WASM=1
    popd

else

    # build for the native platform and run tests
    mkdir build_osx
    pushd build_osx    
    # cc -o test -std=c99 -Wall -Wextra ../app/c/test.c ../app/c/unity/unity.c ../app/c/gl-matrix/*.c ../app/c/seni.c ../app/c/seni_*.c
    cc -o test -std=c99 ../app/c/test.c ../app/c/unity/unity.c ../app/c/gl-matrix/*.c ../app/c/seni.c ../app/c/seni_*.c -lm
    popd

    if [ "$1" == "test" ]; then
        ./build_osx/test
    fi

fi
