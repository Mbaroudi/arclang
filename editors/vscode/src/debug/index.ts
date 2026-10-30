// The model debugger: VS Code runs `arclang debug --dap` (the Debug Adapter
// Protocol) for a launch of type `arclang`. F5 on a .arc file runs its first
// state machine; breakpoints go on a state (stopped when it is entered) or a
// transition (when it fires). *ArcLang: Fire an event* injects an event; the
// workbench diagrams light the active states and the transition just fired.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { binary } from '../cli';
import { WorkbenchPanel } from '../workbench/panel';
import { DebugStateMessage, firePicks } from '../pure/debug';

interface MachineState {
  machine: string;
  active: string[];
  leaves: string[];
  fired: string[];
  events: string[];
}

const states = new Map<string, MachineState>();

function programOf(session: vscode.DebugSession): string | null {
  const program = session.configuration.program;
  return typeof program === 'string' ? program : null;
}

function paint(session: vscode.DebugSession, state: MachineState | null): void {
  const program = programOf(session);
  if (!program) return;
  const message: DebugStateMessage = { type: 'debug-state', machine: state?.machine ?? '', active: state?.active ?? [], fired: state?.fired ?? [] };
  for (const workbench of WorkbenchPanel.all(vscode.Uri.file(program))) workbench.postPanes(message);
}

/** The model a launch runs: the configuration's, else the .arc of the active editor. */
function activeModel(): string | null {
  const editor = vscode.window.activeTextEditor;
  if (editor?.document.languageId === 'arclang') return editor.document.uri.fsPath;
  const tab = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  return tab?.uri?.fsPath.endsWith('.arc') ? tab.uri.fsPath : null;
}

export function registerDebug(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory('arclang', {
      createDebugAdapterDescriptor: () => new vscode.DebugAdapterExecutable(binary(), ['debug', '--dap']),
    }),
    vscode.debug.registerDebugConfigurationProvider('arclang', {
      resolveDebugConfiguration(_folder, config) {
        const program = typeof config.program === 'string' && config.program ? config.program : activeModel();
        if (!program) {
          void vscode.window.showInformationMessage('ArcLang: open a .arc model with a state machine to run it.');
          return undefined;
        }
        return { ...config, type: 'arclang', request: 'launch', name: config.name || `Run ${path.basename(program)}`, program };
      },
    }),
    vscode.debug.onDidReceiveDebugSessionCustomEvent((event) => {
      if (event.session.type !== 'arclang' || event.event !== 'arclang.state') return;
      const state = event.body as MachineState;
      states.set(event.session.id, state);
      paint(event.session, state);
    }),
    vscode.debug.onDidTerminateDebugSession((session) => {
      if (session.type !== 'arclang') return;
      states.delete(session.id);
      paint(session, null);
    }),
    vscode.commands.registerCommand('arclang.debug.run', async (uri?: unknown) => {
      const program = uri instanceof vscode.Uri ? uri.fsPath : activeModel();
      if (!program) return vscode.window.showInformationMessage('ArcLang: open a .arc model with a state machine to run it.');
      await vscode.debug.startDebugging(vscode.workspace.getWorkspaceFolder(vscode.Uri.file(program)), { type: 'arclang', request: 'launch', name: `Run ${path.basename(program)}`, program });
    }),
    vscode.commands.registerCommand('arclang.debug.fire', async () => {
      const session = vscode.debug.activeDebugSession;
      if (!session || session.type !== 'arclang') return vscode.window.showInformationMessage('ArcLang: no state machine is running (F5 on a .arc model).');
      const picks = firePicks(states.get(session.id)?.events ?? []);
      const choice = await vscode.window.showQuickPick(picks.map((p) => ({ label: p.label, description: p.description, event: p.event })), { placeHolder: 'The event to fire' });
      if (!choice) return;
      let event: string | null = choice.event;
      if (choice.event === '?') {
        const typed = await vscode.window.showInputBox({ prompt: 'The event, as a trigger of the model names it' });
        if (!typed) return;
        event = typed.trim();
      }
      await session.customRequest('fire', event === null ? {} : { event });
    }),
  );
}
