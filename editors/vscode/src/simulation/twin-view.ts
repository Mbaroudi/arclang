// The digital twins of a model in VS Code: `arclang twin --json` runs each
// twin (its physics driven by the measured data in OpenModelica), and the
// view shows, per twin, its verdict, each comparison's curves (the
// measurement, the model, the tolerance band, the first instant beyond)
// and the requirements it checks on the run.
import { spawn, type ChildProcess } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { binary, failureText, run } from '../cli';
import { parseTwinOutcomes, twinSummary, type TwinOutcome } from '../pure/twin';

export const TWIN_VIEW = 'arclang.twin';

type FromPage = { type: 'ready' } | { type: 'rerun' } | { type: 'data'; file: string } | { type: 'log' } | { type: 'stop' };

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
    private readonly folder: vscode.Uri,
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
      else if (message.type === 'stop') stopLive();
      else if (message.type === 'data' && !path.isAbsolute(message.file) && !message.file.split(/[\\/]/).includes('..')) {
        void vscode.commands.executeCommand('vscode.open', vscode.Uri.joinPath(this.folder, ...message.file.split(/[\\/]/)));
      }
    });
  }

  /** `folder`: the model's folder, where the view opens a twin's data (on disk, or in the browser's workspace). */
  static show(context: vscode.ExtensionContext, file: string, outcomes: TwinOutcome[], rerun: () => void, folder = vscode.Uri.file(path.dirname(file))): void {
    let view = TwinView.current;
    if (!view || view.file !== file) {
      view?.panel.dispose();
      const panel = vscode.window.createWebviewPanel(TWIN_VIEW, `Twins: ${path.basename(file)}`, vscode.ViewColumn.Beside, {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')],
      });
      view = new TwinView(context, panel, file, rerun, folder);
      TwinView.current = view;
    }
    view.outcomes = outcomes;
    view.panel.reveal(undefined, true);
    view.post();
  }

  static busy(text: string): void {
    void TwinView.current?.panel.webview.postMessage({ type: 'busy', text });
  }

  /** The state of a live run, for the banner: running, and what the connection said last. */
  static live(running: boolean, status: string, detail: string): void {
    void TwinView.current?.panel.webview.postMessage({ type: 'live', running, status, detail });
  }

  static onDispose(listener: () => void): void {
    TwinView.current?.panel.onDidDispose(listener);
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
<header class="bar"><h1 id="title">Digital twins</h1><span class="spacer"></span><button id="log">Log</button><button id="stop" hidden>Stop</button><button id="rerun" class="primary">Run again</button></header>
<p id="live" class="live" role="status" hidden><span class="dot" aria-hidden="true"></span><span id="live-text"></span></p>
<main id="twins" aria-live="polite"></main>
<p id="status" class="status" role="status"></p>
<script nonce="${nonce}" src="${uri('dist', 'twin.js')}"></script>
</body>
</html>`;
  }
}

/** The twins of a run made elsewhere (online, in the browser build): the same view, its data opened in the model's folder. */
export function showTwinOutcomes(context: vscode.ExtensionContext, model: vscode.Uri, outcomes: TwinOutcome[], rerun: () => void): void {
  TwinView.show(context, model.fsPath, outcomes, rerun, vscode.Uri.joinPath(model, '..'));
}

/** Says, on the view if it is open, that a run is under way. */
export function twinsBusy(text: string): void {
  TwinView.busy(text);
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

// ---- Live ----------------------------------------------------------------------

let watching: ChildProcess | null = null;

/** Asks the run to stop (it records what is waiting), then ends it if it lingers. */
function stopLive(): void {
  const child = watching;
  if (child && child.exitCode === null) {
    child.stdin?.write('stop\n');
    setTimeout(() => {
      if (child.exitCode === null) child.kill();
    }, 5000);
  }
  watching = null;
  TwinView.live(false, 'stopped', 'The live run is stopped.');
}

/**
 * Follows a twin's live data (`arclang twin --live --json`): the view is
 * updated at every run of the twin, the banner says how the connection
 * goes; Stop (or closing the view) ends it.
 */
export async function watchTwin(context: vscode.ExtensionContext, arg: unknown, env: () => Record<string, string>): Promise<void> {
  const file = activeModel(arg);
  if (!file) return void vscode.window.showInformationMessage('ArcLang: open a .arc model with a twin on live data (data: { mqtt: … }, { opcua: … }, { kafka: … } or { http: … }).');
  const listed = await run(['twin', file, '--list'], { cwd: path.dirname(file) });
  let twins: { id: string; name: string; live: boolean }[] = [];
  try {
    twins = JSON.parse(listed.stdout) as { id: string; name: string; live: boolean }[];
  } catch {
    return void vscode.window.showErrorMessage(`ArcLang: the model's twins could not be listed. ${failureText(listed).split('\n')[0]}`);
  }
  const live = twins.filter((t) => t.live);
  if (live.length === 0) return void vscode.window.showInformationMessage('ArcLang: no twin of this model reads live data: data: { mqtt: "mqtt://broker:1883" topics: ["site/#"] }, { opcua: "opc.tcp://plc:4840" nodes: { room_c: "ns=2;s=Room.T" } }, { kafka: "kafka://broker:9092" topics: ["house.telemetry"] } or { http: "https://api.example/house/latest" }.');
  const picked = live.length === 1 ? live[0] : await vscode.window.showQuickPick(live.map((t) => ({ label: t.name, description: t.id, twin: t })), { placeHolder: 'The twin to follow live' }).then((p) => p?.twin);
  if (!picked) return;
  stopLive();
  const channel = log();
  channel.appendLine(`── ${new Date().toLocaleTimeString()}  arclang twin ${path.basename(file)} --live --twin ${picked.id}`);
  TwinView.show(context, file, [], () => void watchTwin(context, vscode.Uri.file(file), env));
  TwinView.live(true, 'connecting', `Connecting to the live data of ${picked.name}…`);
  TwinView.onDispose(() => stopLive());
  const child = spawn(binary(), ['twin', file, '--live', '--json', '--twin', picked.id], { cwd: path.dirname(file), env: { ...process.env, ...env() } });
  watching = child;
  let pending = '';
  child.stdout?.on('data', (chunk: Buffer) => {
    if (watching !== child) return; // stopped: what it still says is not news
    pending += chunk.toString('utf8');
    const lines = pending.split('\n');
    pending = lines.pop() ?? '';
    for (const line of lines.filter((l) => l.trim())) {
      let message: unknown;
      try {
        message = JSON.parse(line);
      } catch {
        channel.appendLine(line);
        continue;
      }
      const record = message as { verdict?: string; status?: string; detail?: string };
      if (record.verdict !== undefined) {
        const outcome = message as TwinOutcome;
        channel.appendLine(`${new Date().toLocaleTimeString()} twin '${outcome.name}': ${outcome.verdict.toUpperCase()} over ${outcome.samples} samples${outcome.reason && outcome.verdict !== 'pass' ? `: ${outcome.reason}` : ''}`);
        TwinView.show(context, file, [outcome], () => void watchTwin(context, vscode.Uri.file(file), env));
        TwinView.live(true, 'running', `Live: last run at ${new Date().toLocaleTimeString()}, ${outcome.samples} samples in the window.`);
      } else if (record.status) {
        channel.appendLine(`${record.status}: ${record.detail ?? ''}`);
        TwinView.live(true, record.status, record.detail ?? '');
      }
    }
  });
  child.stderr?.on('data', (chunk: Buffer) => channel.append(chunk.toString('utf8')));
  child.on('exit', (code) => {
    if (watching === child) {
      watching = null;
      TwinView.live(false, 'stopped', code ? `The live run ended (code ${code}): see the ArcLang Twin output.` : 'The live run ended.');
    }
  });
}

export function stopWatchingTwins(): void {
  stopLive();
}
