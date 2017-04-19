@echo off

if "%1" == "wasm" (

   pushd app\dist
   rem ISG: have to list each c file as Windows command prompt doesn't do wildcard expansion
   call emcc -o seni-wasm.js ..\c\wasm.c ..\c\seni.c ..\c\seni_containers.c ..\c\seni_interp.c ..\c\seni_lang.c ..\c\seni_mathutil.c -O3 -s WASM=1 -s EXPORTED_FUNCTIONS="['_mc_m_wasm', '_parse_sample', '_copy_array']"
   popd

) else (

  mkdir build_win
  pushd build_win
  rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
  cl /W4 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /TC ..\app\c\test.c ..\app\c\unity\unity.c ..\app\c\seni.c ..\app\c\seni_*.c /link /OUT:test.exe
  popd

  if "%1" == "test" (
    .\build_win\test.exe
  )
)
