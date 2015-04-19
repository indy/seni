/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

var gulp = require('gulp');
var $ = require('gulp-load-plugins')();
const fs = require('fs');
const del = require('del');
const path = require('path');
const isparta = require('isparta');
const runSequence = require('run-sequence');
const eslint = require('gulp-eslint');
const webpack = require('gulp-webpack');

const manifest = require('./package.json');
const config = manifest.seniOptions;
const mainFile = manifest.main;
const destinationFolder = path.dirname(mainFile);
const exportFileName = path.basename(mainFile, path.extname(mainFile));


gulp.task('eslint-src', function () {
    return gulp.src(['src/**/*.js'])
        // eslint() attaches the lint output to the eslint property
        // of the file object so it can be used by other modules.
        .pipe(eslint())
        // eslint.format() outputs the lint results to the console.
        // Alternatively use eslint.formatEach() (see Docs).
        .pipe(eslint.format())
        // To have the process exit with an error code (1) on
        // lint error, return the stream and pipe to failOnError last.
        .pipe(eslint.failOnError());
});

gulp.task('eslint-test', function () {
    return gulp.src(['test/**/*.js'])
        // eslint() attaches the lint output to the eslint property
        // of the file object so it can be used by other modules.
        .pipe(eslint())
        // eslint.format() outputs the lint results to the console.
        // Alternatively use eslint.formatEach() (see Docs).
        .pipe(eslint.format())
        // To have the process exit with an error code (1) on
        // lint error, return the stream and pipe to failOnError last.
        .pipe(eslint.failOnError());
});


// Remove the built files
gulp.task('clean', function(cb) {
  del([destinationFolder], cb);
});

// Remove our temporary files
gulp.task('clean-tmp', function(cb) {
  del(['tmp'], cb);
});

function webpackConfig(filename) {
  return {
    entry: './src/index.js',
    output: {
      path: __dirname,
      filename: filename,
      sourceMapFilename: '[file].map'
    },
    module: {
      loaders: [
        {
          test: path.join(__dirname, 'src'),
          loader: 'babel-loader'
        }
      ]
    },
    devtool: 'source-map'
  };
}

// Build two versions of the library
gulp.task('build', ['eslint-src', 'clean'], function(done) {
  var configs = webpackConfig(exportFileName + '.js');
  return gulp.src('src/**/*.js')
    .pipe(webpack(configs))
    .pipe(gulp.dest(destinationFolder));
});

gulp.task('coverage', function(done) {
  require('babel/register')({ modules: 'common' });
  gulp.src(['src/**/*.js'])
    .pipe($.plumber())
    .pipe($.istanbul({ instrumenter: isparta.Instrumenter }))
    .pipe($.istanbul.hookRequire())
    .on('finish', function() {
      return test()
      .pipe($.istanbul.writeReports())
      .on('end', done);
    });
});

function test() {
  return gulp.src(['test/setup/node.js', 'test/unit/**/*.js'], {read: false})
    .pipe($.plumber())
    .pipe($.mocha({reporter: 'dot', globals: config.mochaGlobals}));
};

// Lint and run our tests
gulp.task('test', ['eslint-src', 'eslint-test'], function() {
  require('babel/register')({ modules: 'common' });
  return test();
});

// Run the headless unit tests as you make changes.
gulp.task('watch', function() {
  gulp.watch(['src/**/*', 'test/**/*', '.eslintrc'], ['test']);
});

// Build as you make changes.
gulp.task('build-watch', function() {
  gulp.watch(['src/**/*'], ['build']);
});

// An alias of test
gulp.task('default', ['test']);
