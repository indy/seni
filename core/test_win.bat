@echo off

setlocal EnableDelayedExpansion


pushd dist

rem cl can expand wildcards
set test_sources=..\src\test.c ..\..\core\src\lib\unity\unity.c ..\..\core\src\seni\*.c

rem https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /nologo /W4 /wd4146 /wd4127 /wd4001 -Zi -Za /D_CRT_SECURE_NO_DEPRECATE /DSENI_BUILD_WINDOWS /TC !test_sources! /link /OUT:test.exe

popd

.\dist\test.exe
