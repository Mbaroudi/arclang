// What an import takes and what it gave: the source kinds offered, the file
// or folder the compiler is given for a path the user chose (an .aird is read
// through its .capella, a folder of .sysml files is one model), the address
// of a Systems Modeling API server, and the compiler's report read back into
// counts. No VS Code here: the unit tests run it as is.

/** The `-f` of `arclang import` this extension uses. */
export type ImportFormat = 'capella' | 'sysml' | 'req-if' | 'mermaid' | 'plant-uml';

/** What the user may import, as the quick pick lists it. */
export type SourceKind = 'capella' | 'sysml' | 'sysml-api' | 'reqif' | 'doors';

export const SOURCE_KINDS: ReadonlyArray<{ kind: SourceKind; label: string; detail: string }> = [
  { kind: 'capella', label: 'Capella model', detail: 'A .capella file (or its .aird, or the project folder): every layer, the requirements, the diagrams' },
  { kind: 'sysml', label: 'SysML v2 file(s) or folder', detail: 'One .sysml or .kerml file, several of them, or a folder of them, imported as one model' },
  { kind: 'sysml-api', label: 'SysML v2 API server', detail: 'A project and a commit of a Systems Modeling API server (OMG standard API)' },
  { kind: 'reqif', label: 'Requirements (ReqIF)', detail: 'A .reqif or .reqifz file, into the open model or a model of its own' },
  { kind: 'doors', label: 'IBM DOORS (through its ReqIF export)', detail: 'DOORS and DOORS Next export a module as ReqIF; that file is imported' },
];

/** The file name filters of the open dialog of each kind. */
export const OPEN_FILTERS: Record<Exclude<SourceKind, 'sysml-api'>, Record<string, string[]>> = {
  capella: { 'Capella model': ['capella', 'melodymodeller', 'aird', 'afm'] },
  sysml: { 'SysML v2 text': ['sysml', 'kerml'], 'SysML v2 API records': ['json'] },
  reqif: { ReqIF: ['reqif', 'reqifz'] },
  doors: { 'ReqIF exported from DOORS': ['reqif', 'reqifz'] },
};

/** What the compiler is given: `input` with `-f format`. */
export interface ResolvedImport {
  format: ImportFormat;
  input: string;
  /** The name the result is called by (`<base>.arc`). */
  base: string;
  /** A .reqifz: the ReqIF document inside is extracted first. */
  unzip?: boolean;
  /** Said in the report: how the path chosen became the input. */
  note?: string;
}

export type Resolution = ResolvedImport | { error: string };

/** One entry of a directory, as the host lists it. */
export interface DirEntry {
  name: string;
  isDirectory: boolean;
}

const lower = (name: string) => name.toLowerCase();
const extensionOf = (name: string) => {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? lower(name.slice(dot + 1)) : '';
};
const stem = (name: string) => {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
};
export const baseName = (file: string) => file.replace(/[\\/]+$/, '').split(/[\\/]/).pop() ?? file;
const dirName = (file: string) => {
  const at = Math.max(file.lastIndexOf('/'), file.lastIndexOf('\\'));
  return at > 0 ? file.slice(0, at) : '.';
};
const join = (dir: string, name: string) => `${dir.replace(/[\\/]+$/, '')}/${name}`;

export const isSysmlText = (name: string) => ['sysml', 'kerml'].includes(extensionOf(name));
export const isCapellaModel = (name: string) => ['capella', 'melodymodeller'].includes(extensionOf(name));

/**
 * Whether a JSON text is a listing of Systems Modeling API records: an array
 * of objects with `@id` and `@type`, or an object holding one (`elements`).
 * Only the start of the file is needed.
 */
export function looksLikeApiRecords(head: string): boolean {
  const text = head.replace(/^﻿/, '').trimStart();
  if (!text.startsWith('[') && !text.startsWith('{')) return false;
  const window = text.slice(0, 8192);
  return /"@id"\s*:/.test(window) && /"@type"\s*:/.test(window);
}

/**
 * The Capella model of a diagram file (.aird) or a fragment (.afm): the
 * .capella of the same name beside it, else the only one there.
 */
