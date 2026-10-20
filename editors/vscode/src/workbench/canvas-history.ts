// The workbench's ⌘Z and ⌘⇧Z: one history for the edits of the model and
// the changes of the arrangement (see ../pure/history.ts). An edit of the
// model is taken back by writing the text it recorded into the workbench's
// own document (a workspace edit on that document, never the global undo
// command, which acts on whichever editor VS Code thinks is active), and only
// when the document still holds the text the edit left; a change of the
// arrangement by writing the layout file as it was. The text editor's ⌘Z
// stays the document's: a change of the text made outside the workbench
// drops this history, its steps no longer apply.
import * as vscode from 'vscode';
import { History, HistoryEntry, LayoutState } from '../pure/history';
import { ToWebview } from '../pure/workbench';
import { replaceWhole } from './source';

export interface HistoryHost {
  document: vscode.TextDocument;
  /** Write the arrangement (null: no layout file) and show it, unless a change of the text that follows will. */
  restoreLayout(state: LayoutState, show: boolean): Promise<void>;
  post(message: ToWebview): void;
}

export class CanvasHistory implements vscode.Disposable {
  private readonly steps = new History();
  /** Changes of the text the workbench itself is making. */
  private own = 0;
  private readonly listener: vscode.Disposable;

  constructor(private readonly host: HistoryHost) {
    this.listener = vscode.workspace.onDidChangeTextDocument((event) => {
      if (event.document.uri.toString() !== host.document.uri.toString() || event.contentChanges.length === 0) return;
      if (this.own > 0) return;
      // Typed, undone in the text editor, formatted: what this history recorded is gone.
      if (this.steps.canUndo || this.steps.canRedo) {
        this.steps.clear();
        this.announce();
      }
    });
  }

  dispose(): void {
    this.listener.dispose();
  }

  /** Run a change of the text the workbench makes itself (an edit, an undo). */
  async ownChange<T>(change: () => Promise<T>): Promise<T> {
    this.own += 1;
    try {
      return await change();
    } finally {
      // The change events of an applied edit may come a moment after it resolves.
      setTimeout(() => (this.own -= 1), 50);
    }
  }

  recordModel(label: string, before: string, after: string): void {
    this.steps.recordModel(label, before, after, Date.now());
    this.announce();
  }

  recordLayout(before: LayoutState, after: LayoutState, join: boolean): void {
    this.steps.recordLayout('move', before, after, Date.now(), join);
    this.announce();
  }

  private announce(done?: string): void {
    const { undo, redo } = this.steps.labels();
    this.host.post({ type: 'history', undo, redo, done });
  }

  async undo(): Promise<void> {
    const entry = this.steps.undo();
    if (!entry) {
      // Nothing of the canvas's own. The text's earlier edits are the text editor's to undo.
      void vscode.window.showInformationMessage('ArcLang: nothing to undo in the workbench. ⌘Z in the text editor undoes the text.');
      return this.announce();
    }
    const ok = await this.apply(entry, 'undo');
    if (!ok) this.steps.clear();
    this.announce(ok ? `undo ${entry.label}` : undefined);
  }

  async redo(): Promise<void> {
    const entry = this.steps.redo();
    if (!entry) return this.announce();
    const ok = await this.apply(entry, 'redo');
    if (!ok) this.steps.clear();
    this.announce(ok ? `redo ${entry.label}` : undefined);
  }

  private async apply(entry: HistoryEntry, way: 'undo' | 'redo'): Promise<boolean> {
    const document = this.host.document;
    if (entry.kind === 'layout') {
      await this.host.restoreLayout(way === 'undo' ? entry.before : entry.after, true);
      return true;
    }
    const [from, to] = way === 'undo' ? [entry.after, entry.before] : [entry.before, entry.after];
    if (document.isClosed || document.getText() !== from) {
      void vscode.window.showWarningMessage(`ArcLang: the text changed since "${entry.label}": the workbench's history was dropped. ⌘Z in the text editor undoes the text.`);
      return false;
    }
    // The arrangement first: the snapshot of the new text is drawn with it.
    if (entry.layout) await this.host.restoreLayout(way === 'undo' ? entry.layout.before : entry.layout.after, false);
    // On this document only: the text editor of another model is never touched.
    const applied = await this.ownChange(() => replaceWhole(document, to));
    if (!applied) void vscode.window.showWarningMessage(`ArcLang: VS Code did not apply the ${way} of "${entry.label}".`);
    return applied && document.getText() === to;
  }
}
