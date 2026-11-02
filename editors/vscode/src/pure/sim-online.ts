// OpenModelica online (the browser build on arclang.fr): which files of the
// model's folder are sent to the sandboxed OpenModelica, and how the answer
// is read. The names follow the server's rule: a name it would refuse (a
// dot file, the compiler's own `.arclang` folder) is never sent.

/** The files a simulation may read: models, Modelica sources, scripts and data. */
const SENT = /\.(arc|mo|mos|csv|txt|json|tsv|mat)$/i;
const SEGMENT = /^[A-Za-z0-9_][A-Za-z0-9._ -]{0,99}$/;
const SKIPPED_DIRS = new Set(['node_modules', 'target', 'dist', 'out']);

export const ONLINE_LIMITS = { files: 200, bytes: 2_000_000, depth: 8 };

/** Whether a path relative to the model's folder can be sent. */
export function sendablePath(rel: string): boolean {
  const parts = rel.split('/');
  return (
    parts.length <= ONLINE_LIMITS.depth &&
    parts.every((p) => SEGMENT.test(p) && !p.endsWith('.') && !p.endsWith(' ')) &&
    !parts.slice(0, -1).some((p) => SKIPPED_DIRS.has(p)) &&
    SENT.test(parts[parts.length - 1])
  );
}

/** Whether a folder is worth walking into. */
export function walkable(name: string): boolean {
  return SEGMENT.test(name) && !SKIPPED_DIRS.has(name);
}

/**
 * The files sent with the model: the model first, then the others by path,
 * within the limits. `error` says why the model itself cannot go.
 */
export function pickFiles(model: string, found: { path: string; size: number }[]): { take: string[]; left: number; error?: string } {
  const modelFile = found.find((f) => f.path === model);
  if (!sendablePath(model) || !modelFile) return { take: [], left: 0, error: `the model's file name (${model}) cannot be sent: use letters, digits, spaces, dots, dashes and underscores` };
  if (modelFile.size > ONLINE_LIMITS.bytes) return { take: [], left: 0, error: `the model is larger than ${ONLINE_LIMITS.bytes / 1e6} MB` };
  const take = [model];
  let bytes = modelFile.size;
  const others = found.filter((f) => f.path !== model && sendablePath(f.path)).sort((a, b) => a.path.localeCompare(b.path));
  let left = 0;
  for (const f of others) {
    if (take.length >= ONLINE_LIMITS.files || bytes + f.size > ONLINE_LIMITS.bytes) {
      left += 1;
      continue;
    }
    take.push(f.path);
    bytes += f.size;
  }
  return { take, left };
}

/** `· 19 online runs left today`, or nothing when the server did not say. */
export function leftText(left: unknown): string {
  if (typeof left !== 'number' || !Number.isFinite(left)) return '';
  return ` · ${left} online run${left === 1 ? '' : 's'} left today`;
}
