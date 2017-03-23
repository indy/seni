@echo off

mkdir build_win
pushd build_win
rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\code\test.c ..\code\unity\unity.c ..\code\gl-matrix\*.c ..\code\seni.c ..\code\seni_*.c /link /OUT:test.exe
popd
.\build_win\test.exe
