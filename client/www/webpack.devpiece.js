const webpack = require('webpack');
const merge = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
  mode: 'development',
  entry: {
    piece: ['./src/piece.js'],
    worker: ['./src/worker.js']
  },
  plugins: [
    new webpack.DefinePlugin({
      LOAD_FOR_SENI_APP_GALLERY: false,
      WASM_FILE_URI: JSON.stringify('seni-wasm.wasm')
    })
  ],
  devtool: 'source-map'
});
