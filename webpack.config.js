var webpack = require('webpack');
var path = require('path');

module.exports = {
  entry: {
    seni: ['./app/js/index.js']
    // why have polyfill?
    // ,polyfill: ['babel-polyfill']
  },
  output: {
    path: path.resolve(__dirname, 'app', 'dist'),
    filename: '[name].bundle.js',
    chunkFilename: '[id].bundle.js',
    sourceMapFilename: '[file].map'
  },
  module: {
    preLoaders: [
      {
        loader: 'eslint-loader',
        test: /\.js$/,

        exclude: /node_modules/
      }
    ],
    loaders: [
      {
        loader: 'babel',

        // Only run `.js` files through Babel
        test: /\.js$/,

        // Skip any files outside of your project's `js` directory
        include: [
          path.resolve(__dirname, 'app', 'js'),
        ],

        // Options to configure babel with
        query: {
          plugins: ["transform-es2015-arrow-functions",
                    "transform-es2015-classes",
                    "transform-es2015-object-super",
                    "transform-es2015-parameters",
                    "transform-es2015-destructuring",
                    "transform-es2015-block-scoping",
                    "transform-es2015-modules-commonjs"]
        }
      }
    ]/*,
      postLoaders: [
      {
      loader: 'istanbul-instrumenter',
      test: /\.js$/,

      include: path.resolve(__dirname, 'app', 'js')

      }
      ]*/
  },
  devtool: 'source-map'
};
