// The report of an import, in a tab of its own: what the compiler counted
// and what it left out (and why), its notes, and whether the written model
// compiles; or the refusal and the text it refused. Buttons go on: open the
// workbench, open the model, check it.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { countTotals, ImportOutcome } from '../pure/importing';

export interface ReportData {
  source: string;
  format: string;
  target: string;
  /** Requirements added to an open model (not saved yet): that model. */
  mergedInto?: string;
  outcome: ImportOutcome;
  /** Whether the written model compiles (`arclang check`), and its errors when not; null when not checked. */
  compiles: boolean | null;
  checkErrors: string[];
  findings: number;
  errorFindings: number;
  metrics: Record<string, number>;
  /** How the source chosen became the compiler's input, where the result went. */
  notes: string[];
}

export const REPORT_VIEW = 'arclang.importReport';
let current: vscode.WebviewPanel | null = null;
/** The buttons of the report on screen: a new report replaces them. */
let listener: vscode.Disposable | null = null;

const escape = (text: string) => text.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] ?? c);

/** The verdict in one sentence, for the report and the notification. */
export function verdict(data: ReportData): string {
  const name = path.basename(data.target);
  if (!data.outcome.ok) return /would not compile|refused the edit|no declaration/.test(data.outcome.error ?? '') ? `The requirements were not added: ${data.outcome.error}` : `The import was refused: nothing was written to ${name}.`;
  const errors = data.errorFindings > 0 ? `, but the check reports ${data.errorFindings} error finding${data.errorFindings > 1 ? 's' : ''} (Run check lists them)` : '';
  if (data.mergedInto) return `The requirements were added to ${name}, which compiles${errors}; it is not saved yet.`;
  if (data.compiles === true) return `${name} was written and compiles${errors}.`;
  if (data.compiles === false) return `${name} was written but does not compile.`;
  return `${name} was written.`;
}

