// The relations view: what the selected element is connected to, as a
// native tree (Capella's semantic browser): owner, incoming, outgoing,
// owned and traces, each entry a step to that element.
import * as vscode from 'vscode';
import { ElementRecord, neighbors, Snapshot } from '../pure/workbench';
import { onDidChangeSnapshot, snapshotOf } from './snapshot';

export type RelationItem =
  | { type: 'group'; label: string; icon: string; entries: RelationEntry[] }
  | { type: 'entry'; entry: RelationEntry }
  | { type: 'message'; text: string };

export interface RelationEntry {
  kind: string;
  name: string;
  other: ElementRecord | null;
  uri: vscode.Uri;
}

export class RelationsProvider implements vscode.TreeDataProvider<RelationItem> {
  private readonly changed = new vscode.EventEmitter<RelationItem | undefined>();
  readonly onDidChangeTreeData = this.changed.event;
  private document: vscode.TextDocument | null = null;
  private element: ElementRecord | null = null;

  constructor(context: vscode.ExtensionContext) {
    context.subscriptions.push(
      onDidChangeSnapshot((uri) => {
        if (this.document && uri.toString() === this.document.uri.toString()) void this.refreshElement();
      }),
    );
  }

  /** Show the relations of an element of a document (null clears the view). */
  show(document: vscode.TextDocument | null, element: ElementRecord | null): void {
    this.document = document;
    this.element = element;
    this.changed.fire(undefined);
  }

  get shown(): { document: vscode.TextDocument; element: ElementRecord } | null {
    return this.document && this.element ? { document: this.document, element: this.element } : null;
  }

  /** After an edit, the same element (by uuid) with its new attributes and links. */
  private async refreshElement(): Promise<void> {
    if (!this.document || !this.element) return;
    const snapshot = await snapshotOf(this.document);
    const again = (snapshot.elements?.elements ?? []).find((e) => e.uuid === this.element!.uuid) ?? null;
    this.element = again;
    this.changed.fire(undefined);
  }

  getTreeItem(item: RelationItem): vscode.TreeItem {
    if (item.type === 'message') {
      const tree = new vscode.TreeItem(item.text, vscode.TreeItemCollapsibleState.None);
      tree.iconPath = new vscode.ThemeIcon('info');
      return tree;
    }
    if (item.type === 'group') {
      const tree = new vscode.TreeItem(item.label, item.entries.length ? vscode.TreeItemCollapsibleState.Expanded : vscode.TreeItemCollapsibleState.None);
      tree.iconPath = new vscode.ThemeIcon(item.icon);
      tree.description = `${item.entries.length}`;
      return tree;
    }
    const { entry } = item;
    const other = entry.other;
    const tree = new vscode.TreeItem(other ? other.name || other.id : '?', vscode.TreeItemCollapsibleState.None);
    tree.description = entry.name ? `${entry.kind} ${entry.name}` : entry.kind;
    tree.tooltip = other ? `${other.kind} ${other.id}` : entry.kind;
    tree.iconPath = new vscode.ThemeIcon(other ? 'symbol-misc' : 'question');
    tree.contextValue = 'arclang.relation';
    if (other) tree.command = { command: 'arclang.relations.step', title: 'Go to', arguments: [item] };
    return tree;
  }

  async getChildren(item?: RelationItem): Promise<RelationItem[]> {
    if (!this.document || !this.element) return [{ type: 'message', text: 'Select an element to see what it is connected to.' }];
    if (item?.type === 'group') return item.entries.map((entry) => ({ type: 'entry' as const, entry }));
    if (item) return [];
    const snapshot: Snapshot = await snapshotOf(this.document);
    const records = snapshot.elements?.elements ?? [];
    const element = records.find((e) => e.uuid === this.element!.uuid) ?? this.element;
    const uri = this.document.uri;
    const { incoming, outgoing } = neighbors(element, snapshot);
    const owner = records.find((e) => e.uuid === element.owner) ?? null;
    const owned = records.filter((e) => e.owner === element.uuid);
    const entry = (kind: string, name: string, other: ElementRecord | null): RelationEntry => ({ kind, name, other, uri });
    const traces = (kind: string) => !/Exchange|Link|Deployment|Allocation|Involvement|Realization|Message|Transition/.test(kind);
    return [
      { type: 'group', label: 'Owner', icon: 'arrow-up', entries: owner ? [entry('owns', '', owner)] : [] },
      { type: 'group', label: 'Incoming', icon: 'arrow-left', entries: incoming.filter((n) => !traces(n.kind) || n.name).map((n) => entry(n.kind, n.name, n.other)) },
      { type: 'group', label: 'Outgoing', icon: 'arrow-right', entries: outgoing.filter((n) => !traces(n.kind) || n.name).map((n) => entry(n.kind, n.name, n.other)) },
      { type: 'group', label: 'Traces', icon: 'link', entries: [...incoming, ...outgoing].filter((n) => traces(n.kind) && !n.name).map((n) => entry(n.kind, '', n.other)) },
      { type: 'group', label: 'Owned', icon: 'arrow-down', entries: owned.map((o) => entry(o.kind, '', o)) },
    ];
  }
}
