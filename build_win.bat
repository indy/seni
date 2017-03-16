@echo off

mkdir build_win
pushd build_win
rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /W4 -Zi ..\code\main_win.c ..\code\seni.c
popd
