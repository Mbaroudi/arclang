// The commands of the palette and menus: each one is one CLI subcommand.
import { execFile } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, output, refusedHere, report, run } from './cli';
import { shownDiagramFile } from './diagrams';
import { EXPORT_FORMATS, parseCheckReport, renderTransitionPlan, summarizeCheck, TransitionPlan } from './pure/arc';

/** The .arc file to act on: the active editor's, else the one the diagrams show. */
export async function activeArcFile(): Promise<string | null> {
  const editor = vscode.window.activeTextEditor;
  if (editor && editor.document.languageId === 'arclang' && editor.document.uri.scheme === 'file') {
    if (editor.document.isDirty) {
      const choice = await vscode.window.showInformationMessage('ArcLang: the model has unsaved changes. Save it first?', 'Save and continue', 'Use the saved version');
      if (!choice) return null;
      if (choice === 'Save and continue') await editor.document.save();
    }
    return editor.document.uri.fsPath;
  }
  const shown = shownDiagramFile();
  if (shown) return shown;
  vscode.window.showInformationMessage('ArcLang: open a .arc file first.');
  return null;
}

/** A file given by a menu (explorer, editor title) or the active one. */
export async function fileFrom(uri?: vscode.Uri): Promise<string | null> {
  if (uri && uri.scheme === 'file') return uri.fsPath;
  return activeArcFile();
}

export async function check(uri?: vscode.Uri): Promise<void> {
  const file = await fileFrom(uri);
  if (!file) return;
  const result = await run(['check', '--json', file], { cwd: path.dirname(file) });
  const parsed = parseCheckReport(result.stdout);
  const out = output();
  out.appendLine(`$ arclang check --json ${path.basename(file)}`);
  out.appendLine(parsed ? summarizeCheck(parsed) : failureText(result));
  out.appendLine('');
  out.show(true);
  if (parsed && parsed.errors.length === 0) {
    const warnings = parsed.findings.filter((f) => f.severity !== 'info').length;
    vscode.window.showInformationMessage(warnings === 0 ? 'ArcLang: the model compiles with no finding.' : `ArcLang: the model compiles with ${warnings} finding${warnings > 1 ? 's' : ''} (see Problems).`);
  } else {
    vscode.window.showErrorMessage('ArcLang: the model does not compile; see the ArcLang output.');
  }
}

export async function runGate(uri?: vscode.Uri): Promise<void> {
  if (refusedHere(['gate'])) return; // web hook
  const file = await fileFrom(uri);
  if (!file) return;
  const standard = vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
  const result = await run(['gate', '--standard', standard, file], { cwd: path.dirname(file) });
  output().clear();
  report(`arclang gate --standard ${standard} ${path.basename(file)}`, result);
  if (result.code === 0) vscode.window.showInformationMessage(`ArcLang gate (${standard}): PASS`);
  else vscode.window.showWarningMessage(`ArcLang gate (${standard}): FAIL. See the ArcLang output.`);
}

export async function formatActive(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== 'arclang') {
    vscode.window.showInformationMessage('ArcLang: open a .arc file first.');
    return;
  }
  await vscode.commands.executeCommand('editor.action.formatDocument');
}

async function readPlan(file: string): Promise<TransitionPlan[] | null> {
  const result = await run(['transition', file, '--plan'], { cwd: path.dirname(file) });
  if (result.code !== 0) {
    report(`arclang transition ${path.basename(file)} --plan`, result);
    return null;
  }
  try {
    return JSON.parse(result.stdout) as TransitionPlan[];
  } catch {
    report(`arclang transition ${path.basename(file)} --plan`, result);
    return null;
  }
}

export async function transitionPlan(uri?: vscode.Uri): Promise<void> {
  const file = await fileFrom(uri);
  if (!file) return;
  const plans = await readPlan(file);
  if (!plans) return;
  const out = output();
  out.clear();
  out.appendLine(`$ arclang transition ${path.basename(file)} --plan`);
  out.append(renderTransitionPlan(plans, path.basename(file)));
  out.show(true);
}

/** Picks a step, runs the transition and applies the edited model in the editor (undoable, not saved). */
export async function transitionApply(uri?: vscode.Uri, presetStep?: string): Promise<void> {
  const file = await fileFrom(uri);
  if (!file) return;
  const plans = await readPlan(file);
  if (!plans) return;
  let step = presetStep;
  if (!step) {
    const picked = await vscode.window.showQuickPick(
      plans.map((plan) => {
        const pending = plan.candidates.filter((c) => c.realized_by.length === 0).length;
        return { label: plan.label, description: plan.step, detail: plan.blocked ? plan.blocked : `${pending} element${pending === 1 ? '' : 's'} to create`, step: plan.step };
      }),
      { placeHolder: 'Which Arcadia transition step?' },
    );
    if (!picked) return;
    step = picked.step;
  }
  const args = ['transition', file, '--step', step];
  if (step !== 'oa-to-sa') {
    const into = await vscode.window.showInputBox({
      prompt: step === 'sa-to-la' ? 'Logical component that receives the created elements (empty: let the compiler decide)' : 'Physical node that receives the created elements (empty: let the compiler decide)',
      placeHolder: 'id or name',
    });
    if (into === undefined) return;
    if (into.trim()) args.push('--into', into.trim());
  }
  const result = await run(args, { cwd: path.dirname(file) });
  if (result.code !== 0) {
    report(`arclang ${args.map((a) => (a === file ? path.basename(file) : a)).join(' ')}`, result);
    vscode.window.showErrorMessage(`ArcLang: the transition was refused. ${failureText(result).split('\n')[0]}`);
    return;
  }
  const document = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), result.stdout);
  await vscode.workspace.applyEdit(edit);
  await vscode.window.showTextDocument(document, { preview: false });
  vscode.window.showInformationMessage(`ArcLang: transition ${step} applied in the editor. Review it, then save.`);
}

