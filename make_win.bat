@echo off

setlocal EnableDelayedExpansion

rem if not exist "dist" mkdir dist


if "%1" == "test" (

pushd dist
rem cl can expand wildcards
set test_sources=..\src\c\test.c ..\src\c\lib\unity\unity.c ..\src\c\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !test_sources! /link /OUT:test.exe
popd

.\dist\test.exe
)

if "%1" == "native" (

pushd dist
rem cl can expand wildcards
set compile_sources=..\src\c\native.c ..\src\c\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !compile_sources! /link /OUT:native.exe
popd

rem .\dist\native.exe %2
)
