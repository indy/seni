@echo off

pushd build_wasm
rem ISG: have to list each c file as Windows command prompt doesn't do wildcard expansion
rem call emcc -o seni-wasm.js ..\src\wasm.c ..\src\gl-matrix\mat3.c ..\src\gl-matrix\mat4.c ..\src\gl-matrix\quat.c ..\src\gl-matrix\str.c ..\src\gl-matrix\vec3.c ..\src\seni.c ..\src\seni_interp.c ..\src\seni_mathutil.c -O3 -s WASM=1
call emcc -o seni-wasm.js ..\src\wasm.c ..\src\seni.c ..\src\seni_interp.c ..\src\seni_mathutil.c -O3 -s WASM=1
popd
