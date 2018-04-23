import Wrapper from './wrapper'
import Binary from './binary'
import { Pointer } from './types'

interface Handle {
  wrapper: Wrapper
  binary: Binary
}

const load = async (url: string): Promise<void> => {
  const response = await fetch(url)
  const bytes = await response.arrayBuffer()

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

  const WebAssembly: any = (window as any).WebAssembly
  const wasm = await WebAssembly.instantiate(bytes, { env })

  const binary = new Binary(wasm.instance.exports)
  const wrapper = new Wrapper(binary)
  getHandle = () => ({ wrapper, binary })
  binary.main()
}

export default load
