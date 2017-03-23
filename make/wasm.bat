@echo off

pushd build_wasm
call emcc -o seni-wasm.js ..\code\wasm.c ..\code\gl-matrix\mat3.c ..\code\gl-matrix\mat4.c ..\code\gl-matrix\quat.c ..\code\gl-matrix\str.c ..\code\gl-matrix\vec3.c ..\code\seni.c ..\code\seni_interp.c ..\code\seni_mathutil.c -O3 -s WASM=1
rem call emcc -o seni-wasm.js ..\code\wasm.c ..\code\gl-matrix\*.c ..\code\seni.c -O3 -s WASM=1
popd
