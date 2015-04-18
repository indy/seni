var webpack = require("webpack");
var path = require('path');

module.exports = {
  module: {
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
  }
};
