import Wrapper from './wrapper'
import { Pointer } from './types'

class Handle {
  private wrapper?: Wrapper

  constructor() {}

  public get(): undefined | Wrapper {
    return this.wrapper
  }

  public set(wrapper: Wrapper) {
    this.wrapper = wrapper
  }
}

const load = async (url: string): Promise<void> => {
  const response = await fetch(url)
  const bytes = await response.arrayBuffer()

  const handle = new Handle()

  const env = {
    __stasis_module_create(): number {
      const wrapper = handle.get()
      if (!wrapper) return -1

      return wrapper.createModule()
    },
    __stasis_register(ptr: Pointer, len: number) {
      const wrapper = handle.get()
      if (!wrapper) return

      interface Register {
        id: number
        name: string
        code: string
      }

      const json: Register = wrapper.json(ptr, len)

      wrapper
        .getModule(json.id)
        .register(json.name, json.code)
    },
    __stasis_call(ptr: Pointer, len: number): Pointer {
      const wrapper = handle.get()
      if (!wrapper) return 0

      interface Call {
        id: number
        name: string
        args: any
      }

      const call: Call = wrapper.json(ptr, len)

      const ret = wrapper
        .getModule(call.id)
        .call(call.name, call.args)

      return wrapper.pair(ret)
    },
  }

  const WebAssembly: any = (window as any).WebAssembly
  const wasm = await WebAssembly.instantiate(bytes, { env })

  const wrapper = new Wrapper(wasm.instance.exports)
  handle.set(wrapper)
  wrapper.main()
}

export default load
