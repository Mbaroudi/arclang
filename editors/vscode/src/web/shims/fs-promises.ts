import * as vfs from '../vfs';

export async function mkdtemp(prefix: string): Promise<string> {
  const path = `${prefix}${vfs.randomName(6)}`;
  vfs.mkdir(path);
  return path;
}

export async function rm(path: string): Promise<void> {
  vfs.remove(path);
}

export async function writeFile(path: string, data: string | Uint8Array): Promise<void> {
  vfs.write(path, data);
}

export async function readFile(path: string, encoding?: string): Promise<string | Uint8Array> {
  return encoding ? vfs.readText(path) : vfs.read(path);
}
