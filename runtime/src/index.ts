import Wrapper from './wrapper'
import Binary from './binary'
import { Pointer } from './types'

interface Handle {
  wrapper: Wrapper
  binary: Binary
}

const getBytes = (url: string): Promise<Uint8Array> => (
  new Promise((resolve, reject) => {
    try {
      const xhr = new XMLHttpRequest()

      xhr.onreadystatechange = () => {
        if (xhr.readyState === 4) {
          resolve(new Uint8Array(xhr.response))
        }
      }

      xhr.responseType = 'arraybuffer'
      xhr.open('GET', url)
      xhr.send()
    } catch(e) {
      reject(e)
    }
  })
)

// Load a new JS script.
const loadScript = (src: string): Promise<void> => (
  new Promise((resolve, _reject) => {
    const el = document.createElement('script')

    el.onload = () => {
      resolve()
    }

    el.src = src
    document.head.appendChild(el)
  })
)

const getWebAssembly = async (): Promise<any> => {
  const native = (window as any).WebAssembly

  if (native) {
    return Promise.resolve(native)
  } else {
    console.warn(
      'Stasis: attempting to use fallback webassembly interpreter.\n' +
      'It is still in development, this will most likely fail for now.'
    )

    // This is most likely going to fail.
    await loadScript('https://bundle.run/webassemblyjs')
    return (window as any).webassemblyjs
  }
}

export default async (url: string): Promise<void> => {
  const bytes = await getBytes(url)

  let getHandle: () => Handle | null = () => null

  const env = {
    __stasis_module_create(): number {
      const handle = getHandle()
      if (!handle) return -1

      return handle.wrapper.createModule()
    },
    __stasis_register(ptr: Pointer, len: number) {
      const handle = getHandle()
      if (!handle) return

      interface Register {
        id: number
        name: string
        code: string
      }

      const json: Register = handle.binary.getJson(ptr, len)

      handle.wrapper
        .getModule(json.id)
        .register(json.name, json.code)
    },
    __stasis_register_callback(ptr: Pointer, len: number) {
      const handle = getHandle()
      if (!handle) return

      interface RegisterCallback {
        module: number
        callback: number
        name: string
      }

      const json: RegisterCallback = handle.binary.getJson(ptr, len)

      handle.wrapper
        .getModule(json.module)
        .registerCallback(json.name, json.callback)
    },
    __stasis_call(ptr: Pointer, len: number): Pointer {
      const handle = getHandle()
      if (!handle) return 0

      interface Call {
        id: number
        name: string
        args: any
      }

      const call: Call = handle.binary.getJson(ptr, len)

      const ret = handle.wrapper
        .getModule(call.id)
        .call(call.name, call.args)

      return handle.binary.makePair(ret)
    },
  }

  const WebAssembly = await getWebAssembly()
  const wasm = await WebAssembly.instantiate(bytes, { env })

  const binary = new Binary(wasm.instance.exports)
  const wrapper = new Wrapper(binary)
  getHandle = () => ({ wrapper, binary })
  binary.main()
}
