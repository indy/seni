@echo off

mkdir build_wasm
pushd build_wasm
rem call emcc ..\code\hello.c -s WASM=1 -o hello.html -DPLATFORM_WASM
call emcc -o seni-wasm.html ..\code\main_wasm.c ..\code\seni.c -O3 -s WASM=1 --shell-file ..\misc\html_template\shell_minimal.html -DPLATFORM_WASM
popd
