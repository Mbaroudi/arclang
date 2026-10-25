// `node:fs` in the web build: the subset the extension uses, on the worker's
// in-memory file system (see ../vfs.ts). Anything else is not there.
import * as vfs from '../vfs';

export const constants = { R_OK: 4, W_OK: 2, X_OK: 1, F_OK: 0 };

export function existsSync(path: string): boolean {
  return vfs.exists(path);
}

export function readFileSync(path: string, options?: string | { encoding?: string }): string | Uint8Array {
  const encoding = typeof options === 'string' ? options : options?.encoding;
  return encoding ? vfs.readText(path) : (vfs.read(path) as Uint8Array | string);
}

export function writeFileSync(path: string, data: string | Uint8Array): void {
  vfs.write(path, data);
}

export function mkdirSync(path: string): void {
  vfs.mkdir(path);
}

export function mkdtempSync(prefix: string): string {
  const path = `${prefix}${vfs.randomName(6)}`;
  vfs.mkdir(path);
  return path;
}

export function rmSync(path: string): void {
  vfs.remove(path);
}

export function readdirSync(path: string): string[] {
  return vfs.list(path);
}

export function copyFileSync(from: string, to: string): void {
  vfs.write(to, vfs.read(from));
}

export function accessSync(path: string): void {
  vfs.read(vfs.isDirectory(path) ? '/__directory__' + path : path);
}

export function realpathSync(path: string): string {
  return vfs.normalize(path);
}

export function statSync(path: string) {
  if (!vfs.exists(path)) {
    const error = new Error(`ENOENT: no such file or directory, stat '${path}'`) as Error & { code: string };
    error.code = 'ENOENT';
    throw error;
  }
  const value = vfs.isFile(path) ? vfs.read(path) : '';
  return { isFile: () => vfs.isFile(path), isDirectory: () => vfs.isDirectory(path), size: value.length, mtimeMs: 0 };
}

const browserOnly = (name: string) => () => {
  throw new Error(`fs.${name} is not available in the browser build`);
};
export const openSync = browserOnly('openSync');
export const readSync = browserOnly('readSync');
export const closeSync = browserOnly('closeSync');
export type Stats = ReturnType<typeof statSync>;
