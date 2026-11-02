// Export the report of a simulation run: the outcomes the run printed and,
// for each test case, its replay (the series kept of its last run), as a
// PDF or a PowerPoint deck (see ../pure/sim-report).
import * as path from 'node:path';
import { deflateRawSync } from 'node:zlib';
import * as vscode from 'vscode';
import { run } from '../cli';
import type { Outcome } from '../pure/simulation';
import { buildReport, buildReportPdf, buildReportPptx, type Replay } from '../pure/sim-report';

/** The replay of a test case's last run, or null when none was kept. */
async function replayOf(file: string, testCase: string, env: () => Record<string, string>): Promise<Replay | null> {
  const result = await run(['verify', file, '--replay', testCase], { cwd: path.dirname(file), env: env() });
  if (result.code !== 0) return null;
  try {
    const replay = JSON.parse(result.stdout) as Replay;
    return Array.isArray(replay.time) && Array.isArray(replay.elements) && Array.isArray(replay.expectations) ? replay : null;
  } catch {
    return null;
  }
}

export async function exportSimulationReport(file: string, outcomes: Outcome[], env: () => Record<string, string>): Promise<void> {
  const format = await vscode.window.showQuickPick(
    [
      { label: 'PDF', description: 'one page per test case, with its verdict, its values and their curves', ext: 'pdf' as const },
      { label: 'PowerPoint', description: 'the same, as slides of shapes you can edit', ext: 'pptx' as const },
    ],
    { placeHolder: `Export the simulation report of ${path.basename(file)} as…` },
  );
  if (!format) return;
  const base = path.basename(file).replace(/\.[^.]+$/, '');
  const target = await vscode.window.showSaveDialog({
    defaultUri: vscode.Uri.file(path.join(path.dirname(file), `${base}-simulation.${format.ext}`)),
    filters: format.ext === 'pdf' ? { PDF: ['pdf'] } : { 'PowerPoint presentation': ['pptx'] },
    saveLabel: 'Export',
  });
  if (!target) return;
  try {
    const bytes = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Writing the simulation report…' }, async () => {
      const replays = new Map<string, Replay>();
      // A sweep or a Monte Carlo case keeps the series of one point: its page shows the points, not a curve.
      for (const outcome of outcomes.filter((o) => !o.points.length)) {
        const replay = await replayOf(file, outcome.test_case, env);
        if (replay) replays.set(outcome.test_case, replay);
      }
      const report = buildReport(base, outcomes, replays, new Date());
      return format.ext === 'pdf' ? buildReportPdf(report) : buildReportPptx(report, (data) => new Uint8Array(deflateRawSync(data)));
    });
    await vscode.workspace.fs.writeFile(target, bytes);
    const open = await vscode.window.showInformationMessage(`ArcLang: the simulation report is written to ${path.basename(target.fsPath)}`, 'Open', 'Reveal');
    if (open === 'Open') await vscode.env.openExternal(target);
    else if (open === 'Reveal') await vscode.commands.executeCommand('revealFileInOS', target);
  } catch (error) {
    void vscode.window.showErrorMessage(`ArcLang: the simulation report could not be written: ${error instanceof Error ? error.message : String(error)}`);
  }
}
