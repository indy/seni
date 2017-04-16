@echo off

rem Configure console for compiling C to WebAssembly
rem See https://developer.mozilla.org/en-US/docs/WebAssembly/C_to_wasm

rem pre-requisites: git, cmake, visual studio 2015 compiler, python 2.7.x (64bit), pywin32 (64bit)
rem install and activate emsdk from https://github.com/juj/emsdk.git

rem once that's all done, it's assumed that C:\Users\indy\.emscripten exists

rem setup emsdk environment variables
call "D:\code\wasm\emsdk\emsdk_env.bat"
rem modify path so that emcc can be invoked
set path=d:\code\wasm\emsdk\emscripten\incoming;%path%

mkdir build_wasm
copy misc\html_template\seni.html build_wasm\.

rem NOTE: visual studion 2015 is only required to build emsdk, we can use the latest visual studio for seni

call "C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat" x64
rem devenv : loads Visual Studio

