@echo off

mkdir build
pushd build
rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /W4 -Zi ..\code\main.c ..\code\seni.c
popd
