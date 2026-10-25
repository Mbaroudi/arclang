// A client of the Systems Modeling API (the OMG standard REST API of SysML
// v2), as much of it as an import needs: the projects, the commits of one,
// and every element of a commit, page by page. https only but for a server
// on this machine; redirects are refused (a token would follow them); the
// token, when there is one, lives in VS Code's secret storage.
import * as vscode from 'vscode';
import { checkApiBase, newestFirst, nextPage, recordLabel, withPageSize } from '../pure/importing';

const PAGE_SIZE = 1000;
/** No listing longer than this is read: a server that never ends its pages is stopped. */
const MAX_RECORDS = 2_000_000;
const TIMEOUT_MS = 60_000;
const BASE_KEY = 'arclang.import.apiBase';

export interface ApiSession {
  base: string;
  token: string | null;
}

export class ApiError extends Error {}

const secretKey = (base: string) => `arclang.import.apiToken:${new URL(base).origin}`;

async function getJson(session: ApiSession, url: string, cancel?: vscode.CancellationToken): Promise<{ body: unknown; link: string | null }> {
  if (new URL(url).origin !== new URL(session.base).origin) throw new ApiError(`refused: ${url} is not on the server ${session.base}`);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);
  const stop = cancel?.onCancellationRequested(() => controller.abort());
  try {
    const headers: Record<string, string> = { Accept: 'application/json' };
    if (session.token) headers.Authorization = `Bearer ${session.token}`;
    let response: Response;
    try {
      response = await fetch(url, { headers, redirect: 'manual', signal: controller.signal });
    } catch (error) {
      if (cancel?.isCancellationRequested) throw new ApiError('cancelled');
      throw new ApiError(`${new URL(url).origin} did not answer: ${error instanceof Error ? (error.cause instanceof Error ? error.cause.message : error.message) : String(error)}`);
    }
    if (response.status >= 300 && response.status < 400) throw new ApiError(`the server redirects ${new URL(url).pathname} (HTTP ${response.status}): refused, give the address it redirects to`);
    if (response.status === 401 || response.status === 403) throw new ApiError(`the server refuses the request (HTTP ${response.status}): a token is needed, or this one may not read`);
    if (!response.ok) throw new ApiError(`HTTP ${response.status} on ${new URL(url).pathname}${response.status === 404 ? ' (is the base address that of the API?)' : ''}`);
    const text = await response.text();
    try {
      return { body: JSON.parse(text), link: response.headers.get('link') };
    } catch {
      throw new ApiError(`${new URL(url).pathname} did not answer JSON`);
    }
  } finally {
    clearTimeout(timer);
    stop?.dispose();
  }
}

async function list(session: ApiSession, path: string, cancel?: vscode.CancellationToken, onPage?: (count: number) => void): Promise<Record<string, unknown>[]> {
  const out: Record<string, unknown>[] = [];
  let url: string | null = withPageSize(`${session.base}${path}`, PAGE_SIZE);
  const seen = new Set<string>();
  while (url) {
    if (cancel?.isCancellationRequested) throw new ApiError('cancelled');
    if (seen.has(url)) throw new ApiError(`the server gave the same page twice (${url}): stopped`);
    seen.add(url);
    const { body, link } = await getJson(session, url, cancel);
    if (!Array.isArray(body)) throw new ApiError(`${path} did not answer a list`);
    for (const item of body) if (item && typeof item === 'object') out.push(item as Record<string, unknown>);
    if (out.length > MAX_RECORDS) throw new ApiError(`more than ${MAX_RECORDS} records: stopped`);
    onPage?.(out.length);
    const next = nextPage(link, url);
    if (next !== null && typeof next === 'object') throw new ApiError(next.error);
    url = next;
  }
  return out;
}

