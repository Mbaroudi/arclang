// The commands of the impact analysis: "Analyze impact" on a diagram
// selection, a model tree item, the element under the cursor or the
// Properties pane; "What if…" (delete, change an attribute, retype, move),
// computed without writing anything; "Impact of my changes" since HEAD or
// main. Each opens (or retargets) the impact view of the model.
import * as vscode from 'vscode';
import { defaultQuery, ImpactFromWorkbench, ImpactQuery, WhatIfForm, whatIfQuery } from '../pure/impact';
import type { ElementRecord } from '../pure/workbench';
import { snapshotOf } from '../workbench/snapshot';
import { isModelDocument } from '../workbench/source';
import { elementAt } from '../workbench/tree';
import { ImpactPanel } from './panel';

interface Target {
  document: vscode.TextDocument;
  element: ElementRecord | null;
}

/** The document and the element an argument, the editors or a pick point at. */
async function targetOf(arg: unknown, pick: boolean): Promise<Target | null> {
  const object = (typeof arg === 'object' && arg !== null ? arg : {}) as { uri?: unknown; id?: unknown; node?: { element?: ElementRecord } };
  let document: vscode.TextDocument | null = null;
  if (arg instanceof vscode.Uri) document = await vscode.workspace.openTextDocument(arg);
  else if (object.uri instanceof vscode.Uri) document = await vscode.workspace.openTextDocument(object.uri);
  const editor = vscode.window.activeTextEditor;
  if (!document && editor && isModelDocument(editor.document)) document = editor.document;
  if (!document) document = ImpactPanel.activePanel()?.document ?? vscode.workspace.textDocuments.find(isModelDocument) ?? null;
  if (!document) {
    vscode.window.showInformationMessage('ArcLang: open a .arc or .sysml file first.');
    return null;
  }
  const snapshot = await snapshotOf(document);
  const elements = (snapshot.elements?.elements ?? []).filter((e) => e.kind !== 'Model');
  const byId = (id: string) => elements.find((e) => e.id === id) ?? elements.find((e) => e.name === id) ?? null;
  if (object.node?.element) return { document, element: byId(object.node.element.id) ?? object.node.element };
  if (typeof object.id === 'string') return { document, element: byId(object.id) };
  if (typeof arg === 'string') return { document, element: byId(arg) };
  if (editor && editor.document === document) {
    const at = await elementAt(document, editor.selection.active, elements);
    if (at) return { document, element: at };
  }
  if (!pick) return { document, element: null };
  const picked = await vscode.window.showQuickPick(
    elements.filter((e) => !/Port$/.test(e.kind)).map((e) => ({ label: e.name || e.id, description: `${e.id !== e.name ? `${e.id} · ` : ''}${e.kind}`, element: e })),
    { placeHolder: 'Impact of which element?', matchOnDescription: true },
  );
  return picked ? { document, element: picked.element } : null;
}

async function analyze(context: vscode.ExtensionContext, arg: unknown): Promise<void> {
  const target = await targetOf(arg, true);
  if (!target?.element) return;
  const base = ImpactPanel.all().find((p) => p.document === target.document)?.currentQuery;
  const query: ImpactQuery = { ...defaultQuery([target.element.id]), ...(base ? { depth: base.depth, kinds: base.kinds, direction: base.direction } : {}) };
  await ImpactPanel.open(context, target.document, query);
}

const shownValue = (value: unknown): string => {
  if (value === undefined || value === null) return '';
  if (typeof value === 'object' && 'value' in (value as object)) return `${(value as { value: unknown }).value} ${(value as { unit?: string }).unit ?? ''}`.trim();
  return String(value);
};

/** The small form of a what-if: what to change, then the value. */
async function whatIfForm(element: ElementRecord): Promise<WhatIfForm | null> {
  const label = element.name || element.id;
  const choice = await vscode.window.showQuickPick(
    [
      { label: '$(trash) Delete it', detail: 'with the traces and list entries that name it', op: 'delete' as const },
      { label: '$(edit) Change an attribute', detail: 'latency: 80 ms, safety_level: "ASIL-B"…', op: 'set' as const },
      { label: '$(symbol-class) Retype it', detail: 'is: another type', op: 'retype' as const },
      { label: '$(move) Move it', detail: 'into another element: a function reallocated, a component redeployed', op: 'move' as const },
    ],
    { placeHolder: `What if ${label}… (computed on a copy: nothing is written)` },
  );
  if (!choice) return null;
  if (choice.op === 'delete') return { op: 'delete', element: element.id };
  if (choice.op === 'set') {
    const keys = Object.keys(element.attributes ?? {}).filter((k) => k !== 'id');
    let key: string | undefined;
    if (keys.length) {
      const picked = await vscode.window.showQuickPick([...keys, '$(add) Another attribute…'], { placeHolder: `Attribute of ${label}` });
      key = picked?.startsWith('$(add)') ? await vscode.window.showInputBox({ prompt: 'Attribute name' }) : picked;
    } else {
      key = await vscode.window.showInputBox({ prompt: 'Attribute name', placeHolder: 'latency' });
    }
    if (!key) return null;
    const shown = shownValue(element.attributes?.[key]);
    const value = await vscode.window.showInputBox({ prompt: `${label}.${key} = (now ${shown || 'not set'})`, value: shown, placeHolder: '80 ms' });
    return value === undefined || !value.trim() ? null : { op: 'set', element: element.id, key, value };
  }
  if (choice.op === 'retype') {
    const type = await vscode.window.showInputBox({ prompt: `${label} is a…`, placeHolder: 'a type declared in the model' });
    return type?.trim() ? { op: 'retype', element: element.id, type } : null;
  }
  const container = await vscode.window.showInputBox({ prompt: `Move ${label} into…`, placeHolder: 'the id or name of an element, or a layer' });
  return container?.trim() ? { op: 'move', element: element.id, container } : null;
}

