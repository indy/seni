@echo off

setlocal EnableDelayedExpansion

rem cl can expand wildcards
set compile_sources=src\native.c ..\core\src\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !compile_sources! /link /OUT:native.exe /I ..\core.src


rem .\native.exe %2