export async function impact(uri?: vscode.Uri, element?: string): Promise<void> {
  if (refusedHere(['impact'])) return; // web hook
  const file = await fileFrom(uri);
  if (!file) return;
  let target = element;
  if (!target) {
    const editor = vscode.window.activeTextEditor;
    const word = editor ? editor.document.getText(editor.document.getWordRangeAtPosition(editor.selection.active, /[A-Za-z0-9_.-]+/)) : '';
    target = await vscode.window.showInputBox({ prompt: 'Element id or unambiguous name', value: word });
    if (!target) return;
  }
  const result = await run(['impact', file, target], { cwd: path.dirname(file) });
  output().clear();
  report(`arclang impact ${path.basename(file)} ${target}`, result);
}

function git(args: string[], cwd: string): Promise<{ code: number; stdout: string; stderr: string }> {
  return new Promise((resolve) => {
    execFile('git', args, { cwd, maxBuffer: 64 * 1024 * 1024 }, (error, stdout, stderr) => {
      resolve({ code: error && typeof error.code === 'number' ? error.code : error ? 1 : 0, stdout: String(stdout), stderr: String(stderr) });
    });
  });
}

/** `arclang diff` between the committed version (git HEAD) and the file on disk. */
export async function diffWithHead(uri?: vscode.Uri): Promise<void> {
  if (refusedHere(['diff'])) return; // web hook
  const file = await fileFrom(uri);
  if (!file) return;
  const dir = path.dirname(file);
  const top = await git(['rev-parse', '--show-toplevel'], dir);
  if (top.code !== 0) {
    vscode.window.showInformationMessage('ArcLang: the model is not in a git repository.');
    return;
  }
  const root = top.stdout.trim();
  const relative = path.relative(root, file).split(path.sep).join('/');
  const committed = await git(['show', `HEAD:${relative}`], root);
  if (committed.code !== 0) {
    vscode.window.showInformationMessage('ArcLang: the model has no committed version yet.');
    return;
  }
  const dirTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-diff-'));
  const old = path.join(dirTmp, path.basename(file));
  try {
    fs.writeFileSync(old, committed.stdout);
    const result = await run(['diff', old, file], { cwd: dir });
    output().clear();
    report(`arclang diff HEAD:${relative} ${path.basename(file)}`, result);
  } finally {
    fs.rmSync(dirTmp, { recursive: true, force: true });
  }
}

export async function exportModel(uri?: vscode.Uri): Promise<void> {
  const file = await fileFrom(uri);
  if (!file) return;
  const picked = await vscode.window.showQuickPick(
    EXPORT_FORMATS.map((f) => ({ label: f.label, description: f.format, format: f })),
    { placeHolder: 'Export format' },
  );
  if (!picked) return;
  if (refusedHere(['export', '-f', picked.format.format])) return; // web hook
  const base = path.basename(file, path.extname(file));
  const target = await vscode.window.showSaveDialog({
    defaultUri: vscode.Uri.file(path.join(path.dirname(file), `${base}.${picked.format.extension}`)),
    saveLabel: 'Export',
  });
  if (!target) return;
  const result = await run(['export', file, '-f', picked.format.format, '-o', target.fsPath], { cwd: path.dirname(file) });
  report(`arclang export ${path.basename(file)} -f ${picked.format.format} -o ${target.fsPath}`, result, result.code !== 0);
  if (result.code !== 0) {
    vscode.window.showErrorMessage(`ArcLang: export failed. ${failureText(result).split('\n')[0]}`);
    return;
  }
  const choice = await vscode.window.showInformationMessage(`ArcLang: exported to ${path.basename(target.fsPath)}.`, 'Open');
  if (choice === 'Open' && fs.existsSync(target.fsPath) && fs.statSync(target.fsPath).isFile()) {
    await vscode.commands.executeCommand('vscode.open', target);
  }
}

async function showMarkdown(title: string, args: string[]): Promise<void> {
  const result = await run(args);
  if (result.code !== 0) {
    report(`arclang ${args.join(' ')}`, result);
    return;
  }
  const document = await vscode.workspace.openTextDocument({ language: 'markdown', content: `<!-- ${title}: printed by arclang ${args.join(' ')} -->\n${result.stdout}` });
  await vscode.commands.executeCommand('markdown.showPreview', document.uri);
}

export const showMetamodel = (): Promise<void> => showMarkdown('ArcLang metamodel', ['metamodel', '--format', 'markdown']);
export const showRules = (): Promise<void> => showMarkdown('ArcLang rules', ['rules', '--format', 'markdown']);
