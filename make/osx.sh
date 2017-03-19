mkdir build_osx
pushd build_osx    
cc -o seni ../code/test.c ../code/unity/unity.c ../code/seni.c ../code/seni_*.c
popd
./build_osx/seni              
