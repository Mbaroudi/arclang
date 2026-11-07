// `vscode.workspace.findFiles`, its answer filtered by the pattern: in the
// browser build the file search answers every path of the workspace, folders
// included, whatever the pattern; on the desktop this changes nothing.
import * as vscode from 'vscode';
import { filterPaths } from './pure/glob';

export async function findFiles(include: vscode.GlobPattern, exclude?: vscode.GlobPattern | null, maxResults?: number): Promise<vscode.Uri[]> {
  const found = await vscode.workspace.findFiles(include, exclude, maxResults === undefined ? undefined : maxResults * 4);
  const pattern = typeof include === 'string' ? include : include.pattern;
  const base = typeof include === 'string' ? null : typeof include.baseUri === 'object' ? include.baseUri.path : String(include.base);
  const excluded = exclude === undefined || exclude === null ? null : typeof exclude === 'string' ? exclude : exclude.pattern;
  const rel = (uri: vscode.Uri) => (base ? (uri.path.startsWith(`${base}/`) ? uri.path.slice(base.length + 1) : '') : vscode.workspace.asRelativePath(uri, false));
  const keep = new Set(filterPaths(found.map(rel).filter(Boolean), pattern, excluded));
  const out = found.filter((uri) => keep.has(rel(uri)));
  return maxResults === undefined ? out : out.slice(0, maxResults);
}
