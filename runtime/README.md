# Stasis

The Stasis runtime. This module exports a default function that accepts a URL
to your WebAssembly binary. For more details on creating this binary, visit
[the github repo](https://github.com/overdrivenpotato/stasis).

```javascript
import load from 'stasis'

load('/_/app.wasm')
  .then(() => {
    console.log('The main function in app.wasm has finished running.')
  })
```

## I don't need any javascript code

This package also provides a minified bundle which can automatically run this
step for you. All this requires is a minimal html document:

```
<!DOCTYPE html>
<html>
    <head>
        <script
            id="stasis"
            src="http://bundle.run/stasis@0.1.0-alpha.4/dist/stasis.min.js"
            type="text/javascript"
            data-binary="THE_URL_TO_YOUR_BINARY_GOES_HERE"
        ></script>
    </head>
    <body></body>
</html>
```

Simply replace `THE_URL_TO_YOUR_BINARY_GOES_HERE` with the URL of your binary,
and open the HTML file. On Chrome, the file **must** be served via a webserver
or else the loader will receive an error. On Firefox, you can open the file
directly without a webserver.
