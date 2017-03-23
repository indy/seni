pushd build_wasm
emcc -o seni-wasm.js ../src/wasm.c ../src/gl-matrix/*.c ../src/seni.c ../src/seni_*.c -O3 -s WASM=1
popd
