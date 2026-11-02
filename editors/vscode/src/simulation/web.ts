// In the browser build there is no native compiler and no Docker. Where the
// page hosting VS Code offers OpenModelica online (arclang.fr, for signed-in
// visitors: the command `arclangWeb.simulation.run`), "Run simulations" sends
// the model's folder there and shows the verdicts, the Monte Carlo page and
// the report; the page counts the runs and the server runs them in a
// sandbox. Elsewhere the commands say that simulation needs the desktop
// extension, so that a button never fails silently.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { PDF_VIEW_TYPE } from '../pdf/viewer';
import { failureText, run } from '../cli';
import { deliver } from '../download';
import { leftText, pickFiles, walkable, ONLINE_LIMITS } from '../pure/sim-online';
import { buildReport, buildReportPdf, type Replay } from '../pure/sim-report';
import { DOCS_URL, Outcome, countOutcomes, outcomesLog, parseOutcomes, simulates, summaryText } from '../pure/simulation';
import { MonteCarloPanel } from './montecarlo';
import { keepOnlineRun, type OnlineStatistics } from './online-results';
import { replaySimulation } from './replay';

const HOST_COMMAND = 'arclangWeb.simulation.run';
const DESKTOP_ONLY = 'ArcLang: simulation needs the desktop extension. It runs OpenModelica on your computer, which a browser cannot do.';

type HostAnswer = { ok: true; answer: Record<string, unknown> } | { ok: false; error: string } | null;

let channel: vscode.OutputChannel | undefined;
const output = () => (channel ??= vscode.window.createOutputChannel('ArcLang Simulation'));

/** The last online run: what "Export the report" writes. */
let last: { uri: vscode.Uri; outcomes: Outcome[]; replays: Map<string, Replay> } | null = null;
let running = false;

async function online(): Promise<boolean> {
  return (await vscode.commands.getCommands(true)).includes(HOST_COMMAND);
}

/** The model the user is looking at: a text editor, or the workbench's tab. */
function activeModel(arg: unknown): vscode.Uri | null {
  if (arg instanceof vscode.Uri && arg.path.endsWith('.arc')) return arg;
  const doc = vscode.window.activeTextEditor?.document;
  if (doc && doc.languageId === 'arclang' && doc.uri.scheme !== 'untitled') return doc.uri;
  const input = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  return input?.uri instanceof vscode.Uri && input.uri.path.endsWith('.arc') ? input.uri : null;
}

/** Every file under the model's folder, with its size, as paths relative to it. */
async function listFolder(dir: vscode.Uri): Promise<{ path: string; size: number }[]> {
  const found: { path: string; size: number }[] = [];
  const walk = async (uri: vscode.Uri, rel: string, depth: number) => {
    if (depth > ONLINE_LIMITS.depth || found.length > 2000) return;
    for (const [name, type] of await vscode.workspace.fs.readDirectory(uri)) {
      const child = vscode.Uri.joinPath(uri, name);
      const childRel = rel ? `${rel}/${name}` : name;
      if (type & vscode.FileType.Directory) {
        if (walkable(name)) await walk(child, childRel, depth + 1);
      } else if (type & vscode.FileType.File) {
        found.push({ path: childRel, size: (await vscode.workspace.fs.stat(child)).size });
      }
    }
  };
  await walk(dir, '', 1);
  return found;
}

