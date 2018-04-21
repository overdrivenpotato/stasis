const path = require('path')

module.exports = {
  mode: 'production',
  entry: path.resolve(__dirname, 'dist/index.js'),
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bootstrap.js',
  },
}
