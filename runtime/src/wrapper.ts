import Module from './module'
import { Pointer, Map } from './types'

export interface Exports {
  memory: { buffer: ArrayBuffer }
  __stasis_alloc(size: number): Pointer
  __stasis_dealloc(p: Pointer, len: number): void
  __stasis_entrypoint(): void
}

export default class Wrapper {
  private exports: Exports
  private modules: Map<Module>
  private counter: number

  constructor(exports: Exports) {
    this.exports = exports
    this.modules = {}
    this.counter = 0
  }

  public mem(): Uint8Array {
    return new Uint8Array(this.exports.memory.buffer)
  }

  public getString(ptr: Pointer, len: number): string {
    const mem = this.mem()

    let s = ''

    for (let i = 0; i < len; i++) {
      s += String.fromCharCode(mem[ptr + i])
    }

    this.dealloc(ptr, len)

    return s
  }

  public json(ptr: Pointer, len: number): any {
    const text = this.getString(ptr, len)

    try {
      return JSON.parse(text)
    } catch(e) {
      // Don't deallocate here. This should never error in practice, if this
      // branch is reached it is indicative of memory corruption. There's not
      // much else we can do.
      return null
    }
  }

  public main() {
    this.exports.__stasis_entrypoint()
  }

  public createModule(): number {
    this.modules[++this.counter] = new Module()

    return this.counter
  }

  public getModule(id: number): Module {
    // TODO: This may actually be null.
    return this.modules[id]!
  }

  public dealloc(ptr: Pointer, len: number) {
    this.exports.__stasis_dealloc(ptr, len)
  }

  public alloc(size: number): Pointer {
    return this.exports.__stasis_alloc(size)
  }

  public pair(obj: any): Pointer {
    if (obj === undefined) {
      return 0
    }

    const json = JSON.stringify(obj)

    const mem = this.mem()
    const len = json.length
    const ptr = this.alloc(json.length)

    for (let i = 0; i < len; i++) {
      mem[ptr + i] = json.charCodeAt(i)
    }

    const target = this.alloc(8)

    mem[target + 0] = (ptr & 0xFF)
    mem[target + 1] = (ptr & 0xFF00) >> 8
    mem[target + 2] = (ptr & 0xFF0000) >> 16
    mem[target + 3] = (ptr & 0xFF000000) >> 24

    mem[target + 4] = (len & 0xFF)
    mem[target + 5] = (len & 0xFF00) >> 8
    mem[target + 6] = (len & 0xFF0000) >> 16
    mem[target + 7] = (len & 0xFF000000) >> 24

    return target
  }
}
