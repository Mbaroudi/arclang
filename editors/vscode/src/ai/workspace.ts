// Where the extension runs, for what the AI is told: the desktop, or
// arclang.fr signed in or not (the page answers `arclangWeb.account.signedIn`;
// a page without it is taken as not signed in, the cautious answer).
import * as vscode from 'vscode';
import type { Workspace } from './core/capabilities';

export async function currentWorkspace(): Promise<Workspace> {
  if (vscode.env.uiKind !== vscode.UIKind.Web) return 'desktop';
  const commands = await vscode.commands.getCommands(true);
  // The page says who is signed in; its compiler command exists for anyone (it offers to sign in).
  if (commands.includes('arclangWeb.account.signedIn')) {
    const signedIn = await vscode.commands.executeCommand<boolean>('arclangWeb.account.signedIn').then((v) => v === true, () => false);
    return signedIn ? 'online' : 'online-anonymous';
  }
  return 'online-anonymous';
}
