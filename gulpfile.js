var gulp = require('gulp');
var pipe = require('pipe/gulp');
var traceur = require('gulp-traceur');
var jshint = require('gulp-jshint');
var browserify = require('browserify');
var source = require('vinyl-source-stream');
var webserver = require('gulp-webserver');

var karma = require('./tools/gulp/karma');

var paths = {
    src: 'src/**/*.js'
};

gulp.task('webserver', function() {
    gulp.src('.')
        .pipe(webserver({
            //      livereload: true,
            directoryListing: true,
            open: "http://localhost:8000/index.cjs.html"
        }));
});

gulp.task('lint', function() {
    gulp.src(paths.src)
        .pipe(jshint())
        .pipe(jshint.reporter('jshint-stylish'))
        .pipe(jshint.reporter('fail'));
});

gulp.task('build:cjs', function() {
    return gulp.src(paths.src)
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

gulp.task('browserify', ['build:cjs'], function() {
    return browserify('./dist/cjs/main.js')
        .bundle()
        .pipe(source('bundle.js'))
        .pipe(gulp.dest('./dist/cjs/'));
});

// todo: add lint as the first subtask of build
gulp.task('build', ['build:cjs', 'build:es6', 'browserify']);

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