async function runOnline(arg: unknown, monteCarlo: boolean): Promise<void> {
  if (running) return void vscode.window.showInformationMessage('ArcLang: a simulation is already running.');
  const uri = activeModel(arg);
  const doc = uri ? await vscode.workspace.openTextDocument(uri) : null;
  if (!uri || !doc || !simulates(doc.getText())) {
    const choice = await vscode.window.showInformationMessage(
      'ArcLang: open a model with a test case that simulates (simulate: and expects:), then run the simulations.',
      'Open the documentation',
    );
    if (choice) await vscode.env.openExternal(vscode.Uri.parse(DOCS_URL));
    return;
  }
  if (doc.isDirty) await doc.save();
  const dir = vscode.Uri.joinPath(uri, '..');
  const model = path.posix.basename(uri.path);
  const picked = pickFiles(model, await listFolder(dir));
  if (picked.error) return void vscode.window.showErrorMessage(`ArcLang: ${picked.error}.`);
  const files: Record<string, string> = {};
  for (const rel of picked.take) files[rel] = new TextDecoder().decode(await vscode.workspace.fs.readFile(vscode.Uri.joinPath(dir, rel)));

  running = true;
  let reply: HostAnswer;
  try {
    reply = await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Notification, title: 'Simulating online with OpenModelica… (a few minutes for a large model)' },
      () => Promise.resolve(vscode.commands.executeCommand<HostAnswer>(HOST_COMMAND, { model, files })),
    );
  } finally {
    running = false;
  }
  if (!reply) return;
  if (!reply.ok) return void vscode.window.showErrorMessage(`ArcLang: ${reply.error}`);
  const answer = reply.answer;
  const outcomes = Array.isArray(answer.outcomes) ? parseOutcomes(JSON.stringify(answer.outcomes)) : null;
  const log = output();
  if (!outcomes) {
    log.appendLine(`--- ${model}: the model could not be simulated${answer.timedOut ? ' (it took longer than allowed online)' : ''}`);
    if (typeof answer.stderr === 'string') log.appendLine(answer.stderr);
    log.show(true);
    return void vscode.window.showErrorMessage(
      answer.timedOut
        ? 'ArcLang: the simulation took longer than an online run may (15 minutes). Shorten it, or run it on your computer with the desktop extension.'
        : 'ArcLang: the model could not be simulated. See the ArcLang Simulation output.',
    );
  }
  // What the server computed in the same job: the replays (by case, then point) and the statistics.
  const byCase = (answer.replays && typeof answer.replays === 'object' ? answer.replays : {}) as Record<string, Record<string, unknown>>;
  const statistics = (answer.statistics && typeof answer.statistics === 'object' ? answer.statistics : {}) as Record<string, Record<string, OnlineStatistics>>;
  const cache = (answer.cache && typeof answer.cache === 'object' ? answer.cache : {}) as Record<string, string>;
  keepOnlineRun(uri.fsPath, { cases: outcomes.map((o) => ({ id: o.test_case, name: o.name || o.test_case, replayable: !!byCase[o.test_case] })), replays: byCase, statistics, cache });
  // The report draws the curves of single runs (a sweep or a Monte Carlo case shows its points).
  const replays = new Map<string, Replay>();
  for (const o of outcomes.filter((o) => !o.points.length)) {
    const value = byCase[o.test_case]?.['0'] as Replay | undefined;
    if (value && Array.isArray(value.time) && Array.isArray(value.elements) && Array.isArray(value.expectations)) replays.set(o.test_case, value);
  }
  last = { uri, outcomes, replays };
  log.appendLine(`--- ${model}, simulated online (${new Date().toLocaleString()})${picked.left ? `; ${picked.left} file(s) of the folder not sent (limits)` : ''}`);
  log.appendLine(outcomesLog(outcomes));
  log.show(true);
  if (MonteCarloPanel.has(outcomes)) MonteCarloPanel.show(uri.fsPath, outcomes, () => ({}));
  else if (monteCarlo) void vscode.window.showInformationMessage('ArcLang: no test case of this model samples uncertain parameters (uncertain: in simulate: { ... }).');
  const counts = countOutcomes(outcomes);
  const show = counts.fail || counts.other ? vscode.window.showWarningMessage : vscode.window.showInformationMessage;
  const choice = await show(`ArcLang: ${summaryText(counts)}${leftText(answer.left)}`, 'Show the verdicts', 'Export the report (PDF)');
  if (choice === 'Show the verdicts') log.show(true);
  else if (choice === 'Export the report (PDF)') await exportReport();
}

/** The report of the last online run, as a PDF beside the model. */
async function exportReport(): Promise<void> {
  if (!last) return void vscode.window.showInformationMessage('ArcLang: run the simulations first; the report shows their verdicts and curves.');
  const base = path.posix.basename(last.uri.path).replace(/\.[^.]+$/, '');
  const target = vscode.Uri.joinPath(last.uri, '..', `${base}-simulation.pdf`);
  try {
    const report = buildReport(base, last.outcomes, last.replays, new Date());
    const pdf = buildReportPdf(report);
    await vscode.workspace.fs.writeFile(target, pdf);
    await deliver(target, pdf);
    const open = await vscode.window.showInformationMessage(`ArcLang: the simulation report is written to ${path.posix.basename(target.path)}, beside the model.`, 'Open');
    if (open === 'Open') await vscode.commands.executeCommand('vscode.openWith', target, PDF_VIEW_TYPE);
  } catch (error) {
    void vscode.window.showErrorMessage(`ArcLang: the simulation report could not be written: ${error instanceof Error ? error.message : String(error)}`);
  }
}

