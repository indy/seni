@echo off

if "%1" == "wasm" (

   pushd build_wasm
   rem ISG: have to list each c file as Windows command prompt doesn't do wildcard expansion
   rem call emcc -o seni-wasm.js ..\src\wasm.c ..\src\gl-matrix\mat3.c ..\src\gl-matrix\mat4.c ..\src\gl-matrix\quat.c ..\src\gl-matrix\str.c ..\src\gl-matrix\vec3.c ..\src\seni.c ..\src\seni_interp.c ..\src\seni_mathutil.c -O3 -s WASM=1
   call emcc -o seni-wasm.js ..\src\wasm.c ..\src\seni.c ..\src\seni_interp.c ..\src\seni_mathutil.c -O3 -s WASM=1
   popd

) else (

  mkdir build_win
  pushd build_win
  rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
  rem cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\gl-matrix\*.c ..\src\seni.c ..\src\seni_*.c /link advapi32.lib /OUT:test.exe
  rem cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\gl-matrix\*.c ..\src\seni.c ..\src\seni_*.c /link  /OUT:test.exe
  cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\seni.c ..\src\seni_*.c /link  /OUT:test.exe
  popd
  .\build_win\test.exe
)
