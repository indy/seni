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
