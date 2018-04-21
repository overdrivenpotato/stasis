const path = require('path')

module.exports = {
  mode: 'production',
  entry: path.resolve(__dirname, 'dist/bootstrap.js'),
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'stasis.min.js',
  },
}
