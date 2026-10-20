// A minimal ZIP writer (PKWARE APPNOTE 6.3): enough for an Office Open XML
// package. Entries are deflated by the function the caller gives (Node's
// zlib.deflateRawSync) or stored. No ZIP64: a package stays under 4 GB and
// 65 535 entries, which `zip` checks.

export interface ZipEntry {
  name: string;
  data: Uint8Array;
}

export type Deflate = (data: Uint8Array) => Uint8Array;

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

/** CRC-32 (ISO 3309), as ZIP stores it. */
export function crc32(data: Uint8Array): number {
  let crc = 0xffffffff;
  for (let i = 0; i < data.length; i += 1) crc = CRC_TABLE[(crc ^ data[i]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

// 1 January 2026, 00:00: a fixed date makes the same input give the same bytes.
const DOS_TIME = 0;
const DOS_DATE = ((2026 - 1980) << 9) | (1 << 5) | 1;
const LIMIT = 0xffffffff;

/** The entries as one ZIP file, in their order. */
export function zip(entries: ZipEntry[], deflate?: Deflate): Uint8Array {
  if (entries.length > 0xffff) throw new Error('too many entries for a ZIP without ZIP64');
  const names = new Set<string>();
  const encoder = new TextEncoder();
  const locals: Uint8Array[] = [];
  const centrals: Uint8Array[] = [];
  let offset = 0;
  for (const entry of entries) {
    if (!entry.name || entry.name.startsWith('/') || entry.name.includes('..') || names.has(entry.name)) throw new Error(`bad or repeated entry name: ${entry.name}`);
    names.add(entry.name);
    const name = encoder.encode(entry.name);
    const crc = crc32(entry.data);
    const packed = deflate ? deflate(entry.data) : entry.data;
    // Deflate only when it helps (PNG data does not shrink).
    const method = deflate && packed.length < entry.data.length ? 8 : 0;
    const body = method === 8 ? packed : entry.data;
    if (body.length >= LIMIT || entry.data.length >= LIMIT || offset >= LIMIT) throw new Error('a package of 4 GB or more needs ZIP64');
    const local = new Uint8Array(30 + name.length);
    const l = new DataView(local.buffer);
    l.setUint32(0, 0x04034b50, true);
    l.setUint16(4, 20, true); // version needed: 2.0 (deflate)
    l.setUint16(6, 0x0800, true); // UTF-8 names
    l.setUint16(8, method, true);
    l.setUint16(10, DOS_TIME, true);
    l.setUint16(12, DOS_DATE, true);
    l.setUint32(14, crc, true);
    l.setUint32(18, body.length, true);
    l.setUint32(22, entry.data.length, true);
    l.setUint16(26, name.length, true);
    l.setUint16(28, 0, true);
    local.set(name, 30);
    locals.push(local, body);

    const central = new Uint8Array(46 + name.length);
    const c = new DataView(central.buffer);
    c.setUint32(0, 0x02014b50, true);
    c.setUint16(4, 20, true); // made by: version 2.0, MS-DOS
    c.setUint16(6, 20, true);
    c.setUint16(8, 0x0800, true);
    c.setUint16(10, method, true);
    c.setUint16(12, DOS_TIME, true);
    c.setUint16(14, DOS_DATE, true);
    c.setUint32(16, crc, true);
    c.setUint32(20, body.length, true);
    c.setUint32(24, entry.data.length, true);
    c.setUint16(28, name.length, true);
    c.setUint32(42, offset, true);
    central.set(name, 46);
    centrals.push(central);
    offset += local.length + body.length;
  }
  const centralSize = centrals.reduce((sum, part) => sum + part.length, 0);
  const end = new Uint8Array(22);
  const e = new DataView(end.buffer);
  e.setUint32(0, 0x06054b50, true);
  e.setUint16(8, entries.length, true);
  e.setUint16(10, entries.length, true);
  e.setUint32(12, centralSize, true);
  e.setUint32(16, offset, true);
  const parts = [...locals, ...centrals, end];
  const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let at = 0;
  for (const part of parts) {
    out.set(part, at);
    at += part.length;
  }
  return out;
}

/** The entries of a ZIP this writer made (stored or deflated by `inflate`): for tests and checks. */
export function unzip(data: Uint8Array, inflate: (data: Uint8Array) => Uint8Array): ZipEntry[] {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let endAt = data.length - 22;
  while (endAt >= 0 && view.getUint32(endAt, true) !== 0x06054b50) endAt -= 1;
  if (endAt < 0) throw new Error('no end of central directory');
  const count = view.getUint16(endAt + 10, true);
  let at = view.getUint32(endAt + 16, true);
  const decoder = new TextDecoder();
  const out: ZipEntry[] = [];
  for (let i = 0; i < count; i += 1) {
    if (view.getUint32(at, true) !== 0x02014b50) throw new Error('bad central directory');
    const method = view.getUint16(at + 10, true);
    const crc = view.getUint32(at + 16, true);
    const size = view.getUint32(at + 20, true);
    const nameLength = view.getUint16(at + 28, true);
    const extra = view.getUint16(at + 30, true);
    const comment = view.getUint16(at + 32, true);
    const local = view.getUint32(at + 42, true);
    const name = decoder.decode(data.subarray(at + 46, at + 46 + nameLength));
    const localName = view.getUint16(local + 26, true);
    const localExtra = view.getUint16(local + 28, true);
    const start = local + 30 + localName + localExtra;
    const body = data.subarray(start, start + size);
    const content = method === 8 ? inflate(body) : body.slice();
    if (crc32(content) !== crc) throw new Error(`CRC mismatch in ${name}`);
    out.push({ name, data: content });
    at += 46 + nameLength + extra + comment;
  }
  return out;
}
