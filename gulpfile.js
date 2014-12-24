var gulp = require('gulp');
var gulpPlugins = require('gulp-load-plugins')();
var runSequence = require('run-sequence');
var merge = require('merge');
var gulpTraceur = require('./tools/transpiler/gulp-traceur');

var clean = require('./tools/build/clean');
var deps = require('./tools/build/deps');
var css = require('./tools/build/css');
var transpile = require('./tools/build/transpile');
var html = require('./tools/build/html');
var jsserve = require('./tools/build/jsserve');

var karma = require('./tools/build/karma');
var path = require('path');

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
  {src: '/js/deps/traceur-runtime.js', mimeType: 'text/javascript'},
  {src: '/js/rtts_assert/lib/rtts_assert.js', mimeType: 'text/javascript'},
  {src: '/js/deps/es6-module-loader-sans-promises.src.js', mimeType: 'text/javascript'},
  {src: '/js/deps/system.src.js', mimeType: 'text/javascript'},
  {src: '/js/deps/extension-register.js', mimeType: 'text/javascript'},
  {src: '/js/deps/runtime_paths.js', mimeType: 'text/javascript'},
  {src: '/js/deps/gl-matrix.js', mimeType: 'text/javascript'},
  {
    inline: 'System.import(\'$MODULENAME$\').then(function(m) { m.main(); }, console.log.bind(console))',
    mimeType: 'text/javascript'
  }
];


var CONFIG = {
  dest: {
    js: {
      all: 'dist',
      dev: 'dist/dev',
      prod: 'dist/prod'
    }
  },
  srcFolderMapping: {
    'default': 'lib'
  },
  deps: {
    js: [
      gulpTraceur.RUNTIME_PATH,
      "node_modules/es6-module-loader/dist/es6-module-loader-sans-promises.src.js",
      "node_modules/systemjs/dist/system.src.js",
      "node_modules/systemjs/lib/extension-register.js",
      "tools/build/runtime_paths.js",
      "node_modules/gl-matrix/dist/gl-matrix.js"
    ]
  },
  transpile: {
    src: {
      js: ['app/js/**/*.js', 'app/js/**/*.es6']
    },
    copy: {
      js: ['app/js/**/*.es5']
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
  css: {
    src: 'app/css/*.css'
  },
  html: {
    src: {
      js: ['app/*/src/**/*.html', 'app/*.html']
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
// css

gulp.task('build/css.js.dev', css(gulp, gulpPlugins, {
  src: CONFIG.css.src,
  dest: CONFIG.dest.js.dev
}));

gulp.task('build/css.js.prod', css(gulp, gulpPlugins, {
  src: CONFIG.css.src,
  dest: CONFIG.dest.js.prod
}));

// ------------
// deps

gulp.task('build/deps.js.dev', deps(gulp, gulpPlugins, {
  src: CONFIG.deps.js,
  dest: path.join(CONFIG.dest.js.dev, 'js')
}));

gulp.task('build/deps.js.prod', deps(gulp, gulpPlugins, {
  src: CONFIG.deps.js,
  dest: path.join(CONFIG.dest.js.prod, 'js')
}));

// ------------
// transpile

gulp.task('build/transpile.js.dev', transpile(gulp, gulpPlugins, {
  src: CONFIG.transpile.src.js,
  copy: CONFIG.transpile.copy.js,
  dest: path.join(CONFIG.dest.js.dev, 'js'),
  outputExt: 'js',
  options: CONFIG.transpile.options.js.dev,
  srcFolderMapping: CONFIG.srcFolderMapping
}));

gulp.task('build/transpile.js.prod', transpile(gulp, gulpPlugins, {
  src: CONFIG.transpile.src.js,
  copy: CONFIG.transpile.copy.js,
  dest: path.join(CONFIG.dest.js.prod, 'js'),
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
    ['build/deps.js.dev',
     'build/transpile.js.dev',
     'build/html.js.dev',
     'build/css.js.dev']);
});

gulp.task('build.js.prod', function() {
  return runSequence(
    ['build/deps.js.prod',
     'build/transpile.js.prod',
     'build/html.js.prod',
     'build/css.js.prod']);
});

gulp.task('build.js', ['build.js.dev', 'build.js.prod']);

gulp.task('clean', ['build/clean.js']);

gulp.task('build', ['build.js']);



