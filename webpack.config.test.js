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
        loader: "babel",

        test: path.join(__dirname, 'app', 'src'),

        // Options to configure babel with
        query: {
          presets: ['es2015']
        }
      },
      {
        loader: "babel",

        test: path.join(__dirname, 'test'),

        // Options to configure babel with
        query: {
          presets: ['es2015']
        }
      }
    ]
  },
  devtool: 'source-map'
};