function html(data: ReportData): string {
  const nonce = randomBytes(16).toString('base64url');
  const { outcome } = data;
  const imported = outcome.counts.filter((c) => !c.leftOut);
  const leftOut = outcome.counts.filter((c) => c.leftOut);
  const totals = countTotals(outcome.counts);
  const rows = (items: typeof imported) => items.map((c) => `<tr><td class="n">${c.count}</td><td>${escape(c.what)}</td></tr>`).join('');
  const metrics = Object.entries(data.metrics).filter(([, v]) => typeof v === 'number');
  const state = !outcome.ok ? 'refused' : data.compiles === false ? 'broken' : data.errorFindings > 0 ? 'warn' : 'ok';
  const list = (items: string[]) => (items.length ? `<ul>${items.map((n) => `<li>${escape(n)}</li>`).join('')}</ul>` : '');
  return `<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}'">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Import report</title>
<style nonce="${nonce}">
  :root { color-scheme: light dark; }
  body { font-family: var(--vscode-font-family); font-size: var(--vscode-font-size); color: var(--vscode-foreground); background: var(--vscode-editor-background); margin: 0; padding: 20px 24px 40px; max-width: 920px; }
  h1 { font-size: 1.5em; font-weight: 600; margin: 0 0 4px; }
  .from { color: var(--vscode-descriptionForeground); margin: 0 0 16px; word-break: break-all; }
  .verdict { display: flex; gap: 10px; align-items: baseline; padding: 10px 14px; border-radius: 6px; border-left: 4px solid; margin: 0 0 16px; font-size: 1.1em; }
  .verdict.ok { border-color: var(--vscode-testing-iconPassed, #3a3); background: color-mix(in srgb, var(--vscode-testing-iconPassed, #3a3) 12%, transparent); }
  .verdict.warn { border-color: var(--vscode-editorWarning-foreground, #c90); background: color-mix(in srgb, var(--vscode-editorWarning-foreground, #c90) 12%, transparent); }
  .verdict.broken, .verdict.refused { border-color: var(--vscode-errorForeground, #c33); background: color-mix(in srgb, var(--vscode-errorForeground, #c33) 12%, transparent); }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 0 0 20px; }
  button { font: inherit; padding: 5px 12px; border-radius: 3px; border: 1px solid var(--vscode-button-border, transparent); color: var(--vscode-button-secondaryForeground); background: var(--vscode-button-secondaryBackground); cursor: pointer; }
  button.primary { color: var(--vscode-button-foreground); background: var(--vscode-button-background); }
  button:hover { filter: brightness(1.1); }
  button:focus-visible { outline: 1px solid var(--vscode-focusBorder); outline-offset: 2px; }
  .totals { display: flex; gap: 24px; margin: 0 0 12px; }
  .total b { display: block; font-size: 1.8em; font-variant-numeric: tabular-nums; font-weight: 600; }
  .total span { color: var(--vscode-descriptionForeground); }
  .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px 28px; }
  h2 { font-size: 1em; font-weight: 600; margin: 18px 0 6px; text-transform: uppercase; letter-spacing: .04em; color: var(--vscode-descriptionForeground); }
  table { border-collapse: collapse; width: 100%; }
  td { padding: 2px 6px; border-bottom: 1px solid var(--vscode-widget-border, #8883); vertical-align: top; }
  td.n { text-align: right; font-variant-numeric: tabular-nums; width: 4em; }
  .left td { color: var(--vscode-editorWarning-foreground, inherit); }
  ul { margin: 0; padding-left: 20px; } li { margin: 2px 0; }
  pre { white-space: pre-wrap; background: var(--vscode-textCodeBlock-background); padding: 8px; border-radius: 4px; }
</style></head>
<body data-state="${state}">
<h1>Import report</h1>
<p class="from">${escape(path.basename(data.source))} (${escape(data.format)}) → ${escape(data.target)}</p>
<p class="verdict ${state}" role="status" id="verdict">${escape(verdict(data))}</p>
<div class="actions">
  ${outcome.ok ? `<button class="primary" data-action="workbench">Open workbench</button><button data-action="open">Open model</button><button data-action="check">Run check</button>` : ''}
  ${outcome.rejected ? `<button data-action="rejected">Open the refused text</button>` : ''}
  <button data-action="output">Show the compiler output</button>
</div>
${outcome.ok && outcome.counts.length ? `<div class="totals"><div class="total"><b id="imported">${totals.imported}</b><span>counted as imported</span></div><div class="total"><b id="left-out">${totals.leftOut}</b><span>left out</span></div>${data.compiles !== null ? `<div class="total"><b>${data.findings}</b><span>findings of the check</span></div>` : ''}</div>` : ''}
${!outcome.ok ? `<h2>Refusal</h2><pre id="refusal">${escape(outcome.error ?? 'arclang import failed')}</pre>${outcome.rejected ? `<p>The text the compiler refused is kept in <code>${escape(outcome.rejected)}</code>.</p>` : ''}` : ''}
${data.compiles === false ? `<h2>Why it does not compile</h2>${list(data.checkErrors)}` : ''}
${data.notes.length ? `<h2>How it was read</h2>${list(data.notes)}` : ''}
<div class="grid">
${imported.length ? `<section><h2>Imported</h2><table id="counts">${rows(imported)}</table></section>` : ''}
${leftOut.length ? `<section><h2>Left out, and why</h2><table class="left" id="left">${rows(leftOut)}</table></section>` : ''}
${metrics.length ? `<section><h2>In the written model</h2><table id="metrics">${metrics.map(([k, v]) => `<tr><td class="n">${v}</td><td>${escape(k)}</td></tr>`).join('')}</table></section>` : ''}
</div>
${outcome.notes.length ? `<h2>Notes of the compiler</h2>${list(outcome.notes)}` : ''}
${outcome.warnings.length ? `<h2>Warnings</h2>${list(outcome.warnings)}` : ''}
<script nonce="${nonce}">
  const vscode = acquireVsCodeApi();
  for (const b of document.querySelectorAll('button[data-action]')) b.addEventListener('click', () => vscode.postMessage({ action: b.dataset.action }));
  document.querySelector('button.primary, button')?.focus();
</script>
</body></html>`;
}

/** Show the report (one tab, reused); the buttons act on the model written. */
export function showReport(data: ReportData, showOutput: () => void): void {
  const title = `Import: ${path.basename(data.target)}`;
  if (!current) {
    current = vscode.window.createWebviewPanel(REPORT_VIEW, title, vscode.ViewColumn.Active, { enableScripts: true, enableCommandUris: false, localResourceRoots: [] });
    current.onDidDispose(() => {
      current = null;
      listener?.dispose();
      listener = null;
    });
  }
  const panel = current;
  panel.title = title;
  panel.webview.html = html(data);
  const target = vscode.Uri.file(data.target);
  listener?.dispose();
  listener = panel.webview.onDidReceiveMessage(async (message: { action?: string }) => {
    switch (message?.action) {
      case 'open':
        await vscode.window.showTextDocument(target, { preview: false });
        return;
      case 'workbench':
        await vscode.commands.executeCommand('vscode.openWith', target, 'arclang.workbench', { viewColumn: vscode.ViewColumn.Active, preview: false });
        return;
      case 'check':
        // Requirements merged and not saved yet: the check runs on the open editor.
        if (data.mergedInto) {
          await vscode.window.showTextDocument(target, { preview: false });
          await vscode.commands.executeCommand('arclang.check');
        } else await vscode.commands.executeCommand('arclang.check', target);
        return;
      case 'rejected':
        if (data.outcome.rejected) await vscode.window.showTextDocument(vscode.Uri.file(path.resolve(path.dirname(data.target), data.outcome.rejected)), { preview: false });
        return;
      case 'output':
        showOutput();
        return;
      default:
        return;
    }
  });
  panel.reveal(vscode.ViewColumn.Active, false);
}
