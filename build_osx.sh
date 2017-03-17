mkdir build_osx
pushd build_osx    
cc -o seni-c ../code/test.c ../code/seni.c ../code/unity/unity.c
popd
./build_osx/seni-c
