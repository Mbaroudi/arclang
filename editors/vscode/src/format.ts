// Formatting through `arclang format`: whitespace only, comments and order kept.
import * as vscode from 'vscode';
import { failureText, run, withTempFile } from './cli';

export function registerFormatter(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider('arclang', {
      async provideDocumentFormattingEdits(document, _options, token) {
        const text = document.getText();
        const result = await withTempFile(text, 'model.arc', (file, dir) => run(['format', file], { cwd: dir, token }));
        if (result.code !== 0) {
          vscode.window.showWarningMessage(`ArcLang: not formatted. ${failureText(result).split('\n')[0]}`);
          return [];
        }
        if (result.stdout === text) return [];
        const whole = new vscode.Range(document.positionAt(0), document.positionAt(text.length));
        return [vscode.TextEdit.replace(whole, result.stdout)];
      },
    }),
  );
}
