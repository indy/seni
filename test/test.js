(function() {
  var allTestFiles = [];
  var TEST_REGEXP = /\.spec\.js$/;

  var pathToModule = function(path) {
    return path.replace(/^\/base\//, '').replace(/\.js$/, '');
  };

  Object.keys(window.__karma__.files).forEach(function(file) {
    if (TEST_REGEXP.test(file)) {
      
      console.log("before: " + file);
      console.log("after: " + pathToModule(file));
      // Normalize paths to RequireJS module names.
      allTestFiles.push(pathToModule(file));
    }
  });

  require.config({
    // Karma serves files under /base, which is the basePath from your config file

    // isg: http://karma-runner.github.io/0.12/plus/requirejs.html
    // isg: try to change to base url of src/main.js so that relative
    // isg: requires in the source won't need to change.
    baseUrl: '/base',

    paths: {
      'rtts-assert': './node_modules/rtts-assert/src/assert'
    },

    // Dynamically load all test files and ES6 polyfill.
    deps: allTestFiles.concat(['node_modules/es6-shim/es6-shim', 'test/matchers']),

    // we have to kickoff jasmine, as it is asynchronous
    callback: window.__karma__.start
  });
})();
