const webpack = require('webpack');
const merge = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
  mode: 'development',
  entry: {
    seni: ['./src/js/index.js'],
    worker: ['./src/js/worker.js']
  },
  plugins: [
    new webpack.DefinePlugin({
      WASM_FILE_URI: JSON.stringify('seni-wasm.wasm')
    })
  ],
  devtool: 'source-map'
});
