import { Map, Pointer } from './types'
import Binary from './binary'

export default class Module {
  private binary: Binary
  private functions: Map<Function>
  private callbacks: Map<Function>
  private data: any

  constructor(binary: Binary) {
    this.binary = binary
    this.functions = {}
    this.callbacks = {}
    this.data = {}
  }

  public register(name: string, f: string) {
    this.functions[name] = eval(`(${f})`)
  }

  public registerCallback(name: string, pointer: Pointer) {
    this.callbacks[name] = (...args: Array<any>): any => {
      return this.binary.callback(pointer, ...args)
    }
  }

  public call(name: string, args: any): any {
    // Make sure this is always an array.
    if (!(args instanceof Array)) {
      args = [ args ]
    }

    const thisArg = {
      data: this.data,
      callbacks: this.callbacks,
    }

    try {
      return this.functions[name]!.apply(thisArg, args)
    } catch (e) {
      console.error(
        'An implementation error within a stasis module has occurred.\n' +
        'This module is now most likely in an invalid state.\n' +
        'This error is FATAL.\n' +
        'Details:\n',
        e
      )

      throw 'stasis function call error'
    }
  }
}
