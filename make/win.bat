@echo off

setlocal EnableDelayedExpansion

if "%1" == "wasm" (
   if not exist "app\dist" mkdir app\dist
   pushd app\dist
      rem emcc doesn't expand wildcards
      set seni_sources=..\c\wasm.c
      for /f %%F in ('dir /b ..\c\seni_*.c') do call set seni_sources=%%seni_sources%% ..\c\%%F

      call emcc -o seni-wasm.js !seni_sources! -O3 -s WASM=1
   popd

) else (
  if not exist "build_win" mkdir build_win

  pushd build_win
      rem cl can expand wildcards
      set test_sources=..\app\c\test.c ..\app\c\unity\unity.c ..\app\c\seni_*.c
      
      rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
      cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /TC !test_sources! /link /OUT:test.exe
  popd

  if "%1" == "test" (
    .\build_win\test.exe
  )
)


