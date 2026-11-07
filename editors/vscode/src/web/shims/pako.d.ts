// The part of pako (1.x, no types of its own) the browser build's zlib uses.
declare module 'pako' {
  export function deflateRaw(data: Uint8Array): Uint8Array;
  export function inflateRaw(data: Uint8Array): Uint8Array;
  export function gzip(data: Uint8Array): Uint8Array;
  export function ungzip(data: Uint8Array): Uint8Array;
}
