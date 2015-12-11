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
        test: /\.js$/,
        loader: 'eslint-loader', exclude: /node_modules/
      }
    ],
    loaders: [
      {
        loader: 'babel',

        // Skip any files outside of your project's `src` directory
        include: [
          path.resolve(__dirname, 'app', 'src'),
        ],

        // Only run `.js` files through Babel
        test: /\.js$/,

        // Options to configure babel with
        query: {
          presets: ['es2015']
        }
      }
    ]
  },
  devtool: 'source-map'
};
