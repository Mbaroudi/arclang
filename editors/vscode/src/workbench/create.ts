// New and Open: a new model (blank, the Arcadia skeleton, an example), a new
// layer on the open model, a new diagram, and a diagram picked by layer. The
// text of a new layer or diagram is written through `arclang edit`: the
// compiler checks it and one undo takes it back.
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { arcadiaSkeleton, arcFileName, blankModel, diagramDeclaration, LayerChoice, layerBlock, missingLayers, modelNameOf, uniqueName } from '../pure/creation';
import { VIEW_OF_LAYER, VIEWPOINTS, ViewerGraph } from '../pure/workbench';
import { applyRequests, refusalMessage } from './edits';
import { WorkbenchPanel } from './panel';
import { openDiagramTab } from './panes-host';
import { forgetSnapshot, snapshotOf } from './snapshot';
import { arcTextOf, isModelDocument } from './source';
import { ModelTreeProvider } from './tree';

/** The views a new diagram may be cut from. */
const DIAGRAM_VIEWS = ['oab', 'sab', 'lab', 'pab', 'cap', 'cdb'];

const NEW_MODEL_KINDS = [
  { id: 'blank', label: '$(file) Blank model', detail: 'A model header only: name and version' },
  { id: 'skeleton', label: '$(symbol-structure) Arcadia skeleton', detail: 'The four levels (operational, system, logical, physical), empty' },
  { id: 'example', label: '$(book) From an example', detail: 'A small complete model to start from' },
] as const;

/** The document the command acts on: the one given, the workbench's, the tree's, the active text. */
async function targetDocument(arg: unknown, tree: ModelTreeProvider): Promise<vscode.TextDocument | null> {
  if (arg instanceof vscode.Uri) return vscode.workspace.openTextDocument(arg);
  const panelDocument = WorkbenchPanel.activeDocument();
  if (panelDocument) return panelDocument;
  const active = vscode.window.activeTextEditor;
  if (active && isModelDocument(active.document)) return active.document;
  if (tree.shown) return tree.shown;
  void vscode.window.showInformationMessage('ArcLang: open a model first, or create one with "New model…".', 'New model…').then((choice) => {
    if (choice) void vscode.commands.executeCommand('arclang.newModel');
  });
  return null;
}

async function exists(uri: vscode.Uri): Promise<boolean> {
  try {
    await vscode.workspace.fs.stat(uri);
    return true;
  } catch {
    return false;
  }
}

/** Write a new model file, open its text and the workbench side by side. */
export async function createModel(context: vscode.ExtensionContext, tree: ModelTreeProvider, kind?: (typeof NEW_MODEL_KINDS)[number]['id']): Promise<void> {
  const chosen = kind ?? (await vscode.window.showQuickPick([...NEW_MODEL_KINDS], { placeHolder: 'New model: start from…', title: 'ArcLang: New model' }))?.id;
  if (!chosen) return;
  let example: string | null = null;
  if (chosen === 'example') {
    const pick = await vscode.window.showQuickPick([{ label: '$(car) Lane Keeping Assist', detail: 'Requirements, four Arcadia levels, a functional chain, safety and a diagram', file: 'reference.arc' }], { placeHolder: 'Which example?' });
    if (!pick) return;
    example = Buffer.from(await vscode.workspace.fs.readFile(vscode.Uri.joinPath(context.extensionUri, 'media', pick.file))).toString('utf8');
  }
  const folder = vscode.workspace.workspaceFolders?.[0]?.uri ?? vscode.Uri.file(os.homedir());
  const target = await vscode.window.showSaveDialog({
    defaultUri: vscode.Uri.joinPath(folder, chosen === 'example' ? 'lane_keeping.arc' : 'model.arc'),
    filters: { ArcLang: ['arc'] },
    saveLabel: 'Create model',
    title: 'ArcLang: name and place of the new model',
  });
  if (!target) return;
  const file = target.fsPath.toLowerCase().endsWith('.arc') ? target : vscode.Uri.file(arcFileName(target.fsPath));
  const name = modelNameOf(path.basename(file.fsPath));
  const text = example ?? (chosen === 'skeleton' ? arcadiaSkeleton(name) : blankModel(name));
  if (await exists(file)) await vscode.workspace.fs.delete(file); // the save dialog already asked to overwrite
  await vscode.workspace.fs.writeFile(file, Buffer.from(text, 'utf8'));
  const document = await vscode.workspace.openTextDocument(file);
  await vscode.window.showTextDocument(document, { preview: false });
  tree.follow(document);
  forgetSnapshot(document.uri);
  await WorkbenchPanel.open(document);
}