export function capellaBeside(file: string, siblings: string[]): Resolution {
  const name = baseName(file);
  const models = siblings.filter(isCapellaModel);
  const same = models.find((m) => lower(stem(m)) === lower(stem(name)));
  const picked = same ?? (models.length === 1 ? models[0] : undefined);
  if (!picked) {
    return { error: models.length === 0
      ? `${name} holds the diagrams of a Capella model, not the model: no .capella file is beside it.`
      : `${name}: several Capella models are beside it (${models.join(', ')}); choose the .capella file.` };
  }
  return { format: 'capella', input: join(dirName(file), picked), base: stem(picked), note: `${name} is read through ${picked}, the model beside it (its diagrams come along).` };
}

/** What a folder can be imported as: a Capella project (a .capella in it), SysML v2 files as one model. */
export function folderImports(folder: string, entries: DirEntry[], holdsSysml: boolean): ResolvedImport[] {
  const out: ResolvedImport[] = [];
  const models = entries.filter((e) => !e.isDirectory && isCapellaModel(e.name)).map((e) => e.name).sort();
  const name = baseName(folder);
  for (const model of models) {
    out.push({ format: 'capella', input: join(folder, model), base: stem(model), note: `The Capella project ${name} is read through ${model}.` });
  }
  if (holdsSysml) out.push({ format: 'sysml', input: folder, base: name, note: `Every .sysml and .kerml file under ${name} is read as one model, in name order.` });
  return out;
}

/**
 * The input of the compiler for a file chosen by the user. `siblings`: the
 * names in its folder (for an .aird); `head`: the start of the file (for a
 * .json, to tell API records from other JSON).
 */
export function resolveFile(file: string, siblings: string[] = [], head = ''): Resolution {
  const name = baseName(file);
  const ext = extensionOf(name);
  switch (ext) {
    case 'capella':
    case 'melodymodeller':
      return { format: 'capella', input: file, base: stem(name) };
    case 'aird':
    case 'afm':
      return capellaBeside(file, siblings);
    case 'sysml':
    case 'kerml':
      return { format: 'sysml', input: file, base: stem(name) };
    case 'reqif':
      return { format: 'req-if', input: file, base: stem(name) };
    case 'reqifz':
      return { format: 'req-if', input: file, base: stem(name), unzip: true, note: `${name} is a ReqIF archive: the .reqif document inside it is read.` };
    case 'json':
      return looksLikeApiRecords(head)
        ? { format: 'sysml', input: file, base: stem(name), note: `${name} is read as a listing of SysML v2 API records.` }
        : { error: `${name} is not a listing of SysML v2 API records (an array of objects with "@id" and "@type").` };
    case 'mmd':
    case 'mermaid':
      return { format: 'mermaid', input: file, base: stem(name) };
    case 'puml':
    case 'plantuml':
    case 'pu':
      return { format: 'plant-uml', input: file, base: stem(name) };
    default:
      return { error: `${name}: ArcLang imports Capella (.capella, .aird), SysML v2 (.sysml, .kerml, API .json), ReqIF (.reqif, .reqifz), Mermaid and PlantUML.` };
  }
}

/**
 * Several SysML v2 files chosen together: one model, read from the folder
 * they share when they are all of it, else from a copy of them in a fresh
 * folder (`copy` is true).
 */
export function sysmlSelection(files: string[], folderEntries: DirEntry[]): { folder: string; copy: boolean } | { error: string } {
  if (files.length === 0) return { error: 'no file chosen' };
  const others = files.filter((f) => !isSysmlText(f));
  if (others.length > 0) return { error: `only .sysml and .kerml files are read as one model: ${others.map(baseName).join(', ')}` };
  const dirs = new Set(files.map(dirName));
  if (dirs.size === 1) {
    const folder = [...dirs][0];
    const chosen = new Set(files.map(baseName));
    const inFolder = folderEntries.filter((e) => e.isDirectory || isSysmlText(e.name));
    const all = inFolder.length > 0 && inFolder.every((e) => !e.isDirectory && chosen.has(e.name));
    return { folder, copy: !all };
  }
  return { folder: '', copy: true };
}

/** The base name of a model imported from several files: their folder's, else the first file's. */
export function selectionBase(files: string[]): string {
  const dirs = new Set(files.map(dirName));
  return dirs.size === 1 ? baseName([...dirs][0]) : stem(baseName(files[0] ?? 'model'));
}

