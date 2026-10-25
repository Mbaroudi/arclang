// A test probe: with ARCLANG_NAV_PROBE=<file> in the environment, the command
// "ArcLang: Dump the editor layout (test)" writes the editors, the groups,
// the selections and the highlight to that file, for the end-to-end tests of
// the real VS Code (which cannot ask the extension host anything else).
import * as fs from 'node:fs';
import * as vscode from 'vscode';
import { highlightState } from './go-to-source';

/** What a tab holds: the text, the workbench (a custom editor), a diagram's own tab (a webview). */
function tabKind(input: unknown): string {
  if (input instanceof vscode.TabInputText) return 'text';
  if (input instanceof vscode.TabInputCustom) return input.viewType === 'arclang.workbench' ? 'workbench' : 'custom';
  if (input instanceof vscode.TabInputWebview) return input.viewType.endsWith('arclang.workbench.diagram') ? 'diagram' : 'webview';
  return 'other';
}

export function registerNavProbe(context: vscode.ExtensionContext): void {
  const file = process.env.ARCLANG_NAV_PROBE;
  if (!file) return;
  void vscode.commands.executeCommand('setContext', 'arclang.testProbe', true);
  context.subscriptions.push(
    vscode.commands.registerCommand('arclang.debug.navState', () => {
      const state = {
        groups: vscode.window.tabGroups.all.map((g) => ({ column: g.viewColumn, active: g.isActive, tabs: g.tabs.map((t) => ({ label: t.label, active: t.isActive, kind: tabKind(t.input) })) })),
        visibleEditors: vscode.window.visibleTextEditors.map((e) => ({
          file: e.document.uri.fsPath,
          column: e.viewColumn ?? null,
          selection: [e.selection.start.line, e.selection.start.character, e.selection.end.line, e.selection.end.character],
          visibleTop: e.visibleRanges[0]?.start.line ?? null,
        })),
        activeEditor: vscode.window.activeTextEditor?.document.uri.fsPath ?? null,
        highlight: highlightState(),
        text: vscode.workspace.textDocuments.filter((d) => d.languageId === 'arclang').map((d) => ({ file: d.uri.fsPath, dirty: d.isDirty, lines: d.lineCount })),
      };
      fs.writeFileSync(file, JSON.stringify(state));
    }),
  );
}
