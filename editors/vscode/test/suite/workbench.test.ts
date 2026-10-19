// The workbench against the real compiler: the snapshot of a document, an
// edit applied as one undoable change, the model tree and the panel.
import * as assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { applyRequests } from '../../src/workbench/edits';
import { WorkbenchPanel } from '../../src/workbench/panel';
import { metamodelJson, snapshotOf } from '../../src/workbench/snapshot';
import { locateElement, ModelItem, ModelTreeProvider } from '../../src/workbench/tree';

const MODEL = `model "Workbench test" {
  version: "1.0.0"
}

logical_architecture "LA" {
  component "Controller" {
    id: "LC-CTL"
    description: "Controls" // kept
  }
  component "Sensor" {}
}
`;

async function until(condition: () => boolean, ms = 3000): Promise<boolean> {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    if (condition()) return true;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  return condition();
}

async function openCopy(): Promise<vscode.TextDocument> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-wb-'));
  const file = path.join(dir, 'model.arc');
  fs.writeFileSync(file, MODEL);
  const document = await vscode.workspace.openTextDocument(file);
  await vscode.window.showTextDocument(document);
  return document;
}

suite('ArcLang workbench', () => {
  suiteSetup(async () => {
    // The compiler under test, when the environment names one.
    if (process.env.ARCLANG_BINARY) {
      await vscode.workspace.getConfiguration('arclang').update('binary', process.env.ARCLANG_BINARY, vscode.ConfigurationTarget.Global);
    }
  });

  test('the workbench commands and the model view are contributed', async () => {
    const extension = vscode.extensions.getExtension('arclang.arclang');
    assert.ok(extension);
    await extension.activate();
    const commands = await vscode.commands.getCommands(true);
    for (const command of ['arclang.openWorkbench', 'arclang.model.refresh', 'arclang.model.rename', 'arclang.model.delete', 'arclang.model.ask', 'arclang.model.reveal']) {
      assert.ok(commands.includes(command), command);
    }
    const views = (extension.packageJSON as { contributes: { views: Record<string, { id: string }[]> } }).contributes.views;
    assert.ok(views.arclang.some((v) => v.id === 'arclangModel'));
  });

  test('a snapshot lists the elements with the ids the compiler gives them', async () => {
    const document = await openCopy();
    const snapshot = await snapshotOf(document);
    assert.equal(snapshot.success, true, snapshot.errors.join('\n'));
    const ids = (snapshot.elements?.elements ?? []).map((e: { id: string }) => e.id);
    assert.ok(ids.includes('LC-CTL'), ids.join(','));
    assert.ok(ids.includes('LC-Sensor'), ids.join(','));
    assert.ok((snapshot.diagrams?.graphs.length ?? 0) >= 1);
  });

  test('an edit from the views lands in the editor as one undoable change', async () => {
    const document = await openCopy();
    const versionBefore = document.version;
    const outcome = await applyRequests(document, [
      { op: 'add', keyword: 'function', name: 'Compute', container: 'LC-CTL' },
      { op: 'set', element: 'LC-Sensor', key: 'description', value: '"Radar"' },
    ]);
    assert.equal(outcome.ok, true, outcome.ok ? '' : outcome.error);
    const text = document.getText();
    assert.ok(text.includes('function "Compute"'), text);
    assert.ok(text.includes('description: "Radar"'), text);
    assert.ok(text.includes('description: "Controls" // kept'), text);
    // Two requests, one change of the document.
    assert.equal(document.version, versionBefore + 1, 'the gesture is one edit of the document');
    // Undo acts on the focused editor; the test host does not always give the
    // editor widget the focus, so the undo itself is checked only when it ran.
    await vscode.window.showTextDocument(document, { preserveFocus: false });
    await vscode.commands.executeCommand('undo');
    if (await until(() => document.version > versionBefore + 1, 1500)) {
      assert.ok(!document.getText().includes('Compute'), 'one undo takes the whole gesture back');
      assert.equal(document.getText(), MODEL);
    }
  });

  test('a refused edit changes nothing and says why', async () => {
    const document = await openCopy();
    const before = document.getText();
    const outcome = await applyRequests(document, [{ op: 'set', element: 'Nobody', key: 'description', value: '"x"' }]);
    assert.equal(outcome.ok, false);
    assert.ok(!outcome.ok && /Nobody/.test(outcome.error), outcome.ok ? '' : outcome.error);
    assert.equal(document.getText(), before);
  });

  test('the model tree groups the elements by layer and reveals a declaration', async () => {
    const document = await openCopy();
    const metamodel = await metamodelJson();
    assert.ok(metamodel && metamodel.kinds.length > 10, `metamodel: ${JSON.stringify(metamodel).slice(0, 200)}`);
    const context = { subscriptions: [] } as unknown as vscode.ExtensionContext;
    const tree = new ModelTreeProvider(context);
    tree.follow(document);
    const layers = await tree.getChildren();
    const logical = layers.find((item: ModelItem) => item.type === 'layer' && item.layer === 'Logical');
    assert.ok(layers[0].type === 'diagrams', 'the diagrams head the tree');
    assert.ok(layers.some((item: ModelItem) => item.type === 'layer' && item.layer === 'Model'), 'the model has its own entry');
    assert.ok(logical, JSON.stringify(layers.map((l: ModelItem) => (l.type === 'layer' ? l.layer : l.type))));
    const components = await tree.getChildren(logical);
    assert.deepEqual(
      components.map((c: ModelItem) => (c.type === 'element' ? c.node.element.id : c.type)),
      ['LC-CTL', 'LC-Sensor'],
    );
    const range = await locateElement(document, { id: 'LC-Sensor', name: 'Sensor' });
    assert.ok(range);
    assert.ok(document.getText(document.lineAt(range.start.line).range).includes('Sensor'));
  });

  test('the model tool lists the elements and the edit tool applies a list of requests', async () => {
    const document = await openCopy();
    const read = await vscode.lm.invokeTool('arclang_model', { input: { path: document.uri.fsPath, layer: 'Logical' }, toolInvocationToken: undefined });
    const text = read.content.map((part) => (part instanceof vscode.LanguageModelTextPart ? part.value : '')).join('\n');
    assert.ok(text.includes('LC-CTL | LogicalComponent'), text);
    assert.ok(text.includes('LC-Sensor | LogicalComponent'), text);
    const edited = await vscode.lm.invokeTool('arclang_edit', {
      input: { path: document.uri.fsPath, requests: [{ op: 'add', keyword: 'function', name: 'Sense', container: 'LC-Sensor' }] },
      toolInvocationToken: undefined,
    });
    const answer = edited.content.map((part) => (part instanceof vscode.LanguageModelTextPart ? part.value : '')).join('\n');
    assert.ok(/exit 0/.test(answer), answer);
    assert.ok(document.getText().includes('function "Sense"'), document.getText());
    const refused = await vscode.lm.invokeTool('arclang_edit', {
      input: { path: document.uri.fsPath, requests: [{ op: 'remove', element: 'Nobody' }] },
      toolInvocationToken: undefined,
    });
    const why = refused.content.map((part) => (part instanceof vscode.LanguageModelTextPart ? part.value : '')).join('\n');
    assert.ok(/Refused at request 1/.test(why) && /Nobody/.test(why), why);
  });

  test('a SysML v2 file goes through the same workbench: imported for the views, exported after an edit', async () => {
    const binary = process.env.ARCLANG_BINARY || 'arclang';
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-wb-sysml-'));
    const arc = path.join(dir, 'model.arc');
    fs.writeFileSync(arc, MODEL);
    const sysml = path.join(dir, 'model.sysml');
    const exported = spawnSync(binary, ['export', '-f', 'sys-ml', arc, '-o', sysml], { encoding: 'utf8' });
    assert.equal(exported.status, 0, exported.stderr);
    const document = await vscode.workspace.openTextDocument(sysml);
    await vscode.window.showTextDocument(document);
    assert.equal(document.languageId, 'sysml');
    const snapshot = await snapshotOf(document);
    assert.equal(snapshot.success, true, snapshot.errors.join('\n'));
    const ids = (snapshot.elements?.elements ?? []).map((e: { id: string }) => e.id);
    assert.ok(ids.includes('LC-CTL'), ids.join(','));
    const outcome = await applyRequests(document, [{ op: 'set', element: 'LC-CTL', key: 'description', value: '"Controls the brakes"' }]);
    assert.equal(outcome.ok, true, outcome.ok ? '' : outcome.error);
    assert.ok(document.getText().includes('Controls the brakes'), document.getText());
    assert.ok(document.getText().includes("<'LC-CTL'>"), 'the SysML text keeps the ids');
    const again = await snapshotOf(document);
    const controller = (again.elements?.elements ?? []).find((e: { id: string }) => e.id === 'LC-CTL') as { attributes: Record<string, unknown> } | undefined;
    assert.equal(controller?.attributes.description, 'Controls the brakes');
  });

  test('the workbench panel opens beside the source', async () => {
    const document = await openCopy();
    await vscode.commands.executeCommand('arclang.openWorkbench', document.uri);
    // The extension runs from its bundle, so its panel registry is not this
    // module's: the workbench shows as a custom editor tab on the document.
    const customTabs = () =>
      vscode.window.tabGroups.all.flatMap((group) =>
        group.tabs.filter((tab) => tab.input instanceof vscode.TabInputCustom && tab.input.viewType === 'arclang.workbench' && tab.input.uri.toString() === document.uri.toString()),
      );
    assert.ok(await until(() => customTabs().length > 0), vscode.window.tabGroups.all.flatMap((g) => g.tabs.map((t) => t.label)).join(' | '));
    void WorkbenchPanel;
  });
});
