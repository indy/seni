pushd build_wasm
emcc -o seni-wasm.js ../code/wasm.c ../code/gl-matrix/*.c ../code/seni.c ../code/seni_*.c -O3 -s WASM=1
popd
