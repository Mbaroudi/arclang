// The page of "Check simulation setup": a checklist with a button for each
// fix. Pure: the text in, the HTML out (every dynamic value is escaped).
import type { Checklist, ItemState } from './simulation';

export function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

const MARK: Record<ItemState, { glyph: string; label: string }> = {
  ok: { glyph: '✓', label: 'done' },
  todo: { glyph: '✗', label: 'to do' },
  optional: { glyph: '○', label: 'optional' },
  unknown: { glyph: '?', label: 'not known yet' },
  na: { glyph: '–', label: 'not needed' },
};

/** `banner`: an extra sentence on top (why the page opened). */
export function checklistHtml(list: Checklist, nonce: string, cspSource: string, banner = ''): string {
  const rows = list.items
    .map((item) => {
      const mark = MARK[item.state];
      const buttons = item.fixes
        .map((fix, index) => `<button class="fix" data-fix="${escapeHtml(item.id)}:${index}">${escapeHtml(fix.label)}</button>`)
        .join('');
      return `<li class="item state-${item.state}" data-item="${escapeHtml(item.id)}" data-state="${item.state}">
  <span class="mark" role="img" aria-label="${mark.label}">${mark.glyph}</span>
  <div class="body"><div class="title">${escapeHtml(item.title)}</div><div class="detail">${escapeHtml(item.detail)}</div>${buttons ? `<div class="fixes">${buttons}</div>` : ''}</div>
</li>`;
    })
    .join('\n');
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${cspSource} 'nonce-${nonce}'; script-src 'nonce-${nonce}';">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Simulation setup</title>
<style nonce="${nonce}">
  body { font-family: var(--vscode-font-family); color: var(--vscode-foreground); background: var(--vscode-editor-background); padding: 24px 32px; max-width: 760px; }
  h1 { font-size: 1.5em; font-weight: 600; margin: 0 0 6px; }
  .headline { padding: 10px 14px; border-left: 4px solid var(--vscode-charts-green); background: var(--vscode-textBlockQuote-background); margin: 14px 0 20px; }
  .headline.not-ready { border-left-color: var(--vscode-charts-orange); }
  .banner { margin: 8px 0; color: var(--vscode-descriptionForeground); }
  ul { list-style: none; padding: 0; margin: 0; }
  .item { display: flex; gap: 14px; padding: 12px 0; border-bottom: 1px solid var(--vscode-widget-border, var(--vscode-panel-border)); }
  .mark { width: 24px; height: 24px; flex: none; border-radius: 50%; display: grid; place-items: center; font-weight: 700; }
  .state-ok .mark { background: var(--vscode-charts-green); color: var(--vscode-editor-background); }
  .state-todo .mark { background: var(--vscode-charts-red); color: var(--vscode-editor-background); }
  .state-optional .mark, .state-unknown .mark, .state-na .mark { border: 1px solid var(--vscode-descriptionForeground); color: var(--vscode-descriptionForeground); }
  .title { font-weight: 600; }
  .detail { color: var(--vscode-descriptionForeground); margin-top: 2px; }
  .fixes { margin-top: 8px; display: flex; gap: 8px; flex-wrap: wrap; }
  button { font: inherit; padding: 5px 12px; border: 1px solid transparent; border-radius: 2px; cursor: pointer; color: var(--vscode-button-foreground); background: var(--vscode-button-background); }
  button:hover { background: var(--vscode-button-hoverBackground); }
  button.secondary { color: var(--vscode-button-secondaryForeground); background: var(--vscode-button-secondaryBackground); }
  button:focus-visible { outline: 1px solid var(--vscode-focusBorder); outline-offset: 2px; }
  .actions { margin-top: 20px; display: flex; gap: 8px; }
</style></head>
<body>
<h1>Simulation setup</h1>
${banner ? `<p class="banner">${escapeHtml(banner)}</p>` : ''}
<div class="headline ${list.ready ? 'ready' : 'not-ready'}" id="headline" role="status">${escapeHtml(list.headline)}</div>
<ul>
${rows}
</ul>
<div class="actions"><button class="secondary" id="recheck">Check again</button><button class="secondary" id="docs">Open the documentation</button></div>
<script nonce="${nonce}">
  const vscode = acquireVsCodeApi();
  document.addEventListener('click', (event) => {
    const target = event.target instanceof Element ? event.target.closest('button') : null;
    if (!target) return;
    if (target.dataset.fix) vscode.postMessage({ type: 'fix', fix: target.dataset.fix });
    else if (target.id === 'recheck') vscode.postMessage({ type: 'recheck' });
    else if (target.id === 'docs') vscode.postMessage({ type: 'docs' });
  });
</script>
</body></html>`;
}
