* compiling for OSX:

for each compile run:
./build_osx.sh

test the output by executing:
build_osx\seni-c

* compiling for WebAssembly on OSX:

run this once in the console:
source misc\setup_wasm.sh

then for each compile run:
./build_wasm.sh

test the output by opening build_wasm\seni-wasm.html in a browser that supports WebAssembly

* compiling for Windows:

run this once in the console:
misc\setup_winconsole.bat

then for each compile run:
build_win.bat

test the output by executing:
build_win\main_win.exe

* compiling for WebAssembly on Windows:

run this once in the console:
misc\setup_wasm.bat

then for each compile run:
build_wasm.bat

test the output by opening build_wasm\seni-wasm.html in a browser that supports WebAssembly

* visual studio integration with the native windows build
  build the windows version (main_win.exe)
  load visual studio with: devenv ..\build_win\main_win.exe
  Insert any breakpoints
  Press F11 to begin execution
  When exiting for the first time, save the MSDev solution to the build_win folder
