var common = require('./webpack.config.js');
var path = require('path');

module.exports = {
  output: {
    sourceMapFilename: '[file].map'
  },
  module: {
    preLoaders: common.module.preLoaders,
    loaders: common.module.loaders.concat([
      {
        loader: "babel",
        test: path.join(__dirname, 'test')
      }
    ])
  }
};
