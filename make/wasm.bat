@echo off

pushd build_wasm
call emcc -o seni-wasm.js ..\code\wasm.c ..\code\gl-matrix\*.c ..\code\seni.c -O3 -s WASM=1
popd
