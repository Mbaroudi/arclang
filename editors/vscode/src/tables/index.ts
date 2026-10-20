// The commands of the live tables: open a declared table (from the model
// tree, where tables sit under their layer, or from a pick), start a new
// one (a kind, a scope), and the table view's own ⌘Z / ⌘⇧Z.
import * as vscode from 'vscode';
import { run, withTempFile } from '../cli';
import { emptySpec, TABLE_KINDS } from '../pure/tables';
import { snapshotOf } from '../workbench/snapshot';
import { arcTextOf, isModelDocument } from '../workbench/source';
import { TablePanel } from './panel';

const SCOPE_OF_LAYER: Record<string, string> = { Operational: 'OA', System: 'SA', Logical: 'LA', Physical: 'PA', Epbs: 'EPBS' };

const SCOPES: (vscode.QuickPickItem & { scope: string | null })[] = [
  { label: 'System Analysis', description: 'SA', scope: 'SA' },
  { label: 'Logical Architecture', description: 'LA', scope: 'LA' },
  { label: 'Physical Architecture', description: 'PA', scope: 'PA' },
  { label: 'Operational Analysis', description: 'OA', scope: 'OA' },
  { label: 'EPBS', description: 'EPBS', scope: 'EPBS' },
  { label: 'Whole model', description: 'every layer', scope: null },
];

/** The model document an argument or the editors point at. */
async function modelDocument(arg: unknown): Promise<vscode.TextDocument | null> {
  const object = (typeof arg === 'object' && arg !== null ? arg : {}) as { uri?: unknown };
  if (arg instanceof vscode.Uri) return vscode.workspace.openTextDocument(arg);
  if (object.uri instanceof vscode.Uri) return vscode.workspace.openTextDocument(object.uri);
  const editor = vscode.window.activeTextEditor;
  if (editor && isModelDocument(editor.document)) return editor.document;
  const table = TablePanel.activePanel();
  if (table) return table.document;
  const open = vscode.workspace.textDocuments.find(isModelDocument);
  if (open) return open;
  vscode.window.showInformationMessage('ArcLang: open a .arc or .sysml file first.');
  return null;
}

/** Ask for a kind and a scope, and open the table: not in the model until saved from the view. */
async function newTable(context: vscode.ExtensionContext, arg: unknown): Promise<void> {
  const document = await modelDocument(arg);
  if (!document) return;
  const kind = await vscode.window.showQuickPick(
    TABLE_KINDS.map((k) => ({ label: k.label, description: k.kind, detail: k.detail, tableKind: k.kind })),
    { placeHolder: 'New table: what should it show?', matchOnDescription: true },
  );
  if (!kind) return;
  const layer = (arg as { type?: string; layer?: string } | undefined)?.type === 'layer' ? SCOPE_OF_LAYER[(arg as { layer: string }).layer] : undefined;
  let scope: string | null | undefined = layer;
  if (scope === undefined) {
    const defaults: Record<string, string | null> = { requirements_coverage: 'SA', traceability_matrix: null, elements: null };
    const preferred = kind.tableKind in defaults ? defaults[kind.tableKind] : 'LA';
    const ordered = [...SCOPES].sort((a, b) => Number(b.scope === preferred) - Number(a.scope === preferred));
    const picked = await vscode.window.showQuickPick(ordered, { placeHolder: `Scope of the ${kind.label.toLowerCase()}` });
    if (!picked) return;
    scope = picked.scope;
  }
  const spec = { ...emptySpec(kind.tableKind, scope ?? undefined), name: kind.label };
  if (kind.tableKind === 'elements') {
    const snapshot = await snapshotOf(document);
    const kinds = [...new Set((snapshot.elements?.elements ?? []).map((e) => e.kind).filter((k) => k !== 'Model'))].sort();
    const rowType = await vscode.window.showQuickPick(kinds, { placeHolder: 'Rows: elements of which kind?' });
    if (!rowType) return;
    spec.row_type = rowType;
    spec.name = `${rowType} elements`;
  }
  await TablePanel.open(context, document, spec, null);
}

/** Open a declared table: the tree's item, a name, else a pick among the declared ones. */
async function openTable(context: vscode.ExtensionContext, arg: unknown): Promise<void> {
  const item = (typeof arg === 'object' && arg !== null ? arg : {}) as { type?: string; node?: { element?: { kind?: string; name?: string; id?: string } }; name?: string; uri?: vscode.Uri };
  const document = await modelDocument(arg);
  if (!document) return;
  const fromTree = item.type === 'element' && item.node?.element?.kind === 'Table' ? item.node.element.name ?? item.node.element.id ?? null : null;
  let name = fromTree ?? (typeof item.name === 'string' ? item.name : null);
  if (!name) {
    const source = await arcTextOf(document);
    const listing = source.ok ? await withTempFile(source.text, 'model.arc', (file) => run(['table', file, '--list'])) : { code: 1, stdout: '', stderr: source.error };
    let declared: { name: string; kind: string; scope: string | null }[] = [];
    try {
      declared = (JSON.parse(listing.stdout) as { declared: { name: string; kind: string; scope: string | null }[] }).declared;
    } catch {
      declared = [];
    }
    const NEW = '$(add) New table…';
    const picked = await vscode.window.showQuickPick(
      [...declared.map((t) => ({ label: `$(table) ${t.name}`, description: `${t.kind}${t.scope ? ` · ${t.scope}` : ''}`, name: t.name })), { label: NEW, description: 'a kind and a scope', name: '' }],
      { placeHolder: declared.length ? `Open a table of ${document.uri.path.split('/').pop()}` : 'This model declares no table yet' },
    );
    if (!picked) return;
    if (!picked.name) return newTable(context, document.uri);
    name = picked.name;
  }
  await TablePanel.open(context, document, emptySpec('requirements_coverage'), name);
}

export function registerTables(context: vscode.ExtensionContext): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  register('arclang.table.open', (arg?: unknown) => openTable(context, arg));
  register('arclang.table.new', (arg?: unknown) => newTable(context, arg));
  register('arclang.table.coverage', async (arg?: unknown) => {
    const document = await modelDocument(arg);
    if (document) await TablePanel.open(context, document, { ...emptySpec('requirements_coverage', 'SA'), name: 'Requirement coverage' }, null);
  });
  register('arclang.table.undo', () => TablePanel.activePanel()?.undo());
  register('arclang.table.redo', () => TablePanel.activePanel()?.redo());
  register('arclang.table.refresh', () => TablePanel.activePanel()?.refresh());
}
