// The digital twins of a model in VS Code: `arclang twin --json` runs each
// twin (its physics driven by the measured data in OpenModelica), and the
// view shows, per twin, its verdict, each comparison's curves (the
// measurement, the model, the tolerance band, the first instant beyond)
// and the requirements it checks on the run.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run } from '../cli';
import { parseTwinOutcomes, twinSummary, type TwinOutcome } from '../pure/twin';

export const TWIN_VIEW = 'arclang.twin';

type FromPage = { type: 'ready' } | { type: 'rerun' } | { type: 'data'; file: string } | { type: 'log' };

let output: vscode.OutputChannel | undefined;
const log = () => (output ??= vscode.window.createOutputChannel('ArcLang Twin'));

class TwinView {
  private static current: TwinView | null = null;
  private outcomes: TwinOutcome[] = [];
  private ready = false;

  private constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly panel: vscode.WebviewPanel,
    private file: string,
    private readonly rerun: () => void,
  ) {
    panel.webview.html = this.html();
    panel.onDidDispose(() => {
      if (TwinView.current === this) TwinView.current = null;
    });
    panel.webview.onDidReceiveMessage((message: FromPage) => {
      if (message.type === 'ready') {
        this.ready = true;
        this.post();
      } else if (message.type === 'rerun') this.rerun();
      else if (message.type === 'log') log().show(true);
      else if (message.type === 'data' && !path.isAbsolute(message.file) && !message.file.split(/[\\/]/).includes('..')) {
        void vscode.commands.executeCommand('vscode.open', vscode.Uri.file(path.join(path.dirname(this.file), message.file)));
      }
    });
  }

  static show(context: vscode.ExtensionContext, file: string, outcomes: TwinOutcome[], rerun: () => void): void {
    let view = TwinView.current;
    if (!view || view.file !== file) {
      view?.panel.dispose();
      const panel = vscode.window.createWebviewPanel(TWIN_VIEW, `Twins: ${path.basename(file)}`, vscode.ViewColumn.Beside, {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')],
      });
      view = new TwinView(context, panel, file, rerun);
      TwinView.current = view;
    }
    view.outcomes = outcomes;
    view.panel.reveal(undefined, true);
    view.post();
  }

  static busy(text: string): void {
    void TwinView.current?.panel.webview.postMessage({ type: 'busy', text });
  }

  private post(): void {
    if (!this.ready) return;
    // The page draws the charts at the width it has (src/pure/twin.ts, escaped).
    void this.panel.webview.postMessage({ type: 'twins', model: path.basename(this.file), twins: this.outcomes });
  }

  private html(): string {
    const webview = this.panel.webview;
    const nonce = randomBytes(16).toString('base64url');
    const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...segments)).toString();
    const csp = ["default-src 'none'", `style-src ${webview.cspSource}`, "img-src 'none'", `script-src 'nonce-${nonce}'`, "font-src 'none'", "connect-src 'none'", "frame-src 'none'", "form-action 'none'", "base-uri 'none'"].join('; ');
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="${uri('media', 'twin.css')}">
<title>Digital twins</title>
</head>
<body>
<header class="bar"><h1 id="title">Digital twins</h1><span class="spacer"></span><button id="log">Log</button><button id="rerun" class="primary">Run again</button></header>
<main id="twins" aria-live="polite"></main>
<p id="status" class="status" role="status"></p>
<script nonce="${nonce}" src="${uri('dist', 'twin.js')}"></script>
</body>
</html>`;
  }
}

/** The model a twin run is about: the argument, the active .arc editor, or the workbench's tab. */
function activeModel(arg: unknown): string | null {
  if (arg instanceof vscode.Uri && arg.path.endsWith('.arc')) return arg.fsPath;
  const editor = vscode.window.activeTextEditor;
  if (editor?.document.languageId === 'arclang') return editor.document.uri.fsPath;
  const tab = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  return tab?.uri?.fsPath.endsWith('.arc') ? tab.uri.fsPath : null;
}

export async function runTwins(context: vscode.ExtensionContext, arg: unknown, env: () => Record<string, string>): Promise<void> {
  const file = activeModel(arg);
  if (!file) return void vscode.window.showInformationMessage('ArcLang: open a .arc model with a twin (twin "Name" { simulate: … data: … drives: … }).');
  const document = vscode.workspace.textDocuments.find((d) => d.uri.fsPath === file);
  if (document?.isDirty) await document.save();
  if (document && !/^\s*twin\s+"/m.test(document.getText())) {
    return void vscode.window.showInformationMessage('ArcLang: this model declares no twin: twin "Name" { simulate: { model: … } data: { file: … } drives: [ … ] compares: [ … ] }.');
  }
  TwinView.busy('Running the twins in OpenModelica…');
  const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Running the digital twins in OpenModelica…', cancellable: true }, (_p, token) =>
    run(['twin', file, '--json'], { cwd: path.dirname(file), token, env: env() }),
  );
  const outcomes = parseTwinOutcomes(result.stdout);
  const channel = log();
  channel.appendLine(`── ${new Date().toLocaleTimeString()}  arclang twin ${path.basename(file)}`);
  if (!outcomes) {
    channel.appendLine(failureText(result));
    channel.show(true);
    return void vscode.window.showErrorMessage(`ArcLang: the twins could not run. ${failureText(result).split('\n')[0]}`);
  }
  for (const o of outcomes) {
    channel.appendLine(`twin '${o.name}': ${o.verdict.toUpperCase()}${o.reason ? `: ${o.reason}` : ''}`);
    if (o.log && o.verdict !== 'pass') channel.appendLine(o.log);
  }
  TwinView.show(context, file, outcomes, () => void runTwins(context, vscode.Uri.file(file), env));
  const failed = outcomes.some((o) => o.verdict !== 'pass');
  void (failed ? vscode.window.showWarningMessage : vscode.window.showInformationMessage)(`ArcLang: ${twinSummary(outcomes)}.`);
}
