/// <reference path="./pako.d.ts" />
// node:zlib in the browser build, on pako (MIT): the archives of several
// files (PowerPoint decks, .reqifz packages, the diagram exports) are
// deflated and inflated here as on the desktop.
import * as pako from 'pako';

type Input = Uint8Array | ArrayBuffer | string;
const bytes = (data: Input): Uint8Array => (typeof data === 'string' ? new TextEncoder().encode(data) : data instanceof Uint8Array ? data : new Uint8Array(data));
const out = (data: Uint8Array): Buffer => Buffer.from(data.buffer, data.byteOffset, data.byteLength);

function bounded(data: Uint8Array, options?: { maxOutputLength?: number }): Uint8Array {
  if (options?.maxOutputLength !== undefined && data.length > options.maxOutputLength) {
    throw Object.assign(new RangeError(`Cannot create a Buffer larger than ${options.maxOutputLength} bytes`), { code: 'ERR_BUFFER_TOO_LARGE' });
  }
  return data;
}

export const deflateRawSync = (data: Input): Buffer => out(pako.deflateRaw(bytes(data)));
export const inflateRawSync = (data: Input, options?: { maxOutputLength?: number }): Buffer => out(bounded(pako.inflateRaw(bytes(data)), options));
export const gzipSync = (data: Input): Buffer => out(pako.gzip(bytes(data)));
export const gunzipSync = (data: Input, options?: { maxOutputLength?: number }): Buffer => out(bounded(pako.ungzip(bytes(data)), options));
