// Where the extension runs, for what the AI is told: the desktop, or
// arclang.fr signed in (the page then offers the compiler online, the
// command `arclangWeb.compiler.run`) or not.
import * as vscode from 'vscode';
import type { Workspace } from './core/capabilities';

export async function currentWorkspace(): Promise<Workspace> {
  if (vscode.env.uiKind !== vscode.UIKind.Web) return 'desktop';
  const commands = await vscode.commands.getCommands(true);
  return commands.includes('arclangWeb.compiler.run') ? 'online' : 'online-anonymous';
}
