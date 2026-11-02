// Glob patterns as VS Code reads them (`**`, `*`, `?`, `{a,b}`), for
// filtering paths: the browser build's file search answers every path of the
// workspace, folders included, whatever the pattern (see ../files.ts).

/** A regular expression matching a whole relative path against `pattern`. */
export function globToRegExp(pattern: string): RegExp {
  let out = '';
  for (let i = 0; i < pattern.length; i++) {
    const c = pattern[i];
    if (c === '*') {
      if (pattern[i + 1] === '*') {
        // `**/` is any number of folders (none too); a final `**` is anything.
        const slash = pattern[i + 2] === '/';
        out += slash ? '(?:.*/)?' : '.*';
        i += slash ? 2 : 1;
      } else out += '[^/]*';
    } else if (c === '?') out += '[^/]';
    else if (c === '{') {
      const end = pattern.indexOf('}', i);
      if (end < 0) {
        out += '\\{';
        continue;
      }
      out += `(?:${pattern
        .slice(i + 1, end)
        .split(',')
        .map((alt) => globToRegExp(alt).source.slice(1, -1))
        .join('|')})`;
      i = end;
    } else out += /[.+^$()|[\]\\]/.test(c) ? `\\${c}` : c;
  }
  return new RegExp(`^${out}$`);
}

/** The paths (relative, with `/`) matching `include` and not `exclude`. */
export function filterPaths(paths: string[], include: string, exclude?: string | null): string[] {
  const inc = globToRegExp(include);
  const exc = exclude ? globToRegExp(exclude) : null;
  return paths.filter((p) => inc.test(p) && !(exc && exc.test(p)));
}
