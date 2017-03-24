Keep things simple by using C.
Avoid C++, even trying to use it as a 'nicer C' results in increased compilation times, obtuse error messages, complicates the build/link commands and reduces platform independance.
Switch to Rust if you need a better C



* compiling and testing on Linux/MacOS:

./make/unix.sh

* compiling WebAssembly on Linux/MacOS:

run this once in the console:
source misc\setup_wasm.sh

then for each compile run:
./make/unix.sh wasm

test the output by opening build_wasm\seni-wasm.html in a browser that supports WebAssembly


* compiling for Windows:

setup the paths by running this once in the console:
misc\setup_win.bat

* compiling and testing on Windows

make\win.bat

* compiling for WebAssembly on Windows:

then for each compile run:
make\win.bat wasm



test the output by opening build_wasm\seni-wasm.html in a browser that supports WebAssembly

* visual studio integration with the native windows build
  build the windows version (main_win.exe)
  load visual studio with: devenv ..\build_win\main_win.exe
  Insert any breakpoints
  Press F11 to begin execution
  When exiting for the first time, save the MSDev solution to the build_win folder


