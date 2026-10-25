import { Buffer } from 'buffer';

/** `crypto.randomBytes` on the worker's `crypto.getRandomValues`. */
export function randomBytes(size: number): Buffer {
  const bytes = new Uint8Array(size);
  crypto.getRandomValues(bytes);
  return Buffer.from(bytes);
}
