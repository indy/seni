var gulp = require('gulp'),
    pipe = require('pipe/gulp'),
    karma = require('./lib/gulp/karma'),
    traceur = require('gulp-traceur'),
    jshint = require('gulp-jshint');

var webserver = require('gulp-webserver');

var paths = {
  src: 'src/**/*.js'
};

gulp.task('webserver', function() {
  gulp.src('.')
    .pipe(webserver({
//      livereload: true,
      directoryListing: true,
//      open: true
    }));
});

gulp.task('lint', function() {
  gulp.src(paths.src)
  .pipe(jshint())
  .pipe(jshint.reporter('jshint-stylish'))
  .pipe(jshint.reporter('fail'));
});

gulp.task('build:amd', function() {
  gulp.src(paths.src)
    .pipe(traceur(pipe.traceur({experimental: true})))
  .pipe(gulp.dest('dist/amd'));
});

gulp.task('build:cjs', function() {
  gulp.src(paths.src)
    .pipe(traceur(pipe.traceur({modules: 'commonjs',
                               experimental: true})))
  .pipe(gulp.dest('dist/cjs'));
});

gulp.task('build:es6', function() {
  gulp.src(paths.src)
    .pipe(traceur(pipe.traceur({outputLanguage: 'es6',
                               experimental: true})))
  .pipe(gulp.dest('dist/es6'));
});

// todo: add lint as the first subtask of build
gulp.task('build', ['build:amd', 'build:cjs', 'build:es6']);

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
