@echo off

setlocal EnableDelayedExpansion

if not exist "build_win" mkdir build_win


if "%1" == "test" (

pushd build_win
rem cl can expand wildcards
set test_sources=..\app\c\test.c ..\app\c\lib\unity\unity.c ..\app\c\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !test_sources! /link /OUT:test.exe
popd

.\build_win\test.exe
)

if "%1" == "native" (

pushd build_win
rem cl can expand wildcards
set compile_sources=..\app\c\native.c ..\app\c\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !compile_sources! /link /OUT:native.exe
popd

rem .\build_win\native.exe %2
)





