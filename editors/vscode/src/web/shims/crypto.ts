import { Buffer } from 'buffer';

/**
 * `crypto.randomBytes` on the worker's `crypto.getRandomValues`. The Buffer
 * polyfill knows no `base64url` (Node's own does, and the pages' nonces use it).
 */
export function randomBytes(size: number): Buffer {
  const bytes = new Uint8Array(size);
  crypto.getRandomValues(bytes);
  const buffer = Buffer.from(bytes);
  const toString = buffer.toString.bind(buffer) as (encoding?: string, start?: number, end?: number) => string;
  buffer.toString = ((encoding?: string, start?: number, end?: number) =>
    encoding === 'base64url' ? toString('base64').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '') : toString(encoding, start, end)) as typeof buffer.toString;
  return buffer;
}
