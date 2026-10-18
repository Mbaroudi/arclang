// Language model tools: the compiler's answers, for agent mode and for @arclang.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run, RunResult, withTempFile } from '../cli';
import { clip, exitLine, parseCheckReport, renderTransitionPlan, summarizeCheck, TransitionPlan } from '../pure/arc';

const MAX_RESULT = 24000;

function answer(lines: Array<string | undefined>): vscode.LanguageModelToolResult {
  return new vscode.LanguageModelToolResult([new vscode.LanguageModelTextPart(lines.filter((l): l is string => !!l).join('\n'))]);
}

/** A path the tools may touch: inside a workspace folder, or open in the editor. */
export function allowedPath(candidate: string | undefined): string | null {
  if (!candidate) return null;
  const resolved = path.resolve(candidate);
  const folders = vscode.workspace.workspaceFolders ?? [];
  const inWorkspace = folders.some((f) => resolved === f.uri.fsPath || resolved.startsWith(f.uri.fsPath + path.sep));
  const open = vscode.workspace.textDocuments.some((d) => d.uri.scheme === 'file' && d.uri.fsPath === resolved);
  return inWorkspace || open ? resolved : null;
}

function refused(candidate: string | undefined): vscode.LanguageModelToolResult {
  return answer([`Refused: ${candidate ?? '(no path)'} is neither in the workspace nor open in the editor. Give the absolute path of a model of the workspace.`]);
}

/** Runs the compiler on the model, on the editor's buffer when the file is open with unsaved changes. */
async function runOnModel(file: string, args: (model: string) => string[], token: vscode.CancellationToken): Promise<RunResult> {
  const open = vscode.workspace.textDocuments.find((d) => d.uri.scheme === 'file' && d.uri.fsPath === file);
  if (open && open.isDirty) {
    return withTempFile(open.getText(), path.basename(file), (tmp) => run(args(tmp), { cwd: path.dirname(file), token }));
  }
  return run(args(file), { cwd: path.dirname(file), token });
}

function checkAnswer(label: string, result: RunResult): vscode.LanguageModelToolResult {
  const parsed = parseCheckReport(result.stdout);
  if (!parsed) return answer([exitLine(label, result.code), failureText(result)]);
  return answer([exitLine(label, result.code), summarizeCheck(parsed), parsed.findings.length ? `Findings as JSON:\n${clip(JSON.stringify(parsed.findings, null, 1), MAX_RESULT)}` : undefined]);
}

/** Replaces the whole text of the document at `file` with `text`, as an undoable edit. */
async function replaceDocument(file: string, text: string): Promise<void> {
  const document = await vscode.workspace.openTextDocument(vscode.Uri.file(file));
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), text);
  await vscode.workspace.applyEdit(edit);
}

