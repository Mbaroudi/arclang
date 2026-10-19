// One status bar item for the active model: the gate verdict and the number
// of findings, from the compiler's snapshot. A click opens the Problems view.
import * as vscode from 'vscode';
import { Snapshot } from '../pure/workbench';
import { lastSnapshot, onDidChangeSnapshot, snapshotOf } from './snapshot';
import { isModelDocument } from './source';

export function registerStatusBar(context: vscode.ExtensionContext): void {
  const item = vscode.window.createStatusBarItem('arclang.gate', vscode.StatusBarAlignment.Left, 50);
  item.name = 'ArcLang gate';
  item.command = 'workbench.actions.view.problems';
  context.subscriptions.push(item);

  const render = (document: vscode.TextDocument | undefined) => {
    if (!document || !isModelDocument(document)) {
      item.hide();
      return;
    }
    const snapshot: Snapshot | null = lastSnapshot(document);
    if (!snapshot) {
      item.text = '$(sync~spin) ArcLang';
      item.tooltip = 'Compiling the model';
      item.backgroundColor = undefined;
      item.show();
      void snapshotOf(document);
      return;
    }
    if (!snapshot.success) {
      item.text = `$(error) ArcLang: ${snapshot.errors.length} error${snapshot.errors.length > 1 ? 's' : ''}`;
      item.tooltip = snapshot.errors[0] ?? 'the model does not compile';
      item.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
      item.show();
      return;
    }
    const findings = snapshot.findings.length;
    const gate = snapshot.gate;
    const verdict = gate ? (gate.passed ? '$(pass) Gate PASS' : '$(warning) Gate FAIL') : '$(check) ArcLang';
    item.text = `${verdict} · ${findings}`;
    item.tooltip = new vscode.MarkdownString(
      [
        gate ? `Production gate: **${gate.passed ? 'PASS' : 'FAIL'}**, ${gate.requirements_verified}/${gate.requirements_total} requirements verified` : '',
        `${findings} finding${findings === 1 ? '' : 's'}`,
        'Click to open the Problems view.',
      ]
        .filter(Boolean)
        .join('\n\n'),
    );
    item.backgroundColor = gate && !gate.passed ? new vscode.ThemeColor('statusBarItem.warningBackground') : undefined;
    item.show();
  };

  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => render(editor?.document)),
    onDidChangeSnapshot((uri) => {
      const active = vscode.window.activeTextEditor?.document;
      if (active && active.uri.toString() === uri.toString()) render(active);
    }),
  );
  render(vscode.window.activeTextEditor?.document);
}