async function whatIf(context: vscode.ExtensionContext, arg: unknown): Promise<void> {
  const target = await targetOf(arg, true);
  if (!target?.element) return;
  const form = await whatIfForm(target.element);
  if (!form) return;
  await ImpactPanel.open(context, target.document, whatIfQuery(form, defaultQuery([])));
}

async function mine(context: vscode.ExtensionContext, arg: unknown): Promise<void> {
  const target = await targetOf(arg, false);
  if (!target) return;
  const picked = await vscode.window.showQuickPick(
    [
      { label: 'Since HEAD', detail: 'what is not committed yet, saved or not', revision: 'HEAD' },
      { label: 'Since main', detail: 'everything this branch changes', revision: 'main' },
      { label: 'Since another revision…', detail: 'a branch, a tag or a commit', revision: '' },
    ],
    { placeHolder: 'Impact of my changes' },
  );
  if (!picked) return;
  const revision = picked.revision || (await vscode.window.showInputBox({ prompt: 'Revision', placeHolder: 'origin/main, v4.5.0, a1b2c3d' }))?.trim();
  if (!revision) return;
  await ImpactPanel.open(context, target.document, { ...defaultQuery([]), since: revision });
}

/** "Analyze impact" and "What if…" on the element under the cursor. */
class ImpactActions implements vscode.CodeActionProvider {
  static readonly kinds = [vscode.CodeActionKind.Empty];

  async provideCodeActions(document: vscode.TextDocument, range: vscode.Range): Promise<vscode.CodeAction[]> {
    const snapshot = await snapshotOf(document);
    const element = await elementAt(document, range.start, (snapshot.elements?.elements ?? []).filter((e) => e.kind !== 'Model'));
    if (!element) return [];
    const label = element.name || element.id;
    const analyze = new vscode.CodeAction(`Analyze impact of ${label}`, vscode.CodeActionKind.Empty);
    analyze.command = { command: 'arclang.impact.analyze', title: 'Analyze impact', arguments: [{ uri: document.uri, id: element.id }] };
    const what = new vscode.CodeAction(`What if ${label}…`, vscode.CodeActionKind.Empty);
    what.command = { command: 'arclang.impact.whatIf', title: 'What if…', arguments: [{ uri: document.uri, id: element.id }] };
    return [analyze, what];
  }
}

/** The workbench page's own asks (impact hook in src/workbench/panel.ts). */
export function isImpactMessage(message: unknown): message is ImpactFromWorkbench {
  const type = (message as { type?: unknown } | null)?.type;
  return type === 'impact-open' || type === 'impact-hello';
}

export async function handleImpactMessage(document: vscode.TextDocument, message: ImpactFromWorkbench): Promise<void> {
  if (message.type === 'impact-hello') {
    for (const panel of ImpactPanel.all().filter((p) => p.document.uri.toString() === document.uri.toString())) panel.remark();
    return;
  }
  await vscode.commands.executeCommand(message.whatIf ? 'arclang.impact.whatIf' : 'arclang.impact.analyze', { uri: document.uri, id: message.id });
}

export function registerImpact(context: vscode.ExtensionContext): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  register('arclang.impact.analyze', (arg?: unknown) => analyze(context, arg));
  register('arclang.impact.whatIf', (arg?: unknown) => whatIf(context, arg));
  register('arclang.impact.mine', (arg?: unknown) => mine(context, arg));
  register('arclang.impact.refresh', () => ImpactPanel.activePanel()?.refresh());
  context.subscriptions.push(vscode.languages.registerCodeActionsProvider([{ language: 'arclang' }, { language: 'sysml' }], new ImpactActions(), { providedCodeActionKinds: ImpactActions.kinds }));
}
