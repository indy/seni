# Senie

Senie is a Scheme-like graphical language that runs on modern web browsers.

It's scripts can be annotated so that genetic algorithms can generate variations and the user can select which of the generated images should be used in future generations.

## Build

### Prerequisites:

1. go and node.js should be installed
2. `npm install`

### Build:

1. `npm run build`
2. `npm run build:wasm`
3. `npm run build:server`

### Test:

* compiling natively and running tests on Linux/MacOS:

./make_unix.sh test

* compiling natively and running tests on Windows

make_win.bat test


### Running

1. `./serve`
2. visit http://localhost:3000

## Windows 10 Shenanigans

You will need to build a server.exe and add it to the Firewall whitelist in order to prevent a Windows Firewall dialog appearing every time.

1. `go build src/go/serve.go`
2. In the windows control panel go to:
   `Control Panel\All Control Panel Items\Windows Firewall\Allowed applications`
   and add the serve.exe that was just created
3. `serve.exe`

## C used in WebAssembly

Keep things simple by using C.
Avoid C++, even trying to use it as a 'nicer C' results in increased compilation times, obtuse error messages, complicates the build/link commands and reduces platform independance.
Switch to Rust if you need a better C

test the output by opening build_wasm\senie-wasm.html in a browser that supports WebAssembly

* visual studio integration with the native windows build
  build the windows version (test.exe)
  load visual studio with: devenv build_win\test.exe
  Insert any breakpoints
  Press F11 to begin execution
  When exiting for the first time, save the MSDev solution to the build_win folder
