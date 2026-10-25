// Navigation between the diagram and the text, in one place.
//
// A click on the diagram only selects: the text follows by a highlight when
// it is already on screen (`arclang.workbench.followSelection`), never by
// opening, replacing or focusing an editor, and never by moving the cursor
// or the scroll position. A double-click, the Source button, "Go to source"
// and the tree go to the source: the declaration selected in the text editor,
// opened beside the workbench when it is not on screen, never over its tab.
import * as vscode from 'vscode';
import { elementLabel, ElementRecord } from '../pure/workbench';
import { locateElement, shownTextEditor } from './tree';

/** The columns of the workbench pages of a document (set by the panel module, which owns them). */
let columnsOf: (uri: vscode.Uri) => (vscode.ViewColumn | undefined)[] = () => [];
export function setWorkbenchColumns(fn: typeof columnsOf): void {
  columnsOf = fn;
}

export type FollowSelection = 'highlight' | 'off';
export type DoubleClick = 'source' | 'rename';

export function followSelectionSetting(): FollowSelection {
  return vscode.workspace.getConfiguration('arclang.workbench').get<string>('followSelection') === 'off' ? 'off' : 'highlight';
}

export function doubleClickSetting(): DoubleClick {
  return vscode.workspace.getConfiguration('arclang.workbench').get<string>('doubleClick') === 'rename' ? 'rename' : 'source';
}

/** The editor group (its column) that holds a workbench page of the document, else undefined. */
function workbenchColumn(document: vscode.TextDocument): vscode.ViewColumn | undefined {
  return columnsOf(document.uri).find((column) => column !== undefined);
}

/**
 * The column to open the text in: where it is already visible; else a group
 * other than the workbench's (a new one beside when the workbench is alone);
 * else the active group when no workbench is open.
 */
export function sourceColumn(document: vscode.TextDocument): vscode.ViewColumn {
  const shown = shownTextEditor(document)?.viewColumn;
  if (shown !== undefined) return shown;
  const workbench = workbenchColumn(document);
  if (workbench === undefined) return vscode.ViewColumn.Active;
  const other = vscode.window.tabGroups.all.find((group) => group.viewColumn !== workbench);
  // The group after the workbench's: a column that does not exist yet is a new group beside it.
  return other ? other.viewColumn : ((workbench + 1) as vscode.ViewColumn);
}

export interface GoToSource {
  /** Keep the keyboard focus where it is (default: the text takes it). */
  preserveFocus?: boolean;
  /** Open the text when it is not on screen (default true). */
  open?: boolean;
}

/** Show the element's declaration, selected, in the text editor; see the file header for where. */
export async function goToSource(document: vscode.TextDocument, element: Pick<ElementRecord, 'id' | 'name'>, options: GoToSource = {}): Promise<boolean> {
  const range = await locateElement(document, element);
  if (!range) {
    vscode.window.setStatusBarMessage(`ArcLang: ${elementLabel({ ...element, kind: '' })} is not declared in this file (it may come from an import)`, 4000);
    return false;
  }
  const visible = shownTextEditor(document);
  if (!visible && options.open === false) return false;
  const editor = await vscode.window.showTextDocument(document, { viewColumn: visible?.viewColumn ?? sourceColumn(document), preserveFocus: options.preserveFocus ?? false, preview: false });
  editor.selection = new vscode.Selection(range.start, range.end);
  editor.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
  return true;
}

/** The text itself, in the column the source goes to, without moving its cursor. */
export async function openSource(document: vscode.TextDocument, options: GoToSource = {}): Promise<void> {
  const visible = shownTextEditor(document);
  await vscode.window.showTextDocument(document, { viewColumn: visible?.viewColumn ?? sourceColumn(document), preserveFocus: options.preserveFocus ?? false, preview: false });
}

// --- the highlight ------------------------------------------------------------

let decoration: vscode.TextEditorDecorationType | null = null;
let decorated: { editor: vscode.TextEditor; range: vscode.Range } | null = null;

function decorationType(): vscode.TextEditorDecorationType {
  decoration ??= vscode.window.createTextEditorDecorationType({
    isWholeLine: false,
    backgroundColor: new vscode.ThemeColor('editor.findMatchHighlightBackground'),
    borderRadius: '2px',
    overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.findMatchForeground'),
    overviewRulerLane: vscode.OverviewRulerLane.Center,
  });
  return decoration;
}

/** Take the highlight away. */
export function clearHighlight(): void {
  if (decorated) decorated.editor.setDecorations(decorationType(), []);
  decorated = null;
}

/**
 * Follow the selection of the diagram in the text without touching it: the
 * declaration decorated, scrolled into view only when it is out of the
 * viewport. Nothing is opened, focused or selected.
 */
export async function highlightElement(document: vscode.TextDocument, element: Pick<ElementRecord, 'id' | 'name'> | null): Promise<void> {
  clearHighlight();
  if (!element || followSelectionSetting() === 'off') return;
  const editor = shownTextEditor(document);
  if (!editor) return;
  const range = await locateElement(document, element);
  if (!range) return;
  // The editor may have gone while the range was looked up.
  const still = shownTextEditor(document);
  if (!still) return;
  clearHighlight();
  still.setDecorations(decorationType(), [range]);
  decorated = { editor: still, range };
  if (!still.visibleRanges.some((visible) => visible.contains(range.start))) still.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
}

/** What the highlight is on, for the tests. */
export function highlightState(): { uri: string; startLine: number; endLine: number } | null {
  return decorated ? { uri: decorated.editor.document.uri.toString(), startLine: decorated.range.start.line, endLine: decorated.range.end.line } : null;
}

export function disposeHighlight(): void {
  clearHighlight();
  decoration?.dispose();
  decoration = null;
}