// --- the report --------------------------------------------------------------

export interface ImportCount {
  count: number;
  what: string;
  /** The compiler says these were not written: what it left out, and why. */
  leftOut: boolean;
}

export interface ImportOutcome {
  ok: boolean;
  counts: ImportCount[];
  notes: string[];
  warnings: string[];
  /** The refusal, when the import failed. */
  error: string | null;
  /** Where the compiler kept the text it refused (`<output>.rejected.arc`). */
  rejected: string | null;
}

const LEFT_OUT = /\(left out|left out\)|\bleft out\b|not written|left unresolved|not imported/i;

/** The output of `arclang import`, read back: counts, notes, warnings, the refusal. */
export function parseImportOutput(stdout: string, stderr: string, code: number): ImportOutcome {
  const counts: ImportCount[] = [];
  const notes: string[] = [];
  const warnings: string[] = [];
  let error: string | null = null;
  let rejected: string | null = null;
  for (const line of `${stdout}\n${stderr}`.split(/\r?\n/)) {
    const count = /^\s+(\d+)\s+(.+?)\s*$/.exec(line);
    if (count && !/^\s+(Input|Output|Format):/.test(line)) {
      counts.push({ count: Number(count[1]), what: count[2], leftOut: LEFT_OUT.test(count[2]) });
      continue;
    }
    const note = /^\s+note:\s*(.+)$/.exec(line);
    if (note) { notes.push(note[1].trim()); continue; }
    const kept = /the refused text is in (.+?)\s*$/.exec(line);
    if (kept) { rejected = kept[1]; continue; }
    const warning = /^\s*(?:warning|⚠)[:\s]\s*(.+)$/i.exec(line);
    if (warning) { warnings.push(warning[1].trim()); continue; }
    const failed = /^\s*Error:\s*(.+)$/.exec(line);
    if (failed) error = failed[1].replace(/^Compilation error:\s*/, '').trim();
  }
  if (code !== 0 && !error) error = (stderr || stdout).trim().split(/\r?\n/).slice(-3).join(' ') || `arclang import failed (exit ${code})`;
  return { ok: code === 0, counts, notes, warnings, error: code === 0 ? null : error, rejected };
}

/** The total of the counts imported and of those left out. */
export function countTotals(counts: ImportCount[]): { imported: number; leftOut: number } {
  let imported = 0;
  let leftOut = 0;
  for (const c of counts) {
    if (c.leftOut) leftOut += c.count;
    else imported += c.count;
  }
  return { imported, leftOut };
}

// --- the Systems Modeling API --------------------------------------------------

const LOCAL_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]', '::1']);

/**
 * The base address of a Systems Modeling API server, as the user typed it:
 * https only, but for a server on this machine; no credentials in the
 * address (the token is asked for apart). The trailing slash goes.
 */
export function checkApiBase(raw: string): { base: string; local: boolean } | { error: string } {
  let url: URL;
  try {
    url = new URL(raw.trim());
  } catch {
    return { error: 'not an address (for example https://server/api or http://localhost:9000)' };
  }
  const local = LOCAL_HOSTS.has(url.hostname);
  if (url.protocol !== 'https:' && !(url.protocol === 'http:' && local)) return { error: 'the address must be https (http only for a server on this machine)' };
  if (url.username || url.password) return { error: 'no user or password in the address: the token is asked for next' };
  if (url.search || url.hash) return { error: 'the base address takes no query or fragment' };
  const base = `${url.origin}${url.pathname.replace(/\/+$/, '')}`;
  return { base, local };
}

/** The next page of a listing from its `Link` header, on the same server only; null at the end. */
export function nextPage(link: string | null, base: string): string | null | { error: string } {
  if (!link) return null;
  for (const part of link.split(/,(?=\s*<)/)) {
    const match = /<([^>]+)>\s*;\s*(.+)$/.exec(part.trim());
    if (!match || !/\brel\s*=\s*"?next"?/i.test(match[2])) continue;
    let next: URL;
    try {
      next = new URL(match[1], base);
    } catch {
      return { error: `the server gave an unreadable next page: ${match[1]}` };
    }
    if (next.origin !== new URL(base).origin) return { error: `the server sent the next page to another server (${next.origin}): refused` };
    return next.toString();
  }
  return null;
}

