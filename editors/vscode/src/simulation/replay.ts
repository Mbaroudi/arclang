// *ArcLang: Replay a simulation on the diagram*: the results `arclang
// verify` kept of the last run of a test case are read back (`arclang verify
// --replay`) and sent to the workbench, which moves through time, shows each
// part's values on its box and jumps to the first instant an expectation
// fails.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run } from '../cli';
import { WorkbenchPanel } from '../workbench/panel';
import { parseReplay, Replay, ReplayMessage, timeLabel } from '../pure/replay';

interface SimulatedCase {
  id: string;
  name: string;
  /** Whether results of its last run are kept. */
  replayable: boolean;
}

/** The test cases of the model that simulate (`arclang verify --list`). */
async function simulatedCases(file: string): Promise<SimulatedCase[] | string> {
  const result = await run(['verify', file, '--list'], { cwd: path.dirname(file) });
  try {
    const cases = JSON.parse(result.stdout) as SimulatedCase[];
    return Array.isArray(cases) ? cases : failureText(result);
  } catch {
    return `the model does not compile: ${result.stderr.trim().split('\n').pop() ?? ''}`;
  }
}

async function readReplay(file: string, testCase: string, point: number): Promise<Replay | string> {
  const result = await run(['verify', file, '--replay', testCase, '--point', String(point)], { cwd: path.dirname(file) });
  return parseReplay(result.stdout) ?? (result.stderr.trim().split('\n').pop() || failureText(result));
}

/** The workbenches of a model, however its path is spelled (`/var` or `/private/var`). */
function workbenchesOf(uri: vscode.Uri): WorkbenchPanel[] {
  const real = (p: string) => {
    try {
      return fs.realpathSync(p);
    } catch {
      return p;
    }
  };
  const target = real(uri.fsPath);
  const found = WorkbenchPanel.all(uri);
  if (found.length) return found;
  const active = WorkbenchPanel.activePanel();
  return active && real(active.documentOf.uri.fsPath) === target ? [active] : [];
}

/** Send the replay to every workbench of the model; a workbench just opened gets it once its page is up. */
function send(uri: vscode.Uri, message: ReplayMessage): void {
  for (const delay of [0, 1500, 4000, 8000, 15000]) {
    setTimeout(() => {
      const found = workbenchesOf(uri);
      for (const workbench of found) workbench.postPanes(message);
    }, delay);
  }
}

export async function replaySimulation(uri: vscode.Uri | null): Promise<void> {
  if (!uri) {
    void vscode.window.showInformationMessage('ArcLang: open a model (.arc) to replay one of its simulations.');
    return;
  }
  const file = uri.fsPath;
  const cases = await simulatedCases(file);
  if (typeof cases === 'string') {
    void vscode.window.showErrorMessage(`ArcLang: ${cases}`);
    return;
  }
  if (cases.length === 0) {
    void vscode.window.showInformationMessage('ArcLang: no test case of this model simulates (simulate: { ... }).');
    return;
  }
  const picked = cases.length === 1 ? cases[0] : (await vscode.window.showQuickPick(cases.map((c) => ({ label: c.name, description: c.id, detail: c.replayable ? undefined : 'Not run yet: run the simulations first', value: c })), { title: 'ArcLang: Replay a simulation', placeHolder: 'The test case whose last run to replay on the diagram' }))?.value;
  if (!picked) return;
  let replay = await readReplay(file, picked.id, 0);
  if (typeof replay === 'string') {
    const choice = await vscode.window.showWarningMessage(`ArcLang: ${replay}`, 'Run simulations');
    if (choice) void vscode.commands.executeCommand('arclang.simulation.run', uri);
    return;
  }
  if (replay.points.length > 1) {
    const point = await vscode.window.showQuickPick(replay.points.map((label, index) => ({ label: label || `Point ${index + 1}`, index })), { title: `ArcLang: Replay ${picked.name}`, placeHolder: 'The point of the sweep to replay' });
    if (!point) return;
    if (point.index !== 0) {
      const again = await readReplay(file, picked.id, point.index);
      if (typeof again === 'string') {
        void vscode.window.showWarningMessage(`ArcLang: ${again}`);
        return;
      }
      replay = again;
    }
  }
  await vscode.commands.executeCommand('arclang.openWorkbench', uri);
  const label = replay.points[replay.point] ? `${picked.name} (${replay.points[replay.point]})` : picked.name;
  send(uri, { type: 'replay-load', replay, title: label });
  const failing = replay.expectations.filter((e) => !e.holds).length;
  const end = replay.time[replay.time.length - 1] ?? 0;
  void vscode.window.setStatusBarMessage(`Replaying ${label}: ${timeLabel(end)}, ${failing ? `${failing} expectation(s) fail` : 'every expectation holds'}${replay.stale ? ' (the model changed since this run)' : ''}`, 6000);
}
