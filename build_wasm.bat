@echo off

mkdir build_wasm
pushd build_wasm
call emcc -o seni-wasm.html ..\code\wasm.c ..\code\seni.c -O3 -s WASM=1 --shell-file ..\misc\html_template\shell_minimal.html -DPLATFORM_WASM
popd
