import load from './index'

const warn = (...args: Array<any>) => {
  console.warn(
    ...args,
    '\n',
    'See github.com/overdrivenpotato/stasis for instructions.'
  )
}

const el = document.getElementById('stasis')

if (!el) {
  warn('Stasis loaded but element with id="stasis" not found.')
} else {
  const binary = el.getAttribute('data-binary')

  if (!binary) {
    warn('Stasis loaded but data-binary attribute is not present on:\n', el)
  } else {
    try {
      load(binary)
    } catch(e) {
      console.error('Failed to load Stasis module:\n', e)
    }
  }
}