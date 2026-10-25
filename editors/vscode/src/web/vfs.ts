// The in-memory file system the WebAssembly backend works on: the compiler
// calls of the extension go through temporary files (`withTempFile`), and in
// the browser those files live here, in the worker's memory, never on a disk.
const files = new Map<string, string | Uint8Array>();
const dirs = new Set<string>(['/', '/tmp']);

export function normalize(path: string): string {
  const parts: string[] = [];
  for (const part of path.split('/')) {
    if (part === '' || part === '.') continue;
    if (part === '..') parts.pop();
    else parts.push(part);
  }
  return '/' + parts.join('/');
}

export function parent(path: string): string {
  const at = normalize(path).lastIndexOf('/');
  return at <= 0 ? '/' : normalize(path).slice(0, at);
}

export function exists(path: string): boolean {
  const key = normalize(path);
  return files.has(key) || dirs.has(key);
}

export function isDirectory(path: string): boolean {
  return dirs.has(normalize(path));
}

export function isFile(path: string): boolean {
  return files.has(normalize(path));
}

export function mkdir(path: string): void {
  let current = '';
  for (const part of normalize(path).split('/').filter(Boolean)) {
    current += '/' + part;
    dirs.add(current);
  }
}

export function write(path: string, data: string | Uint8Array): void {
  mkdir(parent(path));
  files.set(normalize(path), data);
}

export function read(path: string): string | Uint8Array {
  const found = files.get(normalize(path));
  if (found === undefined) {
    const error = new Error(`ENOENT: no such file or directory, open '${path}'`) as Error & { code: string };
    error.code = 'ENOENT';
    throw error;
  }
  return found;
}

export function readText(path: string): string {
  const found = read(path);
  return typeof found === 'string' ? found : new TextDecoder().decode(found);
}

export function remove(path: string): void {
  const key = normalize(path);
  files.delete(key);
  const prefix = key === '/' ? '/' : key + '/';
  for (const name of [...files.keys()]) if (name.startsWith(prefix)) files.delete(name);
  for (const name of [...dirs]) if (name === key || name.startsWith(prefix)) dirs.delete(name);
}

export function list(path: string): string[] {
  const key = normalize(path);
  const prefix = key === '/' ? '/' : key + '/';
  const names = new Set<string>();
  for (const name of [...files.keys(), ...dirs]) {
    if (name.startsWith(prefix) && name !== key) names.add(name.slice(prefix.length).split('/')[0]);
  }
  return [...names].sort();
}

export function randomName(length = 8): string {
  const bytes = new Uint8Array(length);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('').slice(0, length);
}
