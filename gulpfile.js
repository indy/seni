var gulp = require('gulp');
var gulpPlugins = require('gulp-load-plugins')();
var runSequence = require('run-sequence');
var merge = require('merge');
var gulpTraceur = require('./tools/transpiler/gulp-traceur');

var clean = require('./tools/build/clean');
var deps = require('./tools/build/deps');
var transpile = require('./tools/build/transpile');
var html = require('./tools/build/html');
var jsserve = require('./tools/build/jsserve');

var karma = require('./tools/build/karma');

// -----------------------
// configuration

var _COMPILER_CONFIG_JS_DEFAULT = {
  sourceMaps: true,
  annotations: true, // parse annotations
  types: true, // parse types
  script: false, // parse as a module
  memberVariables: true, // parse class fields
  modules: 'instantiate'
};

var _HTLM_DEFAULT_SCRIPTS_JS = [
  {src: '/deps/traceur-runtime.js', mimeType: 'text/javascript'},
  {src: '/rtts_assert/lib/rtts_assert.js', mimeType: 'text/javascript'},
  {src: '/deps/es6-module-loader-sans-promises.src.js', mimeType: 'text/javascript'},
  {src: '/deps/zone.js', mimeType: 'text/javascript'},
  {src: '/deps/long-stack-trace-zone.js', mimeType: 'text/javascript'},
  {src: '/deps/system.src.js', mimeType: 'text/javascript'},
  {src: '/deps/extension-register.js', mimeType: 'text/javascript'},
  {src: '/deps/runtime_paths.js', mimeType: 'text/javascript'},
  {src: '/deps/gl-matrix.js', mimeType: 'text/javascript'},
  {
    inline: 'System.import(\'$MODULENAME$\').then(function(m) { m.main(); }, console.log.bind(console))',
    mimeType: 'text/javascript'
  }
];


var CONFIG = {
  dest: {
    js: {
      all: 'dist/js',
      dev: 'dist/js/dev',
      prod: 'dist/js/prod'
    }
  },
  srcFolderMapping: {
    'default': 'lib',
    '**/example*/**': 'web'
  },
  deps: {
    js: [
      gulpTraceur.RUNTIME_PATH,
      "node_modules/es6-module-loader/dist/es6-module-loader-sans-promises.src.js",
      "node_modules/systemjs/dist/system.src.js",
      "node_modules/systemjs/lib/extension-register.js",
      "node_modules/zone.js/zone.js",
      "node_modules/zone.js/long-stack-trace-zone.js",
      "tools/build/runtime_paths.js",
      "node_modules/gl-matrix/dist/gl-matrix.js"
    ]
  },
  transpile: {
    src: {
      js: ['app/**/*.js', 'app/**/*.es6']
    },
    copy: {
      js: ['app/**/*.es5']
    },
    options: {
      js: {
        dev: merge(true, _COMPILER_CONFIG_JS_DEFAULT, {
          typeAssertionModule: 'rtts_assert/rtts_assert',
          typeAssertions: true
        }),
        prod: merge(true, _COMPILER_CONFIG_JS_DEFAULT, {
          typeAssertions: false
        })
      }
    }
  },
  html: {
    src: {
      js: ['app/*/src/**/*.html']
    },
    scriptsPerFolder: {
      js: {
        default: _HTLM_DEFAULT_SCRIPTS_JS
      }
    }
  }
};

// ------------
// clean

gulp.task('build/clean.js', clean(gulp, gulpPlugins, {
  path: CONFIG.dest.js.all
}));


// ------------
// deps

gulp.task('build/deps.js.dev', deps(gulp, gulpPlugins, {
  src: CONFIG.deps.js,
  dest: CONFIG.dest.js.dev
}));

gulp.task('build/deps.js.prod', deps(gulp, gulpPlugins, {
  src: CONFIG.deps.js,
  dest: CONFIG.dest.js.prod
}));

// ------------
// transpile

gulp.task('build/transpile.js.dev', transpile(gulp, gulpPlugins, {
  src: CONFIG.transpile.src.js,
  copy: CONFIG.transpile.copy.js,
  dest: CONFIG.dest.js.dev,
  outputExt: 'js',
  options: CONFIG.transpile.options.js.dev,
  srcFolderMapping: CONFIG.srcFolderMapping
}));

