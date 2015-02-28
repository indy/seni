

babel-library-boilerplate



## Build

### Prerequisites:

1. `npm install`
2. `npm install -g gulp karma karma-cli`
3. `bower install`
4. `gulp symlink.ext`
5. `gulp build`

### Folder structure

* `modules/*`: modules that will be loaded in the browser
* `tools/*`: tools that are needed to build Angular

### Build:

1. `gulp build` -> result is in `dist` folder

2. `gulp clean` -> cleans the `dist` folder
   (run `gulp cleanbuild` afterwards)

### Tests:

1. `gulp test`: JS tests

Notes for all tests:

The karma preprocessor is setup in a way so that after every test run
the transpiler is reloaded. With that it is possible to make changes
to the preprocessor and run the tests without exiting karma
(just touch a test file that you would like to run).

### Dev

1. `gulp watch`: Watches `app` folder for changes to js, html or css files
2. `gulp serve`
3. open http://localhost:8000

## Debug the transpiler

If you need to debug the transpiler:

- add a `debugger;` statement in the transpiler code,
- from the root folder, execute `node debug node_modules/.bin/gulp build` to enter the node
  debugger
- press "c" to execute the program until you reach the `debugger;` statement,
- you can then type "repl" to enter the REPL and inspect variables in the context.

See the [Node.js manual](http://nodejs.org/api/debugger.html) for more information.

Notes:
- You can also add `debugger;` statements in the specs (JavaScript). The execution will halt when
  the developer tools are opened in the browser running Karma.

## Debug the tests

If you need to debug the tests:

- add a `debugger;` statement to the test you want to debug (or the source code),
- execute karma `node_modules/karma/bin/karma start karma-js.conf.js`,
- press the top right "DEBUG" button,
- open the dev tools and press F5,
- the execution halt at the `debugger;` statement

Note (WebStorm users):
You can create a Karma run config from WebStorm.
Then in the "Run" menu, press "Debug 'karma-js.conf.js'", WebStorm will stop in the generated code
on the `debugger;` statement.
You can then step into the code and add watches.
The `debugger;` statement is needed because WebStorm will stop in a transpiled file. Breakpoints in
the original source files are not supported at the moment.
