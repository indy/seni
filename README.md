# Seni

Seni is a Scheme-like graphical language that runs on modern web browsers.

It's scripts can be annotated so that genetic algorithms can generate variations and the user can select which of the generated images should be used in future generations.

## Build

### Prerequisites:

1. rust and node.js should be installed

### Build:

#### Building the server:

1. `cd server`
2. `cargo build --release`'

#### Running the server:

1. `cd server`
2. `cargo run --release`

#### Building wasm file:

1. `cd client`
2. `npm run build:wasm`

#### Building app for development:

1. `cd client/www`
2. `npm run build:js`

#### Building app for production:

1. `cd client/www`
2. `npm run build:js:production`

#### Building for single piece gallery page on https://seni.app:

1. `cd client/www`
2. `npm run build:js:piece`
3. copy files from `dist` into appropriate folder of seni.app repo

#### Building when developing single piece gallery page for https://seni.app:

1. `cd client/www`
2. `npm run build:js:devpiece`
3. start the server
4. visit http://localhost:3210/piece.html

#### Starting the server

1. `./seni-server`
2. visit http://localhost:3210

### Test:

* compiling natively and running tests on Linux/MacOS:

`cd core`
`./test_unix.sh`

* compiling natively and running tests on Windows

`cd core`
`./test_win.bat`

### Publishing

1. `npm run publish:local`
2. upload the app directory onto a server

## Windows 10 Shenanigans

You will need to build a seni-server.exe and add it to the Firewall whitelist in order to prevent a Windows Firewall dialog appearing every time.

1. Build the server using the instructions above
2. In the windows control panel go to:
   `Control Panel\All Control Panel Items\Windows Firewall\Allowed applications`
   and add the seni-server.exe that was just created
3. `seni-server.exe`

## C used in WebAssembly

Keep things simple by using C.
Avoid C++, even trying to use it as a 'nicer C' results in increased compilation times, obtuse error messages, complicates the build/link commands and reduces platform independance.
Switch to Rust if you need a better C

test the output by opening build_wasm\seni-wasm.html in a browser that supports WebAssembly

* visual studio integration with the native windows build
  build the windows version (test.exe)
  load visual studio with: devenv build\test.exe
  Insert any breakpoints
  Press F11 to begin execution
  When exiting for the first time, save the MSDev solution
