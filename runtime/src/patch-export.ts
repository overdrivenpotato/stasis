// A minimal WASM parser to force table export.
//
// Patching has an overhead on the order of 20 milliseconds. This is not great,
// but patching is not always necessary. If the table is already exported,
// simply scanning the binary should take only 1-2 milliseconds and will not
// create a noticable performance degredation.

const TABLE_SECTION_CODE = 4
const EXPORT_SECTION_CODE = 7

const EXTERNAL_KIND_TABLE = 1

// The default LLVM table export name.
const TABLE_EXPORT_NAME = '__indirect_function_table'

class Cursor<T extends ArrayLike<number>> {
  ptr: number

  constructor (private bytes: T, offset?: number) {
    this.ptr = offset || 0
  }

  // Returns undefined if no more bytes.
  public nextByte(): number | undefined {
    return this.bytes[this.ptr++]
  }

  // LEB128 decoder. Returns undefined if reaches end of stream.
  public nextVarUint(): number | undefined {
    let result = 0
    let shift = 0

    while (true) {
      const byte = this.nextByte()

      if (byte === undefined) {
        return undefined
      }

      result |= (byte & 0x7F) << shift

      if ((byte & 0x80) == 0) {
        break
      }

      shift += 7
    }

    return result
  }

  public skip(n: number) {
    this.ptr += n
  }

  public clone(): Cursor<T> {
    return this.cloneOn(this.bytes)
  }

  public cloneOn<U extends ArrayLike<number>>(bytes: U): Cursor<U> {
    return new Cursor(bytes, this.ptr)
  }

  public index(): number {
    return this.ptr
  }
}

const encodeVarUint = (value: number): Array<number> => {
  const bytes = []

  do {
    let byte = (value & 0x7f)

    value >>= 7

    if (value != 0) {
      byte |= 0x80
    }

    bytes.push(byte)
  } while (value != 0)

  return bytes
}

const findSection = <T extends ArrayLike<number>>(
  sectionCode: number,
  cur: Cursor<T>,
  // Start of data or head?
  startOfData: boolean,
): Cursor<T> | null => {
  while (true) {
    const head = cur.clone()
    const n = cur.nextVarUint()
    const len = cur.nextVarUint()

    if (n === undefined || len === undefined) {
      return null
    }

    if (n == sectionCode) {
      return startOfData ? cur : head
    }

    // Forward to the next section...
    cur.skip(len)
  }
}

// Get the number of tables.
const countTables = <T extends ArrayLike<number>>
  (tableSection: Cursor<T>): number | undefined => tableSection.nextVarUint()

const isTable0Exported = <T extends ArrayLike<number>>(cur: Cursor<T>): boolean | undefined => {
  const numExports = cur.nextVarUint()

  if (numExports === undefined) {
    return undefined
  }

  for (let i = 0; i < numExports; i++) {
    const fieldLen = cur.nextVarUint()

    if (fieldLen === undefined) { return undefined }

    cur.skip(fieldLen)

    const kind = cur.nextByte()
    const index = cur.nextVarUint()

    if (kind === undefined || index === undefined) { return undefined }

    if (kind === EXTERNAL_KIND_TABLE && index === 0) {
      return true
    }

    // We have read the entire entry, no need to skip bytes here for the next
    // one.
  }

  return false
}

const updateExportSectionSize = (
  bytes: Array<number>,
  delta: number,
) => {
  const cur = findSection(EXPORT_SECTION_CODE, new Cursor(bytes, 8), false)

  if (!cur) {
    return
  }

  // Skip type identifier.
  cur.nextVarUint()

  const start = cur.index()
  const current = cur.nextVarUint()
  const end = cur.index()

  const remove = end - start

  if (current === undefined) {
    return
  }

  bytes.splice(start, remove, ...encodeVarUint(current + delta))
}

const patch = <T extends ArrayLike<number>>(
  bytes: Array<number>,
  exportSection: Cursor<T>,
): Uint8Array => {
  let byteDifference = 0

  const cur = exportSection.cloneOn(bytes)

  const start = cur.index()
  const numExports = cur.nextVarUint()
  const end = cur.index()

  const oldLenSize = end - start

  const newLen = encodeVarUint(numExports! + 1)

  bytes.splice(start, oldLenSize, ...newLen)
  byteDifference += newLen.length - oldLenSize

  let insertionIndex = start + newLen.length

  const fieldStr = (() => {
    const a = []

    for (let i = 0; i < TABLE_EXPORT_NAME.length; i++) {
      a.push(TABLE_EXPORT_NAME.charCodeAt(i))
    }

    return a
  })()

  const fieldLen = encodeVarUint(fieldStr.length)

  bytes.splice(insertionIndex, 0, ...fieldLen)
  byteDifference += fieldLen.length
  insertionIndex += fieldLen.length

  bytes.splice(insertionIndex, 0, ...fieldStr)
  byteDifference += fieldStr.length
  insertionIndex += fieldStr.length

  bytes.splice(insertionIndex, 0, EXTERNAL_KIND_TABLE)
  byteDifference += 1
  insertionIndex += 1

  bytes.splice(insertionIndex, 0, 0)
  byteDifference += 1
  insertionIndex += 1

  updateExportSectionSize(bytes, byteDifference)

  return new Uint8Array(bytes)
}

// The table may not be exported. This exports the table binary for us.
export default (bytes: Uint8Array): Uint8Array => {
  const tableSection = findSection(TABLE_SECTION_CODE, new Cursor(bytes, 8), true)

  // Table may not exist.
  if (!tableSection) {
    return bytes
  }

  // We can start searching at the table section offset because the export
  // section is guaranteed to always come after the table section.
  const exportSection = findSection(EXPORT_SECTION_CODE, new Cursor(bytes, 8), true)

  if (!exportSection) {
    // TODO: Creation of export section.
    return bytes
  }

  const numTables = countTables(tableSection.clone())

  if (numTables === undefined || numTables !== 1) {
    return bytes
  }

  const tableExported = isTable0Exported(exportSection.clone())

  if (tableExported === undefined || tableExported === true) {
    return bytes
  }

  // The table is not exported, we need to patch it.
  const modifiedBytes = patch(Array.from(bytes), exportSection)

  return modifiedBytes
}
