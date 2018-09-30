//var webpack = require('webpack');
const path = require('path');

module.exports = [
  {
    mode: "development",
    entry: {
      seni: ['./src/js/index.js'],
      worker: ['./src/js/worker.js'],
      piece: ['./src/js/piece.js']
    },
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: '[name].bundle.js',
      chunkFilename: '[id].bundle.js',
      sourceMapFilename: '[file].map'
    },
    module: {
      rules: [
        {
          loader: 'babel-loader',

          // Only run `.js` files through Babel
          test: /\.js$/,

          // Skip any files outside of your project's `js` directory
          include: [
            path.resolve(__dirname, 'src', 'js')
          ]
        }
      ]
    },
    devtool: 'source-map'
  }
];