/** A page URL with `page[size]` set (the brackets kept as the API names them). */
export function withPageSize(url: string, size: number): string {
  return `${url}${url.includes('?') ? '&' : '?'}page[size]=${size}`;
}

/** A record of the API (project, commit) as a quick pick line. */
export function recordLabel(record: Record<string, unknown>): { id: string; label: string; detail: string } | null {
  const id = typeof record['@id'] === 'string' ? (record['@id'] as string) : null;
  if (!id) return null;
  const name = typeof record.name === 'string' && record.name ? record.name : null;
  const description = typeof record.description === 'string' ? record.description : '';
  const created = typeof record.created === 'string' ? record.created : '';
  return { id, label: name ?? (description || id), detail: [created, name ? description : '', id].filter(Boolean).join(' · ') };
}

/** Commits listed newest first, by their `created` date when they have one. */
export function newestFirst<T extends Record<string, unknown>>(commits: T[]): T[] {
  const time = (c: T) => (typeof c.created === 'string' ? Date.parse(c.created) || 0 : 0);
  return [...commits].sort((a, b) => time(b) - time(a));
}

/** The ReqIF document of a .reqifz archive: its first .reqif entry, by name. */
export function reqifEntryOf<T extends { name: string }>(entries: T[]): T | { error: string } {
  const documents = entries.filter((e) => /\.reqif$/i.test(e.name) && !e.name.endsWith('/') && !e.name.split('/').includes('..')).sort((a, b) => a.name.localeCompare(b.name));
  return documents[0] ?? { error: `the archive holds no .reqif document (${entries.map((e) => e.name).join(', ') || 'it is empty'})` };
}

/** The extensions a file dropped on the workbench may be imported from. */
export const DROP_EXTENSIONS = ['capella', 'melodymodeller', 'aird', 'sysml', 'kerml', 'reqif', 'reqifz'];

/**
 * The model files named by what a drag carries (a uri list, VS Code's list
 * of resources or of editors): `file:` URIs with an importable extension,
 * each once. A tree node of the model (`#arclang-element=`) is not one.
 */
export function droppedModelFiles(payloads: string[]): string[] {
  const found: string[] = [];
  const consider = (value: unknown) => {
    // A URI as VS Code serializes it ({ scheme, path }) or as text.
    const uri = value as { scheme?: unknown; path?: unknown } | null;
    const raw = typeof value === 'string' ? value : uri && uri.scheme === 'file' && typeof uri.path === 'string' ? `file://${encodeURI(uri.path)}` : null;
    if (raw === null) return;
    const text = raw.trim();
    if (!/^file:\/\//i.test(text) || /#arclang-element/i.test(text)) return;
    let pathname: string;
    try {
      pathname = decodeURIComponent(new URL(text).pathname);
    } catch {
      return;
    }
    if (!DROP_EXTENSIONS.includes(extensionOf(pathname))) return;
    if (!found.includes(text)) found.push(text);
  };
  for (const payload of payloads) {
    if (!payload) continue;
    try {
      const parsed: unknown = JSON.parse(payload);
      const items = Array.isArray(parsed) ? parsed : [parsed];
      for (const item of items) consider(typeof item === 'object' && item !== null ? (item as { resource?: unknown; uri?: unknown }).resource ?? (item as { uri?: unknown }).uri : item);
      continue;
    } catch {
      // not JSON: a uri list, one per line, comments after #
    }
    for (const line of payload.split(/\r?\n/)) if (!line.startsWith('#')) consider(line);
  }
  return found;
}

/**
 * The declarations of an imported model without its `model … { … }` header
 * block and the comments above it: what is appended to another model to
 * merge the requirements of a ReqIF file into it. Braces inside string
 * literals do not count. Null when the text has no model block.
 */
export function declarationsAfterModel(text: string): string | null {
  const start = /^model\s+[^\n{]*\{/m.exec(text);
  if (!start) return null;
  let depth = 0;
  let inString = false;
  for (let i = start.index + start[0].length - 1; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      if (c === '\\') i += 1;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') inString = true;
    else if (c === '{') depth += 1;
    else if (c === '}') {
      depth -= 1;
      if (depth === 0) return text.slice(i + 1).replace(/^\s*\n/, '').trimEnd();
    }
  }
  return null;
}
