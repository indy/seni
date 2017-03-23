mkdir build_osx
pushd build_osx    
cc -o test ../src/test.c ../src/unity/unity.c ../src/gl-matrix/*.c ../src/seni.c ../src/seni_*.c
popd
./build_osx/test
