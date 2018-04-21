import { Map } from './types'

export default class Module {
  private functions: Map<Function>
  private data: any

  constructor() {
    this.functions = {}
    this.data = {}
  }

  public register(name: string, f: string) {
    this.functions[name] = eval(`(${f})`)
  }

  public call(name: string, args: any): any {
    // Make sure this is always an array.
    if (!(args instanceof Array)) {
      args = [ args ]
    }

    const thisArg = {
      data: this.data,
    }

    try {
      return this.functions[name]!.apply(thisArg, args)
    } catch (e) {
      throw console.error('IMPLEMENTATION ERROR', e)
    }
  }
}
