import Wrapper from './wrapper'
import Binary from './binary'
import patchExport from './patch-export'

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
  new Promise(resolve => {
    const el = document.createElement('script')

    el.onload = () => {
      resolve()
    }

    el.src = src
    document.head.appendChild(el)
  })
)

const FORCE_INTERPRETER = false

const getWebAssembly = async (): Promise<any> => {
  const native = (window as any).WebAssembly

  if (native && !FORCE_INTERPRETER) {
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

const stasisCall = (getHandle: () => Handle | null) =>
  (op: number, a: number, b: number): number => {
    const handle = getHandle()

    if (!handle) {
      return -1
    }

    const opcodes = {
      REGISTER_STASIS_CB: 0,
      CREATE_MODULE: 1,
      REGISTER_FN: 2,
      REGISTER_CB: 3,
      CALL_FN: 4,
    }

    switch (op) {
      case opcodes.REGISTER_STASIS_CB: {
        handle.binary.setCallbackPointer(a)
        return 0
      }

      case opcodes.CREATE_MODULE: {
        return handle.wrapper.createModule()
      }

      case opcodes.REGISTER_FN: {
        interface Register {
          id: number
          name: string
          code: string
        }

        const json: Register = handle.binary.getJson(a, b)

        handle
          .wrapper
          .getModule(json.id)
          .register(json.name, json.code)

        return 0
      }

      case opcodes.REGISTER_CB: {
        interface RegisterCallback {
          module: number
          callback: number
          name: string
        }

        const json: RegisterCallback = handle.binary.getJson(a, b)

        handle
          .wrapper
          .getModule(json.module)
          .registerCallback(json.name, json.callback)

        return 0
      }

      case opcodes.CALL_FN: {
        interface Call {
          id: number
          name: string
          args: any
        }

        const call: Call = handle.binary.getJson(a, b)

        const ret = handle.wrapper
          .getModule(call.id)
          .call(call.name, call.args)

        return handle.binary.makePair(ret)
      }

      default: return -2
    }
  }

export default async (url: string): Promise<void> => {
  const bytes = patchExport(await getBytes(url))

  let handle: null | Handle = null

  const env = {
    __stasis_call: stasisCall(() => handle),
  }

  const WebAssembly = await getWebAssembly()
  const wasm = await WebAssembly.instantiate(bytes, { env })

  const binary = new Binary(wasm.instance.exports)
  const wrapper = new Wrapper(binary)
  handle = ({ wrapper, binary })
  binary.main()
}
