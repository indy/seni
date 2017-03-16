Value Simplicity.

build.bat/build.sh replaces build tools



// debug info
cl /Zi foo.c




cd code
build.bat

devenv ..\build\main.exe

F11



compiling for Windows:

run this once in the console:
misc\setup_winconsole.bat

then for each compile run:
build.bat

compiling for WebAssembly on Windows:

run this once in the console:
misc\setup_wasm.bat

then for each compile run:
build_wasm.bat
