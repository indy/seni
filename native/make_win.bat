@echo off

setlocal EnableDelayedExpansion

pushd dist

rem cl can expand wildcards
set compile_sources=..\src\native.c ..\..\core\src\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /I ..\..\core\src /I ..\..\core\src\seni /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !compile_sources! /link /OUT:native.exe

popd

rem .\native.exe %2
