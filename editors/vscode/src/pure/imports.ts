// The files a model imports, gathered for a compiler that has no file
// system (the browser): each under its path relative to the model's folder,
// as the compiler resolves `import "..."` beside the file that writes it.

const IMPORT = /^[ \t]*import[ \t]+"([^"\n]+)"/gm;

/** A model may reach this many files, of this many characters in all. */
export const IMPORT_LIMITS = { files: 200, characters: 8_000_000 };

/** The paths a text imports, as written. */
export function importsOf(text: string): string[] {
  return Array.from(text.matchAll(IMPORT), (match) => match[1]);
}

/** `written` in the file `from`, as a path relative to the model's folder. */
export function resolveImport(from: string, written: string): string {
  const at = from.lastIndexOf('/');
  const parts = at < 0 ? [] : from.slice(0, at).split('/');
  for (const part of written.split(/[\\/]/)) {
    if (part === '' || part === '.') continue;
    if (part === '..' && parts.length && parts[parts.length - 1] !== '..') parts.pop();
    else parts.push(part);
  }
  return parts.join('/');
}

/**
 * Every file the model `name` (its text `text`) reaches by its imports and
 * theirs, read with `read` (undefined: no such file, which the compiler then
 * reports by name); null when the model imports nothing.
 */
export async function importClosure(name: string, text: string, read: (path: string) => Promise<string | undefined>, limits = IMPORT_LIMITS): Promise<Record<string, string> | null> {
  if (!importsOf(text).length) return null;
  const files: Record<string, string> = {};
  const seen = new Set<string>([name]);
  const pending = importsOf(text).map((written) => resolveImport(name, written));
  let characters = 0;
  while (pending.length && Object.keys(files).length < limits.files) {
    const path = pending.shift()!;
    if (seen.has(path)) continue;
    seen.add(path);
    const found = await read(path);
    if (found === undefined) continue;
    characters += found.length;
    if (characters > limits.characters) break;
    files[path] = found;
    for (const written of importsOf(found)) pending.push(resolveImport(path, written));
  }
  return files;
}
