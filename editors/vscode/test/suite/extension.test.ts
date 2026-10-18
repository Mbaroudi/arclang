// Integration tests in a VS Code instance, on the repository's examples.
// Tests that need the compiler are skipped when neither ARCLANG_BINARY nor
// `arclang` on the PATH answers to --version.
import * as assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import * as path from 'node:path';
import * as vscode from 'vscode';

const EXTENSION_ROOT = path.resolve(__dirname, '../../..');
const EXAMPLE = path.resolve(EXTENSION_ROOT, '../../examples/complete_emergency_braking_simple.arc');

function compilerAvailable(): boolean {
  const binary = process.env.ARCLANG_BINARY || 'arclang';
  return spawnSync(binary, ['--version'], { encoding: 'utf8' }).status === 0;
}

async function waitFor<T>(probe: () => T | undefined, timeoutMs: number): Promise<T> {
  const start = Date.now();
  for (;;) {
    const value = probe();
    if (value !== undefined) return value;
    if (Date.now() - start > timeoutMs) throw new Error('timed out');
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
}

/** The text after applying `edits` (non-overlapping, any order) to `text`. */
function apply(text: string, edits: vscode.TextEdit[]): string {
  const lines = text.split('\n');
  const offset = (position: vscode.Position) => lines.slice(0, position.line).reduce((sum, line) => sum + line.length + 1, 0) + position.character;
  const sorted = [...edits].sort((a, b) => offset(b.range.start) - offset(a.range.start));
  let result = text;
  for (const edit of sorted) result = result.slice(0, offset(edit.range.start)) + edit.newText + result.slice(offset(edit.range.end));
  return result;
}

function textOf(result: vscode.LanguageModelToolResult): string {
  return result.content.map((part) => (part instanceof vscode.LanguageModelTextPart ? part.value : '')).join('');
}

suite('ArcLang extension', () => {
  suiteSetup(async () => {
    if (process.env.ARCLANG_BINARY) {
      await vscode.workspace.getConfiguration('arclang').update('binary', process.env.ARCLANG_BINARY, vscode.ConfigurationTarget.Global);
    }
  });

  test('activates on a .arc file and contributes its commands', async () => {
    const document = await vscode.workspace.openTextDocument(vscode.Uri.file(EXAMPLE));
    assert.equal(document.languageId, 'arclang');
    const extension = vscode.extensions.getExtension('arclang.arclang');
    assert.ok(extension, 'extension found');
    await extension.activate();
    const commands = await vscode.commands.getCommands(true);
    for (const id of ['arclang.openDiagrams', 'arclang.check', 'arclang.runGate', 'arclang.transitionApply', 'arclang.export', 'arclang.import', 'arclang.ai.apply']) {
      assert.ok(commands.includes(id), `${id} registered`);
    }
  });

  test('contributes the compiler tools and the chat participant', () => {
    const names = vscode.lm.tools.map((t) => t.name);
    for (const name of ['arclang_check', 'arclang_gate', 'arclang_metamodel', 'arclang_impact', 'arclang_set_attribute']) {
      assert.ok(names.includes(name), `${name} is a language model tool`);
    }
    const participants = (vscode.extensions.getExtension('arclang.arclang')?.packageJSON.contributes.chatParticipants ?? []) as Array<{ name: string }>;
    assert.ok(participants.some((p) => p.name === 'arclang'));
  });

  test('the check tool answers with the compiler verdict', async function () {
    if (!compilerAvailable()) this.skip();
    const result = await vscode.lm.invokeTool('arclang_check', { input: { source: 'model Probe {\n  version: "1"\n}\n' }, toolInvocationToken: undefined });
    assert.match(textOf(result), /`arclang check --json \(source text\)` succeeded \(exit 0\)/);
  });

  test('the check tool refuses a path outside the workspace', async () => {
    const result = await vscode.lm.invokeTool('arclang_check', { input: { path: '/etc/hosts' }, toolInvocationToken: undefined });
    assert.match(textOf(result), /^Refused/);
  });

  test('the language server reports the findings of the example', async function () {
    if (!compilerAvailable()) this.skip();
    const uri = vscode.Uri.file(EXAMPLE);
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(uri));
    const diagnostics = await waitFor(() => {
      const found = vscode.languages.getDiagnostics(uri).filter((d) => d.source === 'arclang');
      return found.length > 0 ? found : undefined;
    }, 60000);
    assert.ok(diagnostics.some((d) => String(typeof d.code === 'object' && d.code ? d.code.value : d.code).startsWith('trace.')), 'a trace rule finding');
  });

  test('formatting the sample model is a whole-document edit or nothing', async function () {
    if (!compilerAvailable()) this.skip();
    const sample = path.resolve(EXTENSION_ROOT, 'media/reference.arc');
    const document = await vscode.workspace.openTextDocument(vscode.Uri.file(sample));
    const edits = (await vscode.commands.executeCommand<vscode.TextEdit[]>('vscode.executeFormatDocumentProvider', document.uri, { tabSize: 2, insertSpaces: true })) ?? [];
    // Applying the edits must give what `arclang format` prints.
    const expected = spawnSync(process.env.ARCLANG_BINARY || 'arclang', ['format', sample], { encoding: 'utf8' }).stdout;
    const applied = apply(document.getText(), edits);
    assert.equal(applied, expected, `edits: ${JSON.stringify(edits.map((e) => [e.range.start.line, e.range.end.line, e.newText.length]))}`);
  });
});
