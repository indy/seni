@echo off

mkdir build_win
pushd build_win
rem compiler switches: https://docs.microsoft.com/en-us/cpp/build/reference/compiler-options-listed-alphabetically
rem cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\gl-matrix\*.c ..\src\seni.c ..\src\seni_*.c /link advapi32.lib /OUT:test.exe
rem cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\gl-matrix\*.c ..\src\seni.c ..\src\seni_*.c /link  /OUT:test.exe
cl /W4 -Zi /D_CRT_SECURE_NO_DEPRECATE /TC ..\src\test.c ..\src\unity\unity.c ..\src\seni.c ..\src\seni_*.c /link  /OUT:test.exe
popd
.\build_win\test.exe
