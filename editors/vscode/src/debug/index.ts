// The model debugger: VS Code runs `arclang debug --dap` (the Debug Adapter
// Protocol) for a launch of type `arclang`. *ArcLang: Run…* offers the state
// machines, scenarios and functional chains of the model (F5 runs the first);
// a breakpoint on a line stops when a step reaches it. *ArcLang: Fire an
// event* injects an event into a state machine; the workbench diagram lights
// the active states, the message just sent or the chain steps with a token.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { binary, run } from '../cli';
import { WorkbenchPanel } from '../workbench/panel';
import { DebugStateMessage, firePicks, nothingToRun, Runnables, runPicks } from '../pure/debug';
import { simulates } from '../pure/simulation';

interface MachineState {
  kind: string;
  name: string;
  nodes: string[];
  edges: string[];
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
  const message: DebugStateMessage = { type: 'debug-state', kind: state?.kind ?? '', name: state?.name ?? '', nodes: state?.nodes ?? [], edges: state?.edges ?? [] };
  for (const workbench of WorkbenchPanel.all(vscode.Uri.file(program))) workbench.postPanes(message);
}

/** The model a launch runs: the configuration's, else the .arc of the active editor. */
function activeModel(): string | null {
  const editor = vscode.window.activeTextEditor;
  if (editor?.document.languageId === 'arclang') return editor.document.uri.fsPath;
  const tab = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  return tab?.uri?.fsPath.endsWith('.arc') ? tab.uri.fsPath : null;
}

/**
 * What a model runs, or null once the user has been told why nothing runs:
 * the model does not compile, or it has nothing to run (with the simulations
 * offered when the model is verified by simulation).
 */
async function runnablesOf(program: string): Promise<Runnables | null> {
  const listed = await run(['debug', program, '--list'], { cwd: path.dirname(program) });
  let runnables: Runnables;
  try {
    runnables = JSON.parse(listed.stdout) as Runnables;
  } catch {
    void vscode.window.showErrorMessage(`ArcLang: the model does not compile, nothing to run. ${listed.stderr.trim().split('\n').pop() ?? ''}`);
    return null;
  }
  if (runPicks(runnables).length > 0) return runnables;
  let text = '';
  try {
    text = Buffer.from(await vscode.workspace.fs.readFile(vscode.Uri.file(program))).toString('utf8');
  } catch {
    /* unreadable: nothing more to offer */
  }
  const { text: message, action } = nothingToRun(simulates(text));
  void (action ? vscode.window.showInformationMessage(message, action) : vscode.window.showInformationMessage(message)).then((choice) => {
    if (choice === 'Run simulations') void vscode.commands.executeCommand('arclang.simulation.run', vscode.Uri.file(program));
  });
  return null;
}

export function registerDebug(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory('arclang', {
      createDebugAdapterDescriptor: () => new vscode.DebugAdapterExecutable(binary(), ['debug', '--dap']),
    }),
    vscode.debug.registerDebugConfigurationProvider('arclang', {
      async resolveDebugConfiguration(_folder, config) {
        const program = typeof config.program === 'string' && config.program ? config.program : activeModel();
        if (!program) {
          void vscode.window.showInformationMessage('ArcLang: open a .arc model with a state machine to run it.');
          return undefined;
        }
        // F5 with nothing named runs the first: say so when there is none.
        if (!config.machine && !config.scenario && !config.chain && !(await runnablesOf(program))) return undefined;
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
      if (!program) return vscode.window.showInformationMessage('ArcLang: open a .arc model to run one of its state machines, scenarios or functional chains.');
      // What the model runs: one is run at once, several are offered.
      const runnables = await runnablesOf(program);
      if (!runnables) return;
      const picks = runPicks(runnables);
      const choice = picks.length === 1 ? picks[0] : await vscode.window.showQuickPick(picks, { placeHolder: 'What to run, step by step' });
      if (!choice) return;
      await vscode.debug.startDebugging(vscode.workspace.getWorkspaceFolder(vscode.Uri.file(program)), { type: 'arclang', request: 'launch', name: `Run ${choice.name}`, program, [choice.key]: choice.name });
    }),
    vscode.commands.registerCommand('arclang.debug.fire', async () => {
      const session = vscode.debug.activeDebugSession;
      if (!session || session.type !== 'arclang') return vscode.window.showInformationMessage('ArcLang: nothing is running (ArcLang: Run… on a .arc model).');
      if (states.get(session.id)?.kind && states.get(session.id)?.kind !== 'machine') return vscode.window.showInformationMessage('ArcLang: a scenario or a chain takes no event: Step Over (F10) runs its next step.');
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
