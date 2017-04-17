@echo off

if "%1" == "wasm" (

   pushd build_wasm
   rem ISG: have to list each c file as Windows command prompt doesn't do wildcard expansion
   call emcc -o seni-wasm.js ..\src\wasm.c ..\src\seni.c ..\src\seni_containers.c ..\src\seni_interp.c ..\src\seni_lang.c ..\src\seni_mathutil.c -O3 -s WASM=1 -s EXPORTED_FUNCTIONS="['_mc_m_wasm', '_parse_sample', '_copy_array']"
   popd

) else (

  mkdir build_win
  pushd build_win
  rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
  cl /W4 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\seni.c ..\src\seni_*.c /link /OUT:test.exe
  popd

  if "%1" == "test" (
    .\build_win\test.exe
  )
)
