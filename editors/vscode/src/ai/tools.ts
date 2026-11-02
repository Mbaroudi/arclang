// Language model tools: the compiler's answers, for agent mode and for @arclang.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run, RunResult, withTempFile } from '../cli';
import { clip, exitLine, parseCheckReport, renderTransitionPlan, summarizeCheck, TransitionPlan } from '../pure/arc';
import { confirmEdits, mdText } from '../pure/confirm';
import { EditRequest, layerOfKind, Metamodel, neighbors, Snapshot } from '../pure/workbench';
import { applyRequests } from '../workbench/edits';
import { metamodelJson, snapshotOf } from '../workbench/snapshot';

const MAX_RESULT = 24000;

function answer(lines: Array<string | undefined>): vscode.LanguageModelToolResult {
  return new vscode.LanguageModelToolResult([new vscode.LanguageModelTextPart(lines.filter((l): l is string => !!l).join('\n'))]);
}

const realPath = (p: string): string | null => {
  try {
    return fs.realpathSync.native(p);
  } catch {
    return null;
  }
};

/**
 * A path the tools may touch: inside a workspace folder, or open in the
 * editor. The answer is the real path (symbolic links resolved), the one
 * that was checked: what is read or written is what was allowed.
 */
export function allowedPath(candidate: string | undefined): string | null {
  if (!candidate || typeof candidate !== 'string') return null;
  const target = realPath(path.resolve(candidate));
  if (!target) return null;
  // Symbolic links are followed before the comparison: a link inside the
  // workspace to a file outside it does not pass.
  const folders = (vscode.workspace.workspaceFolders ?? []).map((f) => realPath(f.uri.fsPath)).filter((f): f is string => f !== null);
  const inWorkspace = folders.some((f) => target === f || target.startsWith(f + path.sep));
  const open = vscode.workspace.textDocuments.some((d) => d.uri.scheme === 'file' && realPath(d.uri.fsPath) === target);
  return inWorkspace || open ? target : null;
}

/** The path a confirmation shows: relative to the workspace, whole, or why it is refused. */
function shownPath(candidate: string | undefined): string {
  const file = allowedPath(candidate);
  if (!file) return `${candidate ?? '(no path)'} (outside the workspace: it will be refused)`;
  return vscode.workspace.asRelativePath(file, (vscode.workspace.workspaceFolders ?? []).length > 1);
}

/** The open document of a file, whatever path (link or real) it was opened by. */
function openDocument(file: string): vscode.TextDocument | undefined {
  return vscode.workspace.textDocuments.find((d) => d.uri.scheme === 'file' && (d.uri.fsPath === file || realPath(d.uri.fsPath) === file));
}

function refused(candidate: string | undefined): vscode.LanguageModelToolResult {
  return answer([`Refused: ${candidate ?? '(no path)'} is neither in the workspace nor open in the editor. Give the absolute path of a model of the workspace.`]);
}

