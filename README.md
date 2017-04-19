# Seni

Seni is a Scheme-like graphical language that runs on modern web browsers.

It's scripts can be annotated so that genetic algorithms can generate variations and the user can select which of the generated images should be used in future generations.

## Build

### Prerequisites:

1. go and node.js should be installed
2. `npm install`

### Build:

1. `npm run build`

### Test:

1. `npm run test:web`
2. visit http://localhost:8080/webpack-dev-server/testBundle
http://localhost:8080/testBundle

The tests will automatically re-run in the browser whenever the source code changes.

### Running

1. `go run server.go`
2. visit http://localhost:3000

## Windows 10 Shenanigans

You will need to build a server.exe and add it to the Firewall whitelist in order to prevent a Windows Firewall dialog appearing every time.

1. `go build -o server.exe`
2. In the windows control panel go to:
   `Control Panel\All Control Panel Items\Windows Firewall\Allowed applications`
   and add the server.exe that was just created
3. `server.exe`


## C used in WebAssembly

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
  build the windows version (test.exe)
  load visual studio with: devenv build_win\test.exe
  Insert any breakpoints
  Press F11 to begin execution
  When exiting for the first time, save the MSDev solution to the build_win folder