gulp.task('build/transpile.js.prod', transpile(gulp, gulpPlugins, {
  src: CONFIG.transpile.src.js,
  copy: CONFIG.transpile.copy.js,
  dest: CONFIG.dest.js.prod,
  outputExt: 'js',
  options: CONFIG.transpile.options.js.prod,
  srcFolderMapping: CONFIG.srcFolderMapping
}));

// ------------
// html

gulp.task('build/html.js.dev', html(gulp, gulpPlugins, {
  src: CONFIG.html.src.js,
  dest: CONFIG.dest.js.dev,
  srcFolderMapping: CONFIG.srcFolderMapping,
  scriptsPerFolder: CONFIG.html.scriptsPerFolder.js
}));

gulp.task('build/html.js.prod', html(gulp, gulpPlugins, {
  src: CONFIG.html.src.js,
  dest: CONFIG.dest.js.prod,
  srcFolderMapping: CONFIG.srcFolderMapping,
  scriptsPerFolder: CONFIG.html.scriptsPerFolder.js
}));


// ------------------
// web servers
gulp.task('serve.js.dev', jsserve(gulp, gulpPlugins, {
  path: CONFIG.dest.js.dev
}));

gulp.task('serve.js.prod', jsserve(gulp, gulpPlugins, {
  path: CONFIG.dest.js.prod
}));

// --------------
// doc generation
var Dgeni = require('dgeni');
gulp.task('docs/dgeni', function() {
  try {
    var dgeni = new Dgeni([require('./docs/dgeni-package')]);
    return dgeni.generate();
  } catch(x) {
    console.log(x.stack);
    throw x;
  }
});

var bower = require('bower');
gulp.task('docs/bower', function() {
  var bowerTask = bower.commands.install(undefined, undefined, { cwd: 'docs' });
  bowerTask.on('log', function (result) {
    console.log('bower:', result.id, result.data.endpoint.name);
  });
  bowerTask.on('error', function(error) {
    console.log(error);
  });
  return bowerTask;
});

gulp.task('docs/assets', ['docs/bower'], function() {
  return gulp.src('docs/bower_components/**/*')
    .pipe(gulp.dest('dist/docs/lib'));
});

gulp.task('docs/app', function() {
  return gulp.src('docs/app/**/*')
    .pipe(gulp.dest('dist/docs'));
});

gulp.task('docs', ['docs/assets', 'docs/app', 'docs/dgeni']);
gulp.task('docs/watch', function() {
  return gulp.watch('docs/app/**/*', ['docs/app']);
});

var jasmine = require('gulp-jasmine');
gulp.task('docs/test', function () {
  return gulp.src('docs/**/*.spec.js')
      .pipe(jasmine({
        includeStackTrace: true
      }));
});

var webserver = require('gulp-webserver');
gulp.task('docs/serve', function() {
  gulp.src('dist/docs/')
    .pipe(webserver({
      fallback: 'index.html'
    }));
});


gulp.task('test', function(done) {
    var options = {
        configFile: 'karma.conf.js'
    };
    for (var i=0, ii = process.argv.length; i<ii; ++i) {
        var val = process.argv[i];
        if (val === '--debug') options.debugRun = true;
        if (val === '--watch') options.autoWatch = true;
        else if (val === '--single-run') options.singleRun = true;
        else if (val === '--browsers') options.browsers = process.argv[++i].split(',');
    }
    karma(options, done);
});

// -----------------
// orchestrated targets

gulp.task('build.js.dev', function() {
  return runSequence(
    ['build/deps.js.dev', 'build/transpile.js.dev', 'build/html.js.dev']);
});

gulp.task('build.js.prod', function() {
  return runSequence(
    ['build/deps.js.prod', 'build/transpile.js.prod', 'build/html.js.prod']);
});

gulp.task('build.js', ['build.js.dev', 'build.js.prod']);

gulp.task('clean', ['build/clean.js']);

gulp.task('build', ['build.js']);