/** Runs the compiler on the model, on the editor's buffer when the file is open with unsaved changes. */
async function runOnModel(file: string, args: (model: string) => string[], token: vscode.CancellationToken): Promise<RunResult> {
  const open = openDocument(file);
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
  const document = openDocument(file) ?? (await vscode.workspace.openTextDocument(vscode.Uri.file(file)));
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
          const result = await withTempFile(source, 'model.arc', (tmp, dir) => run(['check', '--json', tmp], { cwd: dir, token }));
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
        const result = await runOnModel(file, (m) => ['gate', `--standard=${standard}`, '--', m], token);
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
      prepareInvocation: () => ({ invocationMessage: 'Reading the catalog of rules' }),
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
        const result = await runOnModel(file, (m) => ['impact', '--', m, options.input.element], token);
        return answer([exitLine(`impact ${path.basename(file)} ${options.input.element}`, result.code), clip(result.code === 0 ? result.stdout : failureText(result), MAX_RESULT)]);
      },
    }),

    vscode.lm.registerTool<{ path: string; step?: string }>('arclang_transition_plan', {
      prepareInvocation: () => ({ invocationMessage: 'Planning the Arcadia transitions' }),
      async invoke(options, token) {
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const args = (m: string) => ['transition', '--plan', ...(options.input.step ? [`--step=${options.input.step}`] : []), '--', m];
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
          message: new vscode.MarkdownString(`Set **${mdText(options.input.key)}** to ${mdText(options.input.value)} on ${mdText(options.input.element)} in ${mdText(shownPath(options.input.path))}? The compiler refuses the edit unless the model still compiles; the change is applied in the editor and not saved.`),
        },
      }),
      async invoke(options, token) {
        const { element, key, value } = options.input;
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const result = await runOnModel(file, (m) => ['set', '--', m, element, key, value], token);
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
          message: new vscode.MarkdownString(`Rename ${mdText(options.input.element)} to ${mdText(`"${options.input.name}"`)} in ${mdText(shownPath(options.input.path))}? Refused if the element is known by its name elsewhere. Applied in the editor, not saved.`),
        },
      }),
      async invoke(options, token) {
        const { element, name } = options.input;
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const result = await runOnModel(file, (m) => ['rename', '--', m, element, name], token);
        const label = `rename ${path.basename(file)} ${element} ${name}`;
        if (result.code !== 0) return answer([exitLine(label, result.code), failureText(result)]);
        await replaceDocument(file, result.stdout);
        return answer([exitLine(label, 0), `Applied in the editor to ${path.basename(file)} (not saved).`]);
      },
    }),

    vscode.lm.registerTool<{ path: string; kind?: string; element?: string; layer?: string }>('arclang_model', {
      prepareInvocation: (options) => ({ invocationMessage: `Reading the model ${path.basename(options.input.path ?? '')}` }),
      async invoke(options) {
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const document = openDocument(file) ?? (await vscode.workspace.openTextDocument(vscode.Uri.file(file)));
        const label = `workbench ${path.basename(file)}`;
        const snapshot = await snapshotOf(document);
        if (!snapshot.success || !snapshot.elements) return answer([exitLine(label, 1), snapshot.errors.join('\n') || 'the model does not compile']);
        const metamodel = await metamodelJson();
        return answer([exitLine(label, 0), clip(describeModel(snapshot, metamodel, options.input), MAX_RESULT)]);
      },
    }),

    vscode.lm.registerTool<{ path: string; requests: EditRequest[] }>('arclang_edit', {
      prepareInvocation: (options) => ({
        invocationMessage: `Editing ${path.basename(options.input.path ?? '')} (${(options.input.requests ?? []).length} request${(options.input.requests ?? []).length === 1 ? '' : 's'})`,
        confirmationMessages: {
          title: 'ArcLang: edit the model',
          message: new vscode.MarkdownString(confirmEdits(shownPath(options.input.path), Array.isArray(options.input.requests) ? options.input.requests : [])),
        },
      }),
      async invoke(options) {
        const file = allowedPath(options.input.path);
        if (!file) return refused(options.input.path);
        const requests = Array.isArray(options.input.requests) ? options.input.requests : [];
        if (requests.length === 0) return answer(['Refused: no request. Give a list of edit requests ({"op": "add", ...}).']);
        const document = openDocument(file) ?? (await vscode.workspace.openTextDocument(vscode.Uri.file(file)));
        const outcome = await applyRequests(document, requests);
        const label = `edit ${path.basename(file)} (${requests.length})`;
        if (!outcome.ok) {
          return answer([exitLine(label, 1), `Refused at request ${outcome.at + 1} (${describeRequest(requests[outcome.at])}): ${outcome.error}`, 'Nothing was changed.']);
        }
        return answer([exitLine(label, 0), `Applied in the editor to ${path.basename(file)} as one undoable change (not saved); the compiler accepted the edited model.`]);
      },
    }),
  );
}

