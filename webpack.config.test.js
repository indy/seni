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
        loader: "babel-loader",

        test: path.join(__dirname, 'src'),

        // Options to configure babel with
        query: {
          plugins: ['transform-runtime'],
          presets: ['es2015']
        }
      },
      {
        loader: "babel-loader",

        test: path.join(__dirname, 'test'),

        // Options to configure babel with
        query: {
          plugins: ['transform-runtime'],
          presets: ['es2015']
        }
      }
    ]
  },
  devtool: 'source-map'
};