/** Submit edit requests through the open workbench (one step of its history) or straight to the text. */
async function submit(document: vscode.TextDocument, request: Parameters<typeof applyRequests>[1], label: string): Promise<boolean> {
  const page = WorkbenchPanel.for(document.uri);
  if (page) {
    await page.submitEdit(request, label);
    return true;
  }
  const outcome = await applyRequests(document, request);
  if (!outcome.ok) void vscode.window.showWarningMessage(refusalMessage(label, outcome));
  return outcome.ok;
}

/** A level or a section the model does not hold yet, added as an empty block. */
export async function createLayer(document: vscode.TextDocument): Promise<void> {
  const source = await arcTextOf(document);
  if (!source.ok) return void vscode.window.showWarningMessage(`ArcLang: ${source.error}`);
  const missing = missingLayers(source.text);
  if (missing.length === 0) return void vscode.window.showInformationMessage('ArcLang: this model already holds every level and section.');
  const pick = await vscode.window.showQuickPick(
    missing.map((choice) => ({ label: choice.label, description: choice.keyword, detail: choice.detail, choice })),
    { placeHolder: 'New layer: the levels and sections this model does not hold yet', title: 'ArcLang: New layer' },
  );
  if (!pick) return;
  const choice: LayerChoice = pick.choice;
  if (!(await submit(document, [{ op: 'add_raw', declaration: layerBlock(choice), top_level: true }], `new layer ${choice.label}`))) return;
  await showLayerTab(document, choice);
}

/** The tab of a layer just added, once the compiler has drawn it. */
async function showLayerTab(document: vscode.TextDocument, choice: LayerChoice): Promise<void> {
  const layer = Object.entries({ operational_analysis: 'Operational', system_analysis: 'System', logical_architecture: 'Logical', physical_architecture: 'Physical', epbs: 'Epbs' }).find(([keyword]) => keyword === choice.keyword)?.[1];
  const view = layer ? VIEW_OF_LAYER[layer] : null;
  if (!view) return;
  const page = WorkbenchPanel.for(document.uri);
  if (!page) return;
  forgetSnapshot(document.uri);
  const graph = ((await snapshotOf(document)).diagrams?.graphs ?? []).find((g) => g.arc.view === view);
  if (graph) page.show(graph.id);
}

/** A named diagram of a chosen view, starting from the selected element. */
export async function createDiagram(document: vscode.TextDocument): Promise<void> {
  const snapshot = await snapshotOf(document);
  const graphs = snapshot.diagrams?.graphs ?? [];
  const views = VIEWPOINTS.filter((v) => DIAGRAM_VIEWS.includes(v.view));
  const pick = await vscode.window.showQuickPick(
    views.map((v) => ({ label: v.label, description: v.view.toUpperCase(), view: v.view })),
    { placeHolder: 'New diagram: which kind of view?', title: 'ArcLang: New diagram' },
  );
  if (!pick) return;
  const page = WorkbenchPanel.for(document.uri);
  const selected = page?.selectedId ? (snapshot.elements?.elements ?? []).find((e) => e.id === page.selectedId) : undefined;
  const proposed = uniqueName(selected ? `${selected.name || selected.id} diagram` : `${pick.label} diagram`, graphs.map((g) => g.arc.title));
  const name = await vscode.window.showInputBox({
    prompt: selected ? `Name of the new diagram (it starts from ${selected.name || selected.id})` : 'Name of the new diagram',
    value: proposed,
    validateInput: (value) => (!value.trim() ? 'a name is needed' : graphs.some((g) => g.arc.title === value.trim()) ? 'a diagram of that name exists' : null),
  });
  if (!name) return;
  const title = name.trim();
  const declaration = diagramDeclaration(title, pick.view, selected ? [selected.id] : []);
  if (!(await submit(document, [{ op: 'add_raw', declaration, top_level: true }], `new diagram ${title}`))) return;
  forgetSnapshot(document.uri);
  const made = ((await snapshotOf(document)).diagrams?.graphs ?? []).find((g) => g.arc.view === pick.view && g.arc.title === title);
  if (made) await openDiagramTab(document, made.id);
  else void vscode.window.showWarningMessage(`ArcLang: the diagram "${title}" was written but the compiler draws nothing for it yet (does the model compile?).`);
}

