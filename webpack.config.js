var webpack = require("webpack");
var path = require('path');

module.exports = function(filename) {
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


//   plugins: [new webpack.optimize.UglifyJsPlugin()]
