// Baselines from the editor: the gate result first, then the compiler
// writes baselines/<name>/ and tags the commit; comparing with a baseline is
// a compare mode against its tag.
import * as vscode from 'vscode';
import { run, failureText, output } from '../cli';
import { BASELINE_STANDARDS, BaselineListed, parseBaselines, validBaselineName } from '../pure/semgit';
import { documentDir } from '../workbench/source';
import { repoTop } from './git';

/** What `arclang gate` says in its last line: PASS, or FAIL with the blockers. */
export function gateVerdict(stdout: string): { passed: boolean; blockers: number } {
  const passed = /PRODUCTION GATE:\s*PASS/.test(stdout);
  const blockers = Number(/\((\d+) blockers?\)/.exec(stdout)?.[1] ?? (passed ? 0 : 1));
  return { passed, blockers };
}

export async function listBaselines(dir: string): Promise<BaselineListed[]> {
  const result = await run(['baseline', 'list', '--json'], { cwd: dir });
  return result.code === 0 ? parseBaselines(result.stdout) : [];
}

export async function createBaseline(document: vscode.TextDocument): Promise<string | null> {
  if (document.uri.scheme !== 'file') return null;
  const dir = documentDir(document)!;
  if (!(await repoTop(dir))) {
    void vscode.window.showInformationMessage('ArcLang: a baseline needs the model to be in a git repository.');
    return null;
  }
  if (document.isDirty) {
    const choice = await vscode.window.showInformationMessage('ArcLang: a baseline is made from committed files. Save the model, then commit it first.', { modal: true }, 'Save');
    if (choice === 'Save') await document.save();
    return null;
  }
  const name = await vscode.window.showInputBox({ title: 'Create a baseline', prompt: 'Baseline name (it becomes the tag baseline/<name>)', placeHolder: 'v1.0, PDR, CDR-2026-10', validateInput: (v) => validBaselineName(v) });
  if (!name) return null;
  const standardPick = await vscode.window.showQuickPick(
    BASELINE_STANDARDS.map((s) => ({ label: s, description: s === vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262') ? 'the gate standard setting' : '' })),
    { title: `Baseline ${name}: standard`, placeHolder: 'The standard the gate is run against' },
  );
  if (!standardPick) return null;
  const standard = standardPick.label;
  // The gate first: what the auditor will read.
  const gate = await vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title: 'ArcLang: running the gate' }, () => run(['gate', document.uri.fsPath, '--standard', standard], { cwd: dir }));
  const verdict = gateVerdict(gate.stdout);
  const summary = verdict.passed ? `The production gate (${standard}) passes.` : `The production gate (${standard}) FAILS with ${verdict.blockers} blocker${verdict.blockers === 1 ? '' : 's'}.`;
  const go = await vscode.window.showWarningMessage(`${summary} Create the baseline "${name}" from the committed model?`, { modal: true, detail: gate.stdout.split('\n').filter((l) => /^\s*(✗|⚠|\[|Requirements|PRODUCTION)/.test(l) || /blocker|PRODUCTION/.test(l)).slice(0, 12).join('\n') }, 'Create baseline');
  if (go !== 'Create baseline') return null;
  const result = await run(['baseline', 'create', name, '--standard', standard, '--model', document.uri.fsPath], { cwd: dir });
  output().appendLine(`$ arclang baseline create ${name}\n${result.stdout}${result.stderr}`);
  if (result.code !== 0) {
    void vscode.window.showErrorMessage(`ArcLang: the baseline was not created. ${failureText(result).split('\n')[0]}`);
    return null;
  }
  const next = await vscode.window.showInformationMessage(`ArcLang: baseline "${name}" created and tagged baseline/${name} (not pushed).`, 'Verify', 'Compare with it');
  if (next === 'Verify') await verifyBaseline(dir, name);
  return next === 'Compare with it' ? name : name;
}

export async function verifyBaseline(dir: string, name: string): Promise<boolean> {
  const result = await run(['baseline', 'verify', name], { cwd: dir });
  output().appendLine(`$ arclang baseline verify ${name}\n${result.stdout}${result.stderr}`);
  const ok = result.code === 0;
  if (ok) void vscode.window.showInformationMessage(`ArcLang: baseline "${name}" is verified: files, tag and identities match the manifest.`);
  else void vscode.window.showErrorMessage(`ArcLang: baseline "${name}" does not verify. ${result.stdout.split('\n').filter((l) => l.includes('FAIL')).slice(0, 2).join(' ')}`);
  return ok;
}