interface DiagramPick extends vscode.QuickPickItem {
  graph?: ViewerGraph;
}

/** The diagrams of a model by layer, the declared ones and the automatic ones. */
export function diagramPicks(graphs: ViewerGraph[]): DiagramPick[] {
  const items: DiagramPick[] = [];
  const known = new Set(VIEWPOINTS.map((v) => v.view));
  const groups = [...VIEWPOINTS.map((v) => ({ label: v.label, view: v.view })), { label: 'Other', view: '' }];
  for (const group of groups) {
    const own = graphs.filter((g) => (group.view ? g.arc.view === group.view : !known.has(g.arc.view)));
    if (own.length === 0) continue;
    items.push({ label: group.label, kind: vscode.QuickPickItemKind.Separator });
    const automatic = own.filter((g) => !g.id.includes(':') || g.id === g.arc.view);
    const declared = own.filter((g) => !automatic.includes(g));
    for (const graph of [...automatic, ...declared]) {
      const isDeclared = declared.includes(graph);
      items.push({
        label: `$(graph) ${graph.arc.title}`,
        description: isDeclared ? 'declared' : 'automatic',
        detail: graph.id,
        graph,
        buttons: [{ iconPath: new vscode.ThemeIcon('link-external'), tooltip: 'Open in a new tab' }],
      });
    }
  }
  return items;
}

/** "Open diagram…": a diagram picked among the layers' diagrams, here or in a tab of its own (the button). */
export async function pickDiagram(document: vscode.TextDocument): Promise<void> {
  const graphs = (await snapshotOf(document)).diagrams?.graphs ?? [];
  if (graphs.length === 0) return void vscode.window.showInformationMessage('ArcLang: this model has no diagram yet (does it compile?). "New layer…" and "New diagram…" add some.');
  const quick = vscode.window.createQuickPick<DiagramPick>();
  quick.title = `ArcLang: open a diagram of ${path.basename(document.uri.fsPath)} (${graphs.length})`;
  quick.placeholder = 'A diagram by layer; the button opens it in a new tab';
  quick.matchOnDescription = true;
  quick.matchOnDetail = true;
  quick.items = diagramPicks(graphs);
  const done = () => quick.dispose();
  quick.onDidTriggerItemButton(async (event) => {
    quick.hide();
    if (event.item.graph) await openDiagramTab(document, event.item.graph.id);
  });
  quick.onDidAccept(async () => {
    const graph = quick.selectedItems[0]?.graph;
    quick.hide();
    if (!graph) return;
    await WorkbenchPanel.open(document);
    WorkbenchPanel.for(document.uri)?.show(graph.id);
  });
  quick.onDidHide(done);
  quick.show();
}

/** "Open model…": a file picked, its text and the workbench side by side. */
export async function openModel(tree: ModelTreeProvider): Promise<void> {
  const picked = await vscode.window.showOpenDialog({ canSelectMany: false, filters: { 'ArcLang or SysML v2': ['arc', 'sysml'] }, openLabel: 'Open model', title: 'ArcLang: open a model' });
  if (!picked?.[0]) return;
  const document = await vscode.workspace.openTextDocument(picked[0]);
  await vscode.window.showTextDocument(document, { preview: false });
  tree.follow(document);
  await WorkbenchPanel.open(document);
}

/** The commands, and the context key the empty tree's welcome view reads. */
export function registerCreation(context: vscode.ExtensionContext, tree: ModelTreeProvider): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  register('arclang.newModel', () => createModel(context, tree));
  register('arclang.openModel', () => openModel(tree));
  register('arclang.newLayer', async (arg?: unknown) => {
    const document = await targetDocument(arg, tree);
    if (document) await createLayer(document);
  });
  register('arclang.newDiagram', async (arg?: unknown) => {
    const document = await targetDocument(arg, tree);
    if (document) await createDiagram(document);
  });
  register('arclang.pickDiagram', async (arg?: unknown) => {
    const document = await targetDocument(arg, tree);
    if (document) await pickDiagram(document);
  });
}
