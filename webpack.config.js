//var webpack = require('webpack');
const path = require('path');

module.exports = {
  entry: {
    seni: ['./app/js/index.js'],
    worker: ['./app/js/worker.js']
  },
  output: {
    path: path.resolve(__dirname, 'app', 'dist'),
    filename: '[name].bundle.js',
    chunkFilename: '[id].bundle.js',
    sourceMapFilename: '[file].map'
  },
  module: {
    loaders: [
      {
        loader: 'babel-loader',

        // Only run `.js` files through Babel
        test: /\.js$/,

        // Skip any files outside of your project's `js` directory
        include: [
          path.resolve(__dirname, 'app', 'js')
        ]
      }
    ]
  },
  devtool: 'source-map'
};
