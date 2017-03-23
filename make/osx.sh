mkdir build_osx
pushd build_osx    
cc -o test ../code/test.c ../code/unity/unity.c ../code/gl-matrix/*.c ../code/seni.c ../code/seni_*.c
popd
./build_osx/test
