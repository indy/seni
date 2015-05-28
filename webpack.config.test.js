var webpack = require("webpack");
var path = require('path');

module.exports = {
  output: {
    sourceMapFilename: '[file].map'
  },
  module: {
    preLoaders: [
      {
        test: /\.js$/,
        loader: "eslint-loader", exclude: /node_modules/
      }
    ],

    loaders: [
      {
        test: path.join(__dirname, 'src'),
        loader: 'babel-loader'
      },
      {
        test: path.join(__dirname, 'test'),
        loader: 'babel-loader'
      }
    ]
  },
  devtool: 'source-map'
};