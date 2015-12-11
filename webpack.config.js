var webpack = require('webpack');
var path = require('path');

module.exports = {
  entry: {
    seni: ['./app/src/index.js']
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

        // Skip any files outside of your project's `src` directory
        include: [
          path.resolve(__dirname, 'app', 'src'),
        ],

        // Options to configure babel with
        query: {
          presets: ['es2015']
        }
      }
    ]/*,
    postLoaders: [
      {
        loader: 'istanbul-instrumenter',
        test: /\.js$/,

        include: path.resolve(__dirname, 'app', 'src')

      }
    ]*/
  },
  devtool: 'source-map'
};
