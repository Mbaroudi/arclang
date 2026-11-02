// Full screen for any ArcLang view: the model's text, the workbench, a PDF, a
// table, the impact analysis... One button in the view's title bar (or
// Ctrl/Cmd+Alt+Enter) gives the view the whole window: its editor group is
// maximized and Zen mode hides the side bars, the panel and the status bar.
// The same button, Esc or the shortcut gives everything back as it was.
import * as vscode from 'vscode';

export const FULL_SCREEN_CONTEXT = 'arclang.fullScreen';

let entered: { maximized: boolean } | null = null;
const changed = new vscode.EventEmitter<boolean>();
/** Fires when full screen is entered or left, from any view. */
export const onDidChangeFullScreen = changed.event;

export const isFullScreen = (): boolean => entered !== null;

async function run(command: string): Promise<void> {
  await Promise.resolve(vscode.commands.executeCommand(command)).catch(() => undefined);
}

export async function setFullScreen(on: boolean): Promise<void> {
  if (on === isFullScreen()) return;
  if (on) {
    // A single group needs no maximizing: toggling it would do nothing to undo later.
    const maximized = vscode.window.tabGroups.all.length > 1;
    if (maximized) await run('workbench.action.toggleMaximizeEditorGroup');
    await run('workbench.action.toggleZenMode');
    // Zen mode centers the editor by default: a diagram or a page wants the full width.
    // Leaving Zen mode puts the centering back as it was.
    if (vscode.workspace.getConfiguration('zenMode').get<boolean>('centerLayout', true)) await run('workbench.action.toggleCenteredLayout');
    entered = { maximized };
  } else {
    const was = entered;
    entered = null;
    // Not a toggle: Zen mode may have been left another way (Esc Esc).
    await run('workbench.action.exitZenMode');
    if (was?.maximized) await run('workbench.action.toggleMaximizeEditorGroup');
  }
  await vscode.commands.executeCommand('setContext', FULL_SCREEN_CONTEXT, on);
  changed.fire(on);
}

export function registerFullScreen(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    changed,
    vscode.commands.registerCommand('arclang.view.fullScreen', () => setFullScreen(true)),
    vscode.commands.registerCommand('arclang.view.exitFullScreen', () => setFullScreen(false)),
    vscode.commands.registerCommand('arclang.view.toggleFullScreen', () => setFullScreen(!isFullScreen())),
  );
}
