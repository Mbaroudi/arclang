// The AI's change written in the file, for the engineer to keep or undo, as
// in an editor with an agent: the lines it added are highlighted, the places
// it removed lines are marked (the removed text on hover), and Keep / Undo sit
// above the first change and in the chat. Undo puts back the text as it was;
// Keep only clears the marks (the edit stays undoable with Ctrl/Cmd+Z).
import * as vscode from 'vscode';
import { changeMarks } from './core/diff';

interface Pending {
  document: vscode.TextDocument;
  before: string;
  after: string;
  label: string;
  firstLine: number;
}

const added = vscode.window.createTextEditorDecorationType({
  isWholeLine: true,
  backgroundColor: new vscode.ThemeColor('diffEditor.insertedLineBackground'),
  overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.addedForeground'),
  overviewRulerLane: vscode.OverviewRulerLane.Left,
  gutterIconSize: 'contain',
});
const removed = vscode.window.createTextEditorDecorationType({
  isWholeLine: true,
  borderColor: new vscode.ThemeColor('editorOverviewRuler.deletedForeground'),
  borderStyle: 'solid',
  borderWidth: '2px 0 0 0',
  overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.deletedForeground'),
  overviewRulerLane: vscode.OverviewRulerLane.Left,
});

export class InlineReview implements vscode.CodeLensProvider, vscode.Disposable {
  private readonly pending = new Map<string, Pending>();
  private readonly changed = new vscode.EventEmitter<void>();
  readonly onDidChangeCodeLenses = this.changed.event;
  /** Fired when a change is kept or undone (the chat updates its card). */
  private readonly settled = new vscode.EventEmitter<{ uri: string; kept: boolean; error?: string }>();
  readonly onDidSettle = this.settled.event;
  private readonly disposables: vscode.Disposable[];

  constructor() {
    this.disposables = [
      this.changed,
      this.settled,
      vscode.languages.registerCodeLensProvider({ pattern: '**/*.arc' }, this),
      vscode.commands.registerCommand('arclang.ai.keep', (uri?: string) => this.keep(uri ?? vscode.window.activeTextEditor?.document.uri.toString())),
      vscode.commands.registerCommand('arclang.ai.undoChange', (uri?: string) => this.undo(uri ?? vscode.window.activeTextEditor?.document.uri.toString())),
      vscode.window.onDidChangeVisibleTextEditors(() => this.paintAll()),
      vscode.workspace.onDidCloseTextDocument((d) => this.forget(d.uri.toString())),
    ];
  }

  /** The change was just written: mark it and offer Keep / Undo. */
  async start(document: vscode.TextDocument, before: string, label: string): Promise<void> {
    const after = document.getText();
    const marks = changeMarks(before, after);
    const firstLine = Math.min(...marks.added, ...marks.removed.map((r) => r.at), document.lineCount - 1);
    this.pending.set(document.uri.toString(), { document, before, after, label, firstLine: Math.max(0, firstLine) });
    const editor = await vscode.window.showTextDocument(document, { preserveFocus: true, preview: false });
    editor.revealRange(new vscode.Range(firstLine, 0, firstLine, 0), vscode.TextEditorRevealType.InCenterIfOutsideViewport);
    this.paintAll();
    this.changed.fire();
  }

  isPending(uri: string): boolean {
    return this.pending.has(uri);
  }

  keep(uri: string | undefined): void {
    if (!uri || !this.pending.has(uri)) return;
    this.forget(uri);
    this.settled.fire({ uri, kept: true });
  }

  async undo(uri: string | undefined): Promise<void> {
    const item = uri ? this.pending.get(uri) : undefined;
    if (!uri || !item) return;
    if (item.document.getText() !== item.after) {
      this.forget(uri);
      this.settled.fire({ uri, kept: true, error: 'the file changed since: use Ctrl/Cmd+Z to undo the change' });
      return;
    }
    const whole = new vscode.Range(item.document.positionAt(0), item.document.positionAt(item.after.length));
    const edit = new vscode.WorkspaceEdit();
    edit.replace(item.document.uri, whole, item.before);
    const ok = await vscode.workspace.applyEdit(edit);
    this.forget(uri);
    this.settled.fire({ uri, kept: !ok, error: ok ? undefined : 'VS Code did not take the edit' });
  }

  provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
    const item = this.pending.get(document.uri.toString());
    if (!item) return [];
    const at = new vscode.Range(item.firstLine, 0, item.firstLine, 0);
    const uri = document.uri.toString();
    return [
      new vscode.CodeLens(at, { title: '$(check) Keep', command: 'arclang.ai.keep', arguments: [uri], tooltip: 'Keep the AI\'s change' }),
      new vscode.CodeLens(at, { title: '$(discard) Undo', command: 'arclang.ai.undoChange', arguments: [uri], tooltip: 'Put back the text as it was' }),
      new vscode.CodeLens(at, { title: `AI: ${item.label}`, command: '' }),
    ];
  }

  private forget(uri: string): void {
    if (!this.pending.delete(uri)) return;
    for (const editor of vscode.window.visibleTextEditors) if (editor.document.uri.toString() === uri) {
      editor.setDecorations(added, []);
      editor.setDecorations(removed, []);
    }
    this.changed.fire();
  }

  private paintAll(): void {
    for (const editor of vscode.window.visibleTextEditors) {
      const item = this.pending.get(editor.document.uri.toString());
      if (!item) continue;
      const marks = changeMarks(item.before, item.after);
      const line = (n: number) => editor.document.lineAt(Math.min(n, editor.document.lineCount - 1)).range;
      editor.setDecorations(added, marks.added.map((n) => ({ range: line(n), hoverMessage: 'Added by the AI' })));
      editor.setDecorations(removed, marks.removed.map((r) => {
        const md = new vscode.MarkdownString(`Removed by the AI:\n\n\`\`\`arc\n${r.lines.join('\n')}\n\`\`\``);
        return { range: line(r.at), hoverMessage: md };
      }));
    }
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
  }
}
