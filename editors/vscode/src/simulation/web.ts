// In the browser build there is no native compiler and no Docker: the
// simulation commands exist, so that a button never fails, and say so.
import * as vscode from 'vscode';

export function registerSimulationWeb(context: vscode.ExtensionContext): void {
  const message = 'ArcLang: simulation needs the desktop extension. It runs OpenModelica on your computer, which a browser cannot do.';
  for (const command of ['menu', 'run', 'montecarlo', 'check', 'setup', 'export', 'drawing']) {
    context.subscriptions.push(vscode.commands.registerCommand(`arclang.simulation.${command}`, () => void vscode.window.showInformationMessage(message)));
  }
  context.subscriptions.push(vscode.commands.registerCommand('arclang.simulation.docs', () => vscode.env.openExternal(vscode.Uri.parse('https://github.com/Mbaroudi/arclang/blob/main/docs/modelica.md'))));
}
