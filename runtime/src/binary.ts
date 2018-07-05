import { Pointer } from './types'

export interface Exports {
  memory: { buffer: ArrayBuffer }
  __stasis_callback(op: number, a: number, b: number): number
}

const BYTES = {
  U32: 4,
}

const callback_opcodes = {
  ENTRYPOINT: 0,
  ALLOC: 1,
  DEALLOC: 2,
  CALLBACK: 3,
}

export default class Binary {
  constructor(private exports: Exports) {}

  private mem(): Uint8Array {
    return new Uint8Array(this.exports.memory.buffer)
  }

  private writeU32(ptr: Pointer, n: number) {
    const mem = this.mem()

    mem[ptr + 0] = (n & 0xFF)
    mem[ptr + 1] = (n & 0xFF00) >> 8
    mem[ptr + 2] = (n & 0xFF0000) >> 16
    mem[ptr + 3] = (n & 0xFF000000) >> 24
  }

  private readU32(ptr: Pointer): number {
    const mem = this.mem()

    let n = 0

    n += mem[ptr + 0]
    n += mem[ptr + 1] << 8
    n += mem[ptr + 2] << 16
    n += mem[ptr + 3] << 24

    return n
  }

  private dealloc(ptr: Pointer, len: number) {
    this.exports.__stasis_callback(callback_opcodes.DEALLOC, ptr, len)
  }

  private alloc(size: number): Pointer {
    return this.exports.__stasis_callback(callback_opcodes.ALLOC, size, 0)
  }

  public main() {
    this.exports.__stasis_callback(callback_opcodes.ENTRYPOINT, 0, 0)
  }

  public callback(id: Pointer, ...args: Array<any>): any {
    // A callback only accepts 1 argument, so we flatten the array appropriately
    // here.
    const patchedArgs =
      args.length === 1
        ? args[0]
      : args.length === 0
        ? null
      : args

    const [ptr, len] = this.makeJson(patchedArgs)

    const landingPad = this.alloc(3 * BYTES.U32)

    this.writeU32(landingPad + 0 * BYTES.U32, id)
    this.writeU32(landingPad + 1 * BYTES.U32, ptr)
    this.writeU32(landingPad + 2 * BYTES.U32, len)

    const ret = this.exports.__stasis_callback(callback_opcodes.CALLBACK, landingPad, 0)

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

  private getPair(pad: Pointer): [Pointer, number] {
    const ptr = this.readU32(pad)
    const len = this.readU32(pad + 1 * BYTES.U32)

    this.dealloc(pad, 8)

    return [ptr, len]
  }

  public makePair(obj: any): Pointer {
    if (obj === undefined) {
      return 0
    }

    const [ptr, len] = this.makeJson(obj)

    // Alloc 8 bytes at a target address to write to.
    const target = this.alloc(8)

    this.writeU32(target + 0 * BYTES.U32, ptr)
    this.writeU32(target + 1 * BYTES.U32, len)

    return target
  }
}
