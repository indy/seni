var webpack = require("webpack");
var path = require('path');

module.exports = {
  entry: {
    seni: './src/index.js'
  },
  output: {
    path: __dirname,
    filename: "./dist/[name].bundle.js",
    chunkFilename: "./dist/[id].bundle.js",
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
      }
    ]
  },
  devtool: 'source-map'
};
