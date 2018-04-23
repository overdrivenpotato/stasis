import Module from './module'
import { Map } from './types'
import Binary from './binary'

export default class Wrapper {
  private binary: Binary
  private modules: Map<Module>
  private counter: number

  constructor(binary: Binary) {
    this.binary = binary
    this.modules = {}
    this.counter = 0
  }

  public createModule(): number {
    this.counter++

    this.modules[this.counter] = new Module(this.binary)

    return this.counter
  }

  public getModule(id: number): Module {
    // TODO: This may actually be null.
    return this.modules[id]!
  }
}
