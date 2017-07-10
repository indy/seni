@echo off

setlocal EnableDelayedExpansion

if not exist "build_win" mkdir build_win


if "%1" == "test" (

pushd build_win
rem cl can expand wildcards
set test_sources=..\app\c\main_test.c ..\app\c\unity\unity.c ..\app\c\seni_*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /TC !test_sources! /link /OUT:test.exe
popd

.\build_win\test.exe
)

if "%1" == "compile" (

pushd build_win
rem cl can expand wildcards
set compile_sources=..\app\c\main_compile.c ..\app\c\seni_*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /TC !compile_sources! /link /OUT:compile.exe
popd

.\build_win\compile.exe
)





