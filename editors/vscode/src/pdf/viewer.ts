// A PDF opened in VS Code is shown as a document, not as its bytes: the
// diagrams exported to PDF, the simulation report, any PDF of the workspace.
// The pages are drawn by pdf.js in a webview (webview/pdf/main.ts), the same
// on the desktop and in the browser. Read-only: the file is never written.
import { randomBytes } from 'node:crypto';
import * as vscode from 'vscode';
import { deliver, reveal } from '../download';
import type { FromPdfView } from '../pure/pdf-view';

export const PDF_VIEW_TYPE = 'arclang.pdf';

class PdfDocument implements vscode.CustomDocument {
  constructor(readonly uri: vscode.Uri) {}
  dispose(): void {}
}

function html(context: vscode.ExtensionContext, webview: vscode.Webview): string {
  const nonce = randomBytes(16).toString('base64url');
  const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(context.extensionUri, ...segments)).toString();
  // pdf.js runs its parser in a worker made from the script's text (a blob: a webview cannot start a worker from its resources).
  const csp = ["default-src 'none'", `style-src ${webview.cspSource}`, `font-src ${webview.cspSource} data: blob:`, `img-src ${webview.cspSource} data: blob:`, `script-src 'nonce-${nonce}'`, 'worker-src blob:', `connect-src ${webview.cspSource}`, "frame-src 'none'", "form-action 'none'", "base-uri 'none'"].join('; ');
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="${uri('media', 'pdf.css')}">
<title>PDF</title>
</head>
<body data-worker="${uri('dist', 'pdfjs', 'pdf.worker.min.mjs')}" data-fonts="${uri('dist', 'pdfjs', 'standard_fonts')}/">
<header class="bar" role="toolbar" aria-label="PDF">
  <span class="pages" aria-live="polite"><input id="page" type="number" min="1" value="1" aria-label="Page"> <span id="count">/ …</span></span>
  <span class="group">
    <button id="out" title="Zoom out (−)" aria-label="Zoom out">−</button>
    <button id="zoom" class="zoom" title="Actual size (0)">100 %</button>
    <button id="in" title="Zoom in (+)" aria-label="Zoom in">+</button>
  </span>
  <span class="group">
    <button id="width" title="Fit the width (W)" aria-pressed="true">Width</button>
    <button id="page-fit" title="Fit the page (P)" aria-pressed="false">Page</button>
  </span>
  <span class="spacer"></span>
  <button id="save" title="Download, or open in the system's PDF viewer">Open outside</button>
</header>
<main id="doc" tabindex="0" aria-label="Pages"></main>
<p id="status" class="status" role="status"></p>
<script nonce="${nonce}" src="${uri('dist', 'pdf.js')}"></script>
</body>
</html>`;
}

async function outside(uri: vscode.Uri): Promise<void> {
  if (await deliver(uri)) return;
  // The desktop: the operating system's viewer, else the file in its folder.
  if (!(await vscode.env.openExternal(uri))) await reveal(uri);
}

export function registerPdfViewer(context: vscode.ExtensionContext): void {
  const provider: vscode.CustomReadonlyEditorProvider<PdfDocument> = {
    openCustomDocument: (uri) => new PdfDocument(uri),
    async resolveCustomEditor(document, panel) {
      panel.webview.options = { enableScripts: true, localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')] };
      panel.webview.html = html(context, panel.webview);
      const send = async () => {
        try {
          const bytes = await vscode.workspace.fs.readFile(document.uri);
          // As base64: a typed array from the desktop's extension host reached the page empty.
          await panel.webview.postMessage({ type: 'pdf', name: document.uri.path.split('/').pop() ?? 'document.pdf', base64: Buffer.from(bytes).toString('base64') });
        } catch (error) {
          await panel.webview.postMessage({ type: 'error', text: `The PDF cannot be read: ${error instanceof Error ? error.message : String(error)}` });
        }
      };
      // A PDF written again (an export over it) is drawn again.
      const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(vscode.Uri.joinPath(document.uri, '..'), document.uri.path.split('/').pop() ?? '*.pdf'));
      watcher.onDidChange(() => void send());
      panel.onDidDispose(() => watcher.dispose());
      panel.webview.onDidReceiveMessage((message: FromPdfView) => {
        if (message.type === 'ready') void send();
        else if (message.type === 'outside') void outside(document.uri);
      });
    },
  };
  context.subscriptions.push(vscode.window.registerCustomEditorProvider(PDF_VIEW_TYPE, provider, { webviewOptions: { retainContextWhenHidden: true }, supportsMultipleEditorsPerDocument: true }));
}
