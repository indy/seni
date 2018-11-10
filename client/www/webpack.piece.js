const webpack = require('webpack');
const merge = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
  mode: 'production',
  entry: {
    piece: ['./src/piece.js'],
    worker: ['./src/worker.js']
  },
  plugins: [
    new webpack.DefinePlugin({
      LOAD_FOR_SENI_APP_GALLERY: true,
      WASM_FILE_URI: JSON.stringify('/seni/seni-wasm.wasm')
    })
  ]
});