/** The replay of the last online run of the model looked at (or of the last run). */
async function replayLast(): Promise<void> {
  const uri = activeModel(undefined) ?? last?.uri ?? null;
  if (uri && last && uri.toString() !== last.uri.toString() && !activeModel(undefined)) return replaySimulation(last.uri);
  if (!last) return void vscode.window.showInformationMessage('ArcLang: run the simulations first; the replay moves through their last run.');
  return replaySimulation(uri);
}

/** The physical architecture as a Modelica package, beside the model (the compiler online writes it). */
async function exportModelica(arg?: unknown): Promise<void> {
  const uri = activeModel(arg);
  if (!uri) return void vscode.window.showInformationMessage('ArcLang: open a .arc model first.');
  const dir = path.posix.dirname(uri.fsPath);
  const target = `${dir}/modelica`;
  const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Exporting to Modelica…' }, () =>
    run(['export', uri.fsPath, '-f', 'modelica', '--split', '-o', target], { cwd: dir }),
  );
  if (result.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: the Modelica export failed. ${failureText(result).split('\n')[0]}`);
  const written = /Output: (.+)/.exec(result.stdout)?.[1]?.trim() ?? target;
  void vscode.window.showInformationMessage(`ArcLang: Modelica package written to ${vscode.workspace.asRelativePath(written)}.`);
}

async function menu(): Promise<void> {
  type Entry = vscode.QuickPickItem & { id: string };
  const entries: Entry[] = [
    { id: 'run', label: '$(play) Run simulations on this model', detail: 'Online, in OpenModelica: the verdicts of the test cases. For signed-in visitors, a number of runs a day.' },
    { id: 'montecarlo', label: '$(graph) Show Monte Carlo results', detail: 'Runs the model online and shows how likely the requirements hold.' },
    { id: 'replay', label: '$(history) Replay a simulation on the diagram', detail: 'Moves through the last online run of a test case: the values on each box, the first instant an expectation fails.' },
    { id: 'modelica', label: '$(export) Export Modelica', detail: 'Write the physical architecture as a Modelica package, beside the model.' },
    { id: 'export', label: '$(file-pdf) Export the report of the last run', detail: 'A PDF beside the model: one page per test case, with its verdict, values and curves.' },
    { id: 'docs', label: '$(book) Open the documentation', detail: 'How simulation works and how to write a test case.' },
  ];
  const picked = await vscode.window.showQuickPick(entries, { title: 'ArcLang: Simulation (online)' });
  if (picked?.id === 'run') return runOnline(undefined, false);
  if (picked?.id === 'montecarlo') return runOnline(undefined, true);
  if (picked?.id === 'export') return exportReport();
  if (picked?.id === 'modelica') return exportModelica();
  if (picked?.id === 'replay') return replayLast();
  if (picked?.id === 'docs') return void vscode.env.openExternal(vscode.Uri.parse(DOCS_URL));
}

export function registerSimulationWeb(context: vscode.ExtensionContext): void {
  const desktopOnly = () => void vscode.window.showInformationMessage(DESKTOP_ONLY);
  const register = (command: string, action: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(`arclang.simulation.${command}`, action));
  register('menu', async () => ((await online()) ? menu() : desktopOnly()));
  register('run', async (arg?: unknown) => ((await online()) ? runOnline(arg, false) : desktopOnly()));
  register('montecarlo', async (arg?: unknown) => ((await online()) ? runOnline(arg, true) : desktopOnly()));
  register('export', async (arg?: unknown) => ((await online()) ? exportModelica(arg) : desktopOnly()));
  register('exportReport', async () => ((await online()) ? exportReport() : desktopOnly()));
  register('replay', async () => ((await online()) ? replayLast() : desktopOnly()));
  for (const command of ['check', 'setup', 'drawing']) register(command, desktopOnly);
  register('docs', () => vscode.env.openExternal(vscode.Uri.parse(DOCS_URL)));

  // The ▶ of the editor's title bar, on a model that simulates, where simulation runs online.
  // Asked again until the page has registered its command (once the account is known).
  let available = false;
  const refresh = async () => {
    available ||= await online();
    const uri = available ? activeModel(undefined) : null;
    const doc = uri ? vscode.workspace.textDocuments.find((d) => d.uri.toString() === uri.toString()) : undefined;
    void vscode.commands.executeCommand('setContext', 'arclang.simulatesHere', !!doc && simulates(doc.getText()));
  };
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor(() => void refresh()),
    vscode.window.tabGroups.onDidChangeTabs(() => void refresh()),
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document.languageId === 'arclang') void refresh();
    }),
  );
  for (const delay of [2000, 6000, 15000]) setTimeout(() => void refresh(), delay);
}
