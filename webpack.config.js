var webpack = require("webpack");
var path = require('path');

module.exports = {
  entry: {
    seni: ['babel-polyfill', './src/index.js']
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
        loader: "babel-loader",

        // Skip any files outside of your project's `src` directory
        include: [
          path.resolve(__dirname, "src"),
        ],

        // Only run `.js` and `.jsx` files through Babel
        test: /\.jsx?$/,

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
