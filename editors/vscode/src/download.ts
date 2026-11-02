// A file exported for the user's computer (a PDF, a deck, a table, a
// document...): on the desktop it is on the disk where the save dialog put
// it; in the browser the workspace lives in the page, so the file is also
// handed to the browser as a download (the page's `arclangWeb.download`).
// "Reveal" shows it in the Explorer there, the operating system's file
// manager on the desktop.
import * as vscode from 'vscode';

const PAGE_DOWNLOAD = 'arclangWeb.download';
let pageDownloads: boolean | null = null;

async function canDownload(): Promise<boolean> {
  pageDownloads ??= (await vscode.commands.getCommands(true)).includes(PAGE_DOWNLOAD);
  return pageDownloads;
}

const toBase64 = (bytes: Uint8Array): string => {
  let text = '';
  for (let i = 0; i < bytes.length; i += 0x8000) text += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(text);
};

/** Hands `uri` (just written) to the browser as a download, where there is one; whether it did. */
export async function deliver(uri: vscode.Uri, bytes?: Uint8Array): Promise<boolean> {
  if (!(await canDownload())) return false;
  const data = bytes ?? (await vscode.workspace.fs.readFile(uri));
  const name = uri.path.slice(uri.path.lastIndexOf('/') + 1);
  await vscode.commands.executeCommand(PAGE_DOWNLOAD, { name, base64: toBase64(data) });
  return true;
}

/** Shows a written file: in the operating system's file manager, or in the Explorer in the browser. */
export async function reveal(uri: vscode.Uri): Promise<void> {
  if (await canDownload()) await vscode.commands.executeCommand('revealInExplorer', uri);
  else await vscode.commands.executeCommand('revealFileInOS', uri);
}
