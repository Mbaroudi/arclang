// The Modelica drawing of a model, in VS Code itself: the scenarios and the
// configurations' systems drawn as OMEdit draws them (the icons of the
// Standard Library, the placements, the connections), from OpenModelica's
// instance of the class (`arclang verify --drawing CLASS`). The compiler
// runs on the desktop, online in the browser: the view is the same. A part
// opens its own class's diagram; the path back is at the top.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run } from '../cli';
import { drawDiagram, type Drawing } from '../pure/modelica-drawing';

export const MODELICA_VIEW = 'arclang.modelicaDiagram';

interface Drawable {
  class: string;
  label: string;
  kind: 'scenario' | 'system';
}

/** A level of the path: the class drawn, and what it is called there. */
interface Level {
  className: string;
  label: string;
}

type FromPage = { type: 'ready' } | { type: 'open'; name: string } | { type: 'crumb'; index: number } | { type: 'source' };

class ModelicaView {
  private static current: ModelicaView | null = null;
  private readonly cache = new Map<string, Drawing>();
  private levels: Level[] = [];
  private drawing: Drawing | null = null;
  private ready = false;

  private constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly panel: vscode.WebviewPanel,
    private file: string,
  ) {
    panel.webview.html = this.html();
    panel.onDidDispose(() => {
      if (ModelicaView.current === this) ModelicaView.current = null;
    });
    panel.webview.onDidReceiveMessage((message: FromPage) => void this.receive(message));
  }

  static show(context: vscode.ExtensionContext, file: string, start: Level): void {
    let view = ModelicaView.current;
    if (!view || view.file !== file) {
      view?.panel.dispose();
      const panel = vscode.window.createWebviewPanel(MODELICA_VIEW, 'Modelica drawing', vscode.ViewColumn.Active, {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')],
      });
      view = new ModelicaView(context, panel, file);
      ModelicaView.current = view;
    }
    view.panel.reveal();
    view.levels = [start];
    void view.draw();
  }

  private async receive(message: FromPage): Promise<void> {
    if (message.type === 'ready') {
      this.ready = true;
      if (this.drawing) this.post();
    } else if (message.type === 'crumb') {
      this.levels = this.levels.slice(0, Math.max(1, message.index + 1));
      await this.draw();
    } else if (message.type === 'open') {
      const part = this.drawing?.components.find((c) => c.name === message.name);
      if (!part?.opens) return;
      this.levels = [...this.levels, { className: part.className, label: part.name }];
      await this.draw();
    } else if (message.type === 'source') {
      await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(this.file));
    }
  }

  private async draw(): Promise<void> {
    const level = this.levels[this.levels.length - 1];
    this.panel.title = `Modelica: ${level.label}`;
    const cached = this.cache.get(level.className);
    if (cached) {
      this.drawing = cached;
      return this.post();
    }
    void this.panel.webview.postMessage({ type: 'busy', text: `OpenModelica reads ${level.className}…` });
    const result = await run(['verify', this.file, '--drawing', level.className], { cwd: path.dirname(this.file) });
    let instance: unknown = null;
    try {
      instance = result.code === 0 ? JSON.parse(result.stdout) : null;
    } catch {
      instance = null;
    }
    if (!instance) {
      const why = failureText(result).trim().split('\n').slice(0, 6).join('\n');
      void this.panel.webview.postMessage({ type: 'error', text: `OpenModelica could not give the drawing of ${level.className}.\n${why}` });
      return;
    }
    this.drawing = drawDiagram(instance);
    this.cache.set(level.className, this.drawing);
    this.post();
  }

  private post(): void {
    if (!this.ready || !this.drawing) return;
    const level = this.levels[this.levels.length - 1];
    void this.panel.webview.postMessage({
      type: 'drawing',
      title: level.label,
      className: level.className,
      crumbs: this.levels.map((l) => l.label),
      drawing: this.drawing,
    });
  }

  private html(): string {
    const webview = this.panel.webview;
    const nonce = randomBytes(16).toString('base64url');
    const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...segments)).toString();
    const csp = ["default-src 'none'", `style-src ${webview.cspSource}`, 'img-src data:', `script-src 'nonce-${nonce}'`, "font-src 'none'", "connect-src 'none'", "frame-src 'none'", "form-action 'none'", "base-uri 'none'"].join('; ');
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="${uri('media', 'modelica.css')}">
<title>Modelica drawing</title>
</head>
<body>
<header class="bar" role="toolbar" aria-label="Modelica drawing">
  <nav id="crumbs" class="crumbs" aria-label="Path"></nav>
  <span class="spacer"></span>
  <span id="summary" class="summary"></span>
  <span class="group">
    <button id="out" title="Zoom out (−)" aria-label="Zoom out">−</button>
    <button id="fit" title="Fit the drawing (0)">Fit</button>
    <button id="in" title="Zoom in (+)" aria-label="Zoom in">+</button>
  </span>
  <button id="source" title="The model's source">Source</button>
</header>
<main id="stage" class="stage" tabindex="0" aria-label="Diagram"></main>
<div id="tip" class="tip" role="tooltip" hidden></div>
<p id="status" class="status" role="status"></p>
<script nonce="${nonce}" src="${uri('dist', 'modelica.js')}"></script>
</body>
</html>`;
  }
}

/** The classes worth drawing in a model, or null once the user has been told why there are none. */
async function drawablesOf(file: string): Promise<Drawable[] | null> {
  const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Looking for the Modelica of the model…' }, () =>
    run(['verify', file, '--drawings'], { cwd: path.dirname(file) }),
  );
  let classes: Drawable[] = [];
  try {
    classes = (JSON.parse(result.stdout) as { classes: Drawable[] }).classes ?? [];
  } catch {
    void vscode.window.showErrorMessage(`ArcLang: the Modelica of the model could not be listed. ${failureText(result).split('\n')[0]}`);
    return null;
  }
  if (classes.length === 0) {
    void vscode.window.showInformationMessage('ArcLang: this model has no physics to draw yet: a test case that simulates (simulate: { model: … }) or a configuration.');
    return null;
  }
  return classes;
}

/** Asks which scenario or configuration to draw, then draws it. */
export async function openModelicaDrawing(context: vscode.ExtensionContext, file: string, preferred?: string): Promise<void> {
  const classes = await drawablesOf(file);
  if (!classes) return;
  const chosen = preferred ? classes.find((c) => c.class === preferred) : undefined;
  const picked =
    chosen ??
    (classes.length === 1
      ? classes[0]
      : await vscode.window
          .showQuickPick(
            classes.map((c) => ({ label: c.label, description: c.class, detail: c.kind === 'scenario' ? 'a simulated scenario' : 'a configuration of the model', drawable: c })),
            { placeHolder: 'What to draw', matchOnDescription: true },
          )
          .then((p) => p?.drawable));
  if (!picked) return;
  ModelicaView.show(context, file, { className: picked.class, label: picked.label });
}