/** The `#### Kind` section of the Markdown metamodel, or every line naming the kind. */
function metamodelSection(markdown: string, kind: string): string {
  const wanted = kind.replace(/[^a-z0-9]/gi, '').toLowerCase();
  const lines = markdown.split('\n');
  const start = lines.findIndex((l) => /^#{3,4} /.test(l) && l.replace(/[^a-z0-9]/gi, '').toLowerCase().endsWith(wanted));
  if (start >= 0) {
    const level = lines[start].match(/^#+/)?.[0].length ?? 4;
    let end = start + 1;
    while (end < lines.length && !new RegExp(`^#{1,${level}} `).test(lines[end])) end++;
    return lines.slice(start, end).join('\n');
  }
  const mentions = lines.filter((l) => l.toLowerCase().includes(kind.toLowerCase()));
  return mentions.length ? mentions.join('\n') : `No kind named ${kind} in the metamodel.`;
}

export function registerTools(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.lm.registerTool<{ path?: string; source?: string }>('arclang_check', {
      prepareInvocation: () => ({ invocationMessage: 'Compiling the model with arclang' }),
      async invoke(options, token) {
        const { path: candidate, source } = options.input;
        if (source) {
          const result = await withTempFile(source, 'model.arc', (tmp) => run(['check', '--json', tmp], { token }));
          return checkAnswer('check --json (source text)', result);
        }
        const file = allowedPath(candidate);
        if (!file) return refused(candidate);
        return checkAnswer(`check --json ${path.basename(file)}`, await runOnModel(file, (m) => ['check', '--json', m], token));
      },
    }),

    vscode.lm.registerTool<{ path: string; standard?: string }>('arclang_gate', {
      prepareInvocation: () => ({ invocationMessage: 'Running the ArcLang production gate' }),
      async invoke(options, token) {
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const standard = options.input.standard ?? vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
        const result = await runOnModel(file, (m) => ['gate', '--standard', standard, m], token);
        return answer([exitLine(`gate --standard ${standard} ${path.basename(file)}`, result.code), result.code === 0 ? 'Verdict: PASS' : 'Verdict: FAIL', clip(result.stdout + (result.stderr ? `\n${result.stderr}` : ''), MAX_RESULT)]);
      },
    }),

    vscode.lm.registerTool<{ kind?: string }>('arclang_metamodel', {
      prepareInvocation: () => ({ invocationMessage: 'Reading the ArcLang metamodel' }),
      async invoke(options, token) {
        const result = await run(['metamodel', '--format', 'markdown'], { token });
        if (result.code !== 0) return answer([exitLine('metamodel', result.code), failureText(result)]);
        const text = options.input.kind ? metamodelSection(result.stdout, options.input.kind) : result.stdout;
        return answer([exitLine('metamodel --format markdown', 0), clip(text, 60000)]);
      },
    }),

    vscode.lm.registerTool<Record<string, never>>('arclang_rules', {
      prepareInvocation: () => ({ invocationMessage: 'Reading the catalogue of rules' }),
      async invoke(_options, token) {
        const result = await run(['rules', '--format', 'json'], { token });
        return answer([exitLine('rules --format json', result.code), result.code === 0 ? clip(result.stdout, MAX_RESULT) : failureText(result)]);
      },
    }),

    vscode.lm.registerTool<{ path: string; element: string }>('arclang_impact', {
      prepareInvocation: (options) => ({ invocationMessage: `Change impact of ${options.input.element}` }),
      async invoke(options, token) {
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const result = await runOnModel(file, (m) => ['impact', m, options.input.element], token);
        return answer([exitLine(`impact ${path.basename(file)} ${options.input.element}`, result.code), clip(result.code === 0 ? result.stdout : failureText(result), MAX_RESULT)]);
      },
    }),

    vscode.lm.registerTool<{ path: string; step?: string }>('arclang_transition_plan', {
      prepareInvocation: () => ({ invocationMessage: 'Planning the Arcadia transitions' }),
      async invoke(options, token) {
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const args = (m: string) => ['transition', m, '--plan', ...(options.input.step ? ['--step', options.input.step] : [])];
        const result = await runOnModel(file, args, token);
        if (result.code !== 0) return answer([exitLine('transition --plan', result.code), failureText(result)]);
        let rendered = result.stdout;
        try {
          rendered = renderTransitionPlan(JSON.parse(result.stdout) as TransitionPlan[], path.basename(file));
        } catch {
          // the compiler's text as is
        }
        return answer([exitLine(`transition ${path.basename(file)} --plan`, 0), clip(rendered, MAX_RESULT)]);
      },
    }),

    vscode.lm.registerTool<{ path: string; element: string; key: string; value: string }>('arclang_set_attribute', {
      prepareInvocation: (options) => ({
        invocationMessage: `Setting ${options.input.key} on ${options.input.element}`,
        confirmationMessages: {
          title: 'ArcLang: set an attribute',
          message: new vscode.MarkdownString(`Set \`${options.input.key}: ${options.input.value}\` on **${options.input.element}** in \`${path.basename(options.input.path ?? '')}\`? The compiler refuses the edit unless the model still compiles; the change is applied in the editor and not saved.`),
        },
      }),
      async invoke(options, token) {
        const { element, key, value } = options.input;
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const result = await runOnModel(file, (m) => ['set', m, element, key, value], token);
        const label = `set ${path.basename(file)} ${element} ${key} ${value}`;
        if (result.code !== 0) return answer([exitLine(label, result.code), failureText(result)]);
        await replaceDocument(file, result.stdout);
        return answer([exitLine(label, 0), `Applied in the editor to ${path.basename(file)} (not saved); the compiler accepted the edited model.`]);
      },
    }),

    vscode.lm.registerTool<{ path: string; element: string; name: string }>('arclang_rename', {
      prepareInvocation: (options) => ({
        invocationMessage: `Renaming ${options.input.element}`,
        confirmationMessages: {
          title: 'ArcLang: rename an element',
          message: new vscode.MarkdownString(`Rename **${options.input.element}** to \`${options.input.name}\` in \`${path.basename(options.input.path ?? '')}\`? Refused if the element is known by its name elsewhere. Applied in the editor, not saved.`),
        },
      }),
      async invoke(options, token) {
        const { element, name } = options.input;
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const result = await runOnModel(file, (m) => ['rename', m, element, name], token);
        const label = `rename ${path.basename(file)} ${element} ${name}`;
        if (result.code !== 0) return answer([exitLine(label, result.code), failureText(result)]);
        await replaceDocument(file, result.stdout);
        return answer([exitLine(label, 0), `Applied in the editor to ${path.basename(file)} (not saved).`]);
      },
    }),
  );
}
