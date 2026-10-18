// ArcLang for VS Code: the language server for what the editor shows, and
// the compiler's own diagram viewer beside the source.
//
// Everything comes from the `arclang` binary (setting `arclang.binary`):
// `arclang lsp --stdio` for diagnostics, outline, completion, hover,
// definition and rename; `arclang diagram -f html` for the diagrams, drawn
// in a webview and redrawn on save; `arclang gate` and `arclang transition
// --plan` on demand.

const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');
const { execFile } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

let client = null;
let diagramPanel = null;
let diagramFile = null;

function binary() {
  return vscode.workspace.getConfiguration('arclang').get('binary') || 'arclang';
}

function run(args, cwd) {
  return new Promise((resolve) => {
    execFile(binary(), args, { cwd, maxBuffer: 64 * 1024 * 1024 }, (error, stdout, stderr) => {
      resolve({ code: error && typeof error.code === 'number' ? error.code : error ? 1 : 0, stdout: String(stdout), stderr: String(stderr) });
    });
  });
}

async function startClient(context) {
  const serverOptions = {
    command: binary(),
    args: ['lsp', '--stdio'],
    transport: TransportKind.stdio,
  };
  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'arclang' }],
    synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/*.arc') },
  };
  client = new LanguageClient('arclang', 'ArcLang language server', serverOptions, clientOptions);
  try {
    await client.start();
  } catch (error) {
    vscode.window.showErrorMessage(`ArcLang: cannot start \`${binary()} lsp --stdio\` (${error.message}). Install arclang or set arclang.binary.`);
    client = null;
  }
  if (client) context.subscriptions.push(client);
}

// --- diagrams ---------------------------------------------------------------
async function renderDiagrams(file) {
  const out = path.join(os.tmpdir(), `arclang-diagrams-${process.pid}.html`);
  const result = await run(['diagram', file, '-f', 'html', '-o', out], path.dirname(file));
  if (result.code !== 0 || !fs.existsSync(out)) {
    return { error: (result.stderr || result.stdout || 'arclang diagram failed').trim() };
  }
  const html = fs.readFileSync(out, 'utf8');
  fs.rmSync(out, { force: true });
  return { html };
}

async function showDiagrams(file) {
  diagramFile = file;
  if (!diagramPanel) {
    diagramPanel = vscode.window.createWebviewPanel('arclangDiagrams', 'ArcLang diagrams', vscode.ViewColumn.Beside, {
      enableScripts: true,
      retainContextWhenHidden: true,
    });
    diagramPanel.onDidDispose(() => { diagramPanel = null; diagramFile = null; });
  }
  diagramPanel.title = `Diagrams — ${path.basename(file)}`;
  const outcome = await renderDiagrams(file);
  if (outcome.error) {
    diagramPanel.webview.html = `<!doctype html><body style="font-family:sans-serif;padding:1rem"><h2>No diagrams</h2><pre style="white-space:pre-wrap">${escapeHtml(outcome.error)}</pre></body>`;
    return;
  }
  // The page is self-contained (the layout engine is inlined by native
  // builds); scripts run inside the webview's own origin.
  diagramPanel.webview.html = outcome.html;
  diagramPanel.reveal(vscode.ViewColumn.Beside, true);
}

function escapeHtml(text) {
  return text.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
}

// --- gate and transitions ---------------------------------------------------
async function runGate(file) {
  const result = await run(['gate', file], path.dirname(file));
  const channel = output();
  channel.clear();
  channel.appendLine(`$ arclang gate ${path.basename(file)}`);
  channel.append(result.stdout);
  if (result.stderr) channel.append(result.stderr);
  channel.show(true);
  if (result.code === 0) vscode.window.showInformationMessage('ArcLang gate: PASS');
  else vscode.window.showWarningMessage('ArcLang gate: FAIL — see the ArcLang output');
}

async function transitionPlan(file) {
  const result = await run(['transition', file, '--plan'], path.dirname(file));
  const channel = output();
  channel.clear();
  channel.appendLine(`$ arclang transition ${path.basename(file)} --plan`);
  if (result.code !== 0) {
    channel.append(result.stderr || result.stdout);
    channel.show(true);
    return;
  }
  let plans;
  try { plans = JSON.parse(result.stdout); } catch (error) { channel.append(result.stdout); channel.show(true); return; }
  for (const plan of plans) {
    const pending = plan.candidates.filter((c) => c.realized_by.length === 0);
    channel.appendLine(`\n${plan.label} (${plan.step}): ${plan.candidates.length - pending.length} realized, ${pending.length} to create${plan.blocked ? ` — ${plan.blocked}` : ''}`);
    for (const candidate of plan.candidates) {
      channel.appendLine(`  ${candidate.realized_by.length ? '✓' : '○'} ${candidate.name}${candidate.realized_by.length ? ` ← ${candidate.realized_by.join(', ')}` : ''}`);
    }
    if (pending.length > 0) channel.appendLine(`  → arclang transition ${path.basename(file)} --step ${plan.step} --write`);
  }
  channel.show(true);
}

let channelInstance = null;
function output() {
  if (!channelInstance) channelInstance = vscode.window.createOutputChannel('ArcLang');
  return channelInstance;
}

function activeArcFile() {
  const editor = vscode.window.activeTextEditor;
  if (editor && editor.document.languageId === 'arclang' && editor.document.uri.scheme === 'file') return editor.document.uri.fsPath;
  if (diagramFile) return diagramFile;
  vscode.window.showInformationMessage('ArcLang: open a .arc file first.');
  return null;
}

async function activate(context) {
  await startClient(context);
  context.subscriptions.push(
    vscode.commands.registerCommand('arclang.openDiagrams', async () => { const file = activeArcFile(); if (file) await showDiagrams(file); }),
    vscode.commands.registerCommand('arclang.runGate', async () => { const file = activeArcFile(); if (file) await runGate(file); }),
    vscode.commands.registerCommand('arclang.transitionPlan', async () => { const file = activeArcFile(); if (file) await transitionPlan(file); }),
    vscode.commands.registerCommand('arclang.restartServer', async () => {
      if (client) { await client.stop(); client = null; }
      await startClient(context);
    }),
    // The diagrams follow the file: every save redraws them.
    vscode.workspace.onDidSaveTextDocument(async (document) => {
      if (diagramPanel && diagramFile && document.uri.fsPath === diagramFile) await showDiagrams(diagramFile);
    }),
  );
}

async function deactivate() {
  if (client) await client.stop();
}

module.exports = { activate, deactivate };
