import { Pointer } from './types'

export interface Exports {
  memory: { buffer: ArrayBuffer }
  __stasis_entrypoint(): void
  __stasis_alloc(size: number): Pointer
  __stasis_dealloc(ptr: Pointer, len: number): void
  __stasis_callback(ptr: Pointer, len: number): Pointer
}

export default class Binary {
  constructor(private exports: Exports) {}

  private mem(): Uint8Array {
    return new Uint8Array(this.exports.memory.buffer)
  }

  private dealloc(ptr: Pointer, len: number) {
    this.exports.__stasis_dealloc(ptr, len)
  }

  private alloc(size: number): Pointer {
    return this.exports.__stasis_alloc(size)
  }

  public main() {
    this.exports.__stasis_entrypoint()
  }

  public callback(id: Pointer, ...args: Array<any>): any {
    const patchedArgs =
      args.length === 1
        ? args[0]
      : args.length === 0
        ? null
      : args

    interface Callback {
      id: number
      ptr: Pointer
      len: number
    }

    const argJson = this.makeJson(patchedArgs)

    const json: Callback = {
      id,
      ptr: argJson[0],
      len: argJson[1],
    }

    const [ptr, len] = this.makeJson(json)

    const ret = this.exports.__stasis_callback(ptr, len)

    if (ret === 0) {
      return undefined
    }

    const returnPair = this.getPair(ret)
    return this.getJson(returnPair[0], returnPair[1])
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

  public getJson(ptr: Pointer, len: number): any {
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

  public makeJson(obj: any): [Pointer, number] {
    if (obj === undefined) {
      return [0, 0]
    }

    const json = JSON.stringify(obj)

    const mem = this.mem()
    const len = json.length
    const ptr = this.alloc(json.length)

    for (let i = 0; i < len; i++) {
      mem[ptr + i] = json.charCodeAt(i)
    }

    return [ptr, len]
  }

  private getPair(pointer: Pointer): [Pointer, number] {
    const mem = this.mem()

    let ptr = 0
    let len = 0

    ptr += mem[pointer + 0]
    ptr += mem[pointer + 1] << 8
    ptr += mem[pointer + 2] << 16
    ptr += mem[pointer + 3] << 24

    len += mem[pointer + 4]
    len += mem[pointer + 5] << 8
    len += mem[pointer + 6] << 16
    len += mem[pointer + 7] << 24

    this.dealloc(pointer, 8)

    return [ptr, len]
  }

  public makePair(obj: any): Pointer {
    if (obj === undefined) {
      return 0
    }

    const mem = this.mem()
    const [ptr, len] = this.makeJson(obj)

    // Alloc 8 bytes at a target address to write to.
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