/** One request in a sentence, for confirmations and refusals. */
export function describeRequest(request: EditRequest): string {
  switch (request.op) {
    case 'set':
      return `set \`${request.key}: ${request.value}\` on ${request.element}`;
    case 'unset':
      return `remove \`${request.key}\` from ${request.element}`;
    case 'rename':
      return `rename ${request.element} to "${request.name}"`;
    case 'add':
      return `add ${request.keyword} "${request.name}" ${request.container ? `inside ${request.container}` : request.layer ? `in ${request.layer}` : 'at the top level'}`;
    case 'add_raw':
      return `add the declaration \`${request.declaration.split('\n')[0]}…\` ${request.container ? `inside ${request.container}` : request.layer ? `in ${request.layer}` : 'at the top level'}`;
    case 'remove':
      return `delete ${request.element}`;
    case 'move':
      return `move ${request.element} into ${request.container ?? request.layer ?? '?'}`;
    case 'add_trace':
      return `trace ${request.from} ${request.kind} ${request.to}`;
    case 'remove_trace':
      return `remove the trace ${request.from} ${request.kind} ${request.to}`;
    case 'wrap_fragment':
      return `wrap the messages ${request.first}…${request.last} of ${request.scenario} in a ${request.kind} fragment`;
    default:
      return JSON.stringify(request);
  }
}

/** The model as an agent reads it: counts, the elements asked for, the findings, the gate. */
export function describeModel(snapshot: Snapshot, metamodel: Metamodel | null, filter: { kind?: string; element?: string; layer?: string }): string {
  const records = snapshot.elements?.elements ?? [];
  const byUuid = new Map(records.map((e) => [e.uuid, e]));
  const counts = new Map<string, number>();
  for (const e of records) counts.set(e.kind, (counts.get(e.kind) ?? 0) + 1);
  const lines: string[] = [];
  lines.push(`Elements by kind: ${[...counts.entries()].map(([k, n]) => `${k} ${n}`).join(', ')}.`);
  const wanted = records.filter((e) => {
    if (filter.element && e.id !== filter.element && e.name !== filter.element) return false;
    if (filter.kind && e.kind.toLowerCase() !== filter.kind.toLowerCase()) return false;
    if (filter.layer && layerOfKind(e.kind, metamodel).toLowerCase() !== filter.layer.toLowerCase()) return false;
    return true;
  });
  const limit = filter.element || filter.kind || filter.layer ? 400 : 150;
  lines.push(`${wanted.length} element${wanted.length === 1 ? '' : 's'}${wanted.length > limit ? ` (first ${limit})` : ''}: id | kind | name | owner | attributes`);
  for (const e of wanted.slice(0, limit)) {
    const owner = e.owner ? (byUuid.get(e.owner)?.id ?? '') : '';
    const attributes = Object.entries(e.attributes)
      .filter(([k]) => k !== 'id' && k !== 'name')
      .map(([k, v]) => `${k}=${JSON.stringify(v)}`)
      .join(' ');
    lines.push(`${e.id} | ${e.kind} | ${e.name} | ${owner} | ${attributes}`);
  }
  if (filter.element && wanted.length === 1) {
    const around = neighbors(wanted[0], snapshot);
    lines.push(`Incoming: ${around.incoming.map((n) => `${n.kind}${n.name ? ` ${n.name}` : ''} from ${n.other?.id ?? '?'}`).join('; ') || 'none'}`);
    lines.push(`Outgoing: ${around.outgoing.map((n) => `${n.kind}${n.name ? ` ${n.name}` : ''} to ${n.other?.id ?? '?'}`).join('; ') || 'none'}`);
  }
  const relationships = snapshot.elements?.relationships ?? [];
  if (!filter.element) lines.push(`Relationships: ${relationships.length} (${[...new Set(relationships.map((r) => r.kind))].join(', ')}); traces: ${snapshot.diagrams?.links.length ?? 0}.`);
  lines.push(`Findings: ${snapshot.findings.length}${snapshot.findings.length ? ` (${snapshot.findings.slice(0, 20).map((f) => `${f.rule}: ${f.message}`).join(' | ')})` : ''}.`);
  if (snapshot.gate) lines.push(`Gate: ${snapshot.gate.passed ? 'PASS' : 'FAIL'}, ${snapshot.gate.requirements_verified}/${snapshot.gate.requirements_total} requirements verified.`);
  return lines.join('\n');
}
