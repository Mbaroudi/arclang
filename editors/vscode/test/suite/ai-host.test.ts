// The AI side of the host against the fake provider (test/ai/fake-llm-server.js):
// a refused consent, a whole-text proposal on a changed document, the
// sessions of a page that closed. No network, no key.
import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { applySession, runAsk, sessionCount } from '../../src/ai/assist';
import { bridgeSizes, connectBridge, documentClosed, handlePageMessage, pageClosed } from '../../src/ai/bridge';
import type { ToPageAi } from '../../src/ai/core/protocol';
import { resolveConfig } from '../../src/ai/providers';
import { ensureConsent } from '../../src/ai/settings';
import type { SuggestionEngine } from '../../src/ai/suggest';
import type { FakeReply, FakeRequest, FakeServer } from '../ai/fake-llm-server';

const { startFakeServer } = require(path.resolve(__dirname, '../../../test/ai/fake-llm-server.js')) as typeof import('../ai/fake-llm-server');

const MODEL = `model "Host test" {
  version: "1.0.0"
}

logical_architecture "LA" {
  component "Controller" {
    id: "LC-CTL"
  }
}
`;

const fakeContext = {
  subscriptions: [] as vscode.Disposable[],
  secrets: { get: async () => undefined },
  globalState: { get: <T>(_key: string, fallback: T) => fallback, update: async () => undefined },
} as unknown as vscode.ExtensionContext;

async function openCopy(): Promise<vscode.TextDocument> {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-ai-'));
  const file = path.join(dir, 'model.arc');
  fs.writeFileSync(file, MODEL);
  const document = await vscode.workspace.openTextDocument(file);
  await vscode.window.showTextDocument(document);
  return document;
}

async function until(condition: () => boolean, ms = 15000): Promise<boolean> {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    if (condition()) return true;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  return condition();
}

suite('ArcLang AI in the host', () => {
  let server: FakeServer;
  let reply: (request: FakeRequest) => FakeReply = () => ({ text: 'OK' });

  suiteSetup(async () => {
    server = await startFakeServer((request) => reply(request));
    const ai = vscode.workspace.getConfiguration('arclang.ai');
    await ai.update('provider', 'openai-compatible', vscode.ConfigurationTarget.Global);
    await ai.update('endpoint', `${server.url}/v1`, vscode.ConfigurationTarget.Global);
    await ai.update('model', 'fake-model', vscode.ConfigurationTarget.Global);
    if (process.env.ARCLANG_BINARY) await vscode.workspace.getConfiguration('arclang').update('binary', process.env.ARCLANG_BINARY, vscode.ConfigurationTarget.Global);
  });

  suiteTeardown(async () => {
    const ai = vscode.workspace.getConfiguration('arclang.ai');
    for (const key of ['provider', 'endpoint', 'model']) await ai.update(key, undefined, vscode.ConfigurationTarget.Global);
    await server.close();
  });

  test('a refused consent is not asked again by the suggestions after each edit, but is by a request', async () => {
    const window = vscode.window as unknown as { showWarningMessage: (...args: unknown[]) => Thenable<unknown> };
    const original = window.showWarningMessage;
    let asked = 0;
    window.showWarningMessage = async () => {
      asked += 1;
      return undefined; // the modal closed: refused
    };
    try {
      const cloud = resolveConfig({ provider: 'anthropic', model: '', endpoint: 'https://api.anthropic.example', azureDeployment: '', apiVersion: '', maxTokens: 1024, temperature: null, timeoutSeconds: 10 }, 'fake-key-not-real');
      assert.equal(await ensureConsent(fakeContext, cloud, 'x', { automatic: true }), false);
      assert.equal(await ensureConsent(fakeContext, cloud, 'x', { automatic: true }), false);
      assert.equal(await ensureConsent(fakeContext, cloud, 'x', { automatic: true }), false);
      assert.equal(asked, 1, 'asked once, then remembered for the session');
      assert.equal(await ensureConsent(fakeContext, cloud, 'x'), false);
      assert.equal(asked, 2, 'a request the user makes asks again');
    } finally {
      window.showWarningMessage = original;
    }
  });

  test('a whole-text proposal is not applied on a document changed since', async () => {
    const proposedModel = MODEL.replace('component "Controller" {', 'component "Controller" {\n    description: "Plans"');
    reply = () => ({ text: `Here is the model.\n\n\`\`\`arclang\n${proposedModel}\`\`\`\n` });
    const document = await openCopy();
    const asked = await runAsk('whole-1', { document, ids: ['LC-CTL'], prompt: 'Describe it' }, fakeContext, () => undefined);
    assert.ok(asked.ok, asked.ok ? '' : asked.error);
    assert.ok(asked.session.proposal?.source, 'a whole text was proposed');
    const edit = new vscode.WorkspaceEdit();
    edit.insert(document.uri, new vscode.Position(0, 0), '// typed meanwhile\n');
    await vscode.workspace.applyEdit(edit);
    const changed = document.getText();
    const applied = await applySession('whole-1');
    assert.equal(applied.ok, false);
    assert.match(applied.error ?? '', /changed since/);
    assert.equal(document.getText(), changed, 'what was typed is kept');
    const again = await runAsk('whole-2', { document, ids: ['LC-CTL'], prompt: 'Describe it' }, fakeContext, () => undefined);
    assert.ok(again.ok && again.session.proposal?.source);
    assert.equal((await applySession('whole-2')).ok, true, 'on the text it was computed on, it applies');
    assert.match(document.getText(), /description: "Plans"/);
  });

  test('the sessions of a page go when the page closes, and those of a document when it closes', async () => {
    connectBridge(fakeContext, { replay: () => undefined } as unknown as SuggestionEngine);
    reply = () => ({ text: 'An explanation only.' });
    const document = await openCopy();
    const received: ToPageAi[] = [];
    const post = (message: ToPageAi) => {
      received.push(message);
    };
    const before = bridgeSizes();
    assert.equal(handlePageMessage(document, post, { type: 'ai-hello' }), true);
    assert.equal(handlePageMessage(document, post, { type: 'ai-ask', session: 'page-1', ids: ['LC-CTL'], prompt: 'Explain' }), true);
    assert.ok(await until(() => received.some((m) => m.type === 'ai-proposal' || m.type === 'ai-error')), JSON.stringify(received.slice(-2)));
    assert.equal(bridgeSizes().pages, before.pages + 1);
    assert.equal(bridgeSizes().sessions, before.sessions + 1);
    const open = sessionCount();
    pageClosed(document, post);
    assert.deepEqual(bridgeSizes(), before, 'the page and its session are forgotten');
    assert.equal(sessionCount(), open - 1);
    // A malformed message is consumed and dropped, not handed to the workbench.
    assert.equal(handlePageMessage(document, post, { type: 'ai-ask', session: 'page-2', ids: 'LC-CTL', prompt: 3 }), true);
    assert.equal(handlePageMessage(document, post, { type: 'edit' }), false);
    // A document that closes takes its sessions along.
    assert.ok((await runAsk('doc-1', { document, ids: ['LC-CTL'], prompt: 'Explain' }, fakeContext, () => undefined)).ok);
    const withOne = sessionCount();
    documentClosed(document);
    assert.equal(sessionCount(), withOne - 1);
  });
});
