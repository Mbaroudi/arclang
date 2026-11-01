// The timeline of a model: the commits that touched its file, oldest first,
// with the snapshots (tags) and branch heads on them, as `git log` gives
// them. Pure: the text of the log in, the commits out.

export interface TimelineCommit {
  hash: string;
  short: string;
  /** ISO date of the commit. */
  date: string;
  author: string;
  subject: string;
  /** Snapshots taken on this commit (git tags). */
  tags: string[];
  /** Branches whose head this commit is. */
  heads: string[];
  /** A merge: more than one parent. */
  merge: boolean;
}

export interface Timeline {
  /** The branch checked out, or null when detached. */
  branch: string | null;
  commits: TimelineCommit[];
}

/** The format `parseLog` reads: `git log --format=` this, `%x1e` between commits. */
export const LOG_FORMAT = '%H%x1f%h%x1f%cI%x1f%an%x1f%s%x1f%P%x1f%D%x1e';

/** `HEAD -> main, tag: v1.0, origin/main, feature` → tags and local heads. */
export function decorations(text: string): { tags: string[]; heads: string[] } {
  const tags: string[] = [];
  const heads: string[] = [];
  for (const raw of text.split(',').map((d) => d.trim()).filter(Boolean)) {
    if (raw.startsWith('tag: ')) tags.push(raw.slice(5));
    else if (raw.startsWith('HEAD -> ')) heads.push(raw.slice(8));
    else if (raw !== 'HEAD' && !raw.includes('/')) heads.push(raw);
  }
  return { tags, heads };
}

/** The commits of `git log --format=LOG_FORMAT`, oldest first. */
export function parseLog(text: string): TimelineCommit[] {
  const commits: TimelineCommit[] = [];
  for (const record of text.split('\u001e')) {
    const fields = record.replace(/^\n/, '').split('\u001f');
    if (fields.length < 7 || !/^[0-9a-f]{7,64}$/.test(fields[0])) continue;
    const [hash, short, date, author, subject, parents, refs] = fields;
    const { tags, heads } = decorations(refs.trim());
    commits.push({ hash, short, date, author, subject, tags, heads, merge: parents.trim().split(/\s+/).filter(Boolean).length > 1 });
  }
  return commits.reverse();
}

/** `2026-11-01 14:05`, from an ISO date, in the reader's local time. */
export function shortDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const two = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${two(d.getMonth() + 1)}-${two(d.getDate())} ${two(d.getHours())}:${two(d.getMinutes())}`;
}

/** The commit before `index` on the timeline, the base its changes are shown against; null for the first. */
export function previousOf(commits: TimelineCommit[], index: number): TimelineCommit | null {
  return index > 0 ? commits[index - 1] : null;
}

/** A name git takes for a branch or a tag, refused before git is asked. */
export function validRefName(name: string): boolean {
  return /^[A-Za-z0-9][A-Za-z0-9._\-/]{0,99}$/.test(name) && !name.includes('..') && !name.endsWith('/') && !name.endsWith('.lock') && !name.includes('//');
}