/** The base address, asked for (the last one offered), and the token kept for its server. */
async function askSession(context: vscode.ExtensionContext): Promise<ApiSession | null> {
  const last = context.globalState.get<string>(BASE_KEY) ?? 'http://localhost:9000';
  const raw = await vscode.window.showInputBox({
    title: 'Import from a SysML v2 API server (1/4)',
    prompt: 'Base address of the Systems Modeling API: the one under which /projects is served',
    value: last,
    placeHolder: 'https://sysml.example.org/api   or   http://localhost:9000',
    ignoreFocusOut: true,
    validateInput: (value) => {
      const checked = checkApiBase(value);
      return 'error' in checked ? checked.error : null;
    },
  });
  if (raw === undefined) return null;
  const checked = checkApiBase(raw);
  if ('error' in checked) return null;
  await context.globalState.update(BASE_KEY, checked.base);
  const stored = (await context.secrets.get(secretKey(checked.base))) ?? null;
  const token = await vscode.window.showInputBox({
    title: 'Import from a SysML v2 API server (2/4)',
    prompt: stored ? 'Token of this server (kept in VS Code\'s secret storage; empty to forget it)' : 'Token of this server, if it asks for one (kept in VS Code\'s secret storage); empty for none',
    value: stored ?? '',
    password: true,
    ignoreFocusOut: true,
  });
  if (token === undefined) return null;
  const trimmed = token.trim();
  if (trimmed) await context.secrets.store(secretKey(checked.base), trimmed);
  else if (stored) await context.secrets.delete(secretKey(checked.base));
  return { base: checked.base, token: trimmed || null };
}

export interface ApiChoice {
  session: ApiSession;
  projectId: string;
  projectName: string;
  commitId: string;
}

/** Base address, token, project, commit: the four questions of an API import. */
export async function chooseApiCommit(context: vscode.ExtensionContext): Promise<ApiChoice | null> {
  const session = await askSession(context);
  if (!session) return null;
  const projects = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `ArcLang: projects of ${session.base}`, cancellable: true }, (_p, cancel) => list(session, '/projects', cancel));
  const projectItems = projects.map(recordLabel).filter((x): x is NonNullable<typeof x> => x !== null);
  if (projectItems.length === 0) throw new ApiError(`${session.base}/projects lists no project`);
  const project = await vscode.window.showQuickPick(projectItems, { title: 'Import from a SysML v2 API server (3/4)', placeHolder: `Project (${projectItems.length})`, matchOnDetail: true, ignoreFocusOut: true });
  if (!project) return null;
  const projectPath = `/projects/${encodeURIComponent(project.id)}/commits`;
  const commits = newestFirst(await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `ArcLang: commits of ${project.label}`, cancellable: true }, (_p, cancel) => list(session, projectPath, cancel)));
  const commitItems = commits.map(recordLabel).filter((x): x is NonNullable<typeof x> => x !== null);
  if (commitItems.length === 0) throw new ApiError(`the project ${project.label} has no commit`);
  const commit = commitItems.length === 1 ? commitItems[0] : await vscode.window.showQuickPick(commitItems.map((c, i) => ({ ...c, description: i === 0 ? 'latest' : undefined })), { title: 'Import from a SysML v2 API server (4/4)', placeHolder: `Commit of ${project.label} (${commitItems.length}, newest first)`, matchOnDetail: true, ignoreFocusOut: true });
  if (!commit) return null;
  return { session, projectId: project.id, projectName: project.label, commitId: commit.id };
}

/** Every element of the commit, as one JSON listing of records (what `arclang import -f sysml` reads). */
export async function fetchElements(choice: ApiChoice, progress: (text: string) => void, cancel: vscode.CancellationToken): Promise<string> {
  const path = `/projects/${encodeURIComponent(choice.projectId)}/commits/${encodeURIComponent(choice.commitId)}/elements`;
  const records = await list(choice.session, path, cancel, (count) => progress(`${count} elements read`));
  if (records.length === 0) throw new ApiError('the commit has no element');
  return JSON.stringify(records);
}
