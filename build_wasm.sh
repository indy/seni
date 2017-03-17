mkdir build_wasm
pushd build_wasm
emcc -o seni-wasm.html ../code/main_wasm.c ../code/seni.c -O3 -s WASM=1 --shell-file ../misc/html_template/shell_minimal.html -DPLATFORM_WASM
popd
