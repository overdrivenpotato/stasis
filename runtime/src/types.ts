export type Pointer = number

export type Map<T> = {
  [key in string]?: T
}

export interface Callback {
  name: string
  args: Array<any>
}
