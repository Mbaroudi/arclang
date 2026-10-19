// The agent loop end to end, with a scripted language model instead of
// Copilot: the model asks for the elements, edits the open model through the
// compiler, checks it, and concludes. Everything the loop does with the
// tools is real; only the model's choices are scripted.
import * as assert from 'node:assert/strict';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { converse } from '../../src/ai/participant';

const MODEL = `model "Agent test" {
  version: "1.0.0"
}

logical_architecture "LA" {
  component "Controller" {
    id: "LC-CTL"
    description: "Controls" // kept
  }
  component "Sensor" {}
}

requirements system {
  req "REQ-1" "Shall sense" {
    description: "The system shall sense the lane."
  }
}
`;

type Part = vscode.LanguageModelTextPart | vscode.LanguageModelToolCallPart;

/** A language model that answers each round with a scripted part, and records what it was told. */
function scriptedModel(script: (round: number, messages: vscode.LanguageModelChatMessage[]) => Part[]): { model: vscode.LanguageModelChat; rounds: () => number } {
  let round = 0;
  const model = {
    id: 'scripted',
    vendor: 'test',
    family: 'scripted',
    version: '1',
    name: 'scripted model',
    maxInputTokens: 100000,
    countTokens: async () => 0,
    sendRequest: async (messages: vscode.LanguageModelChatMessage[]) => {
      const parts = script(round++, messages);
      return {
        stream: (async function* () {
          for (const part of parts) yield part;
        })(),
        text: (async function* () {
          for (const part of parts) if (part instanceof vscode.LanguageModelTextPart) yield part.value;
        })(),
      };
    },
  };
  return { model: model as unknown as vscode.LanguageModelChat, rounds: () => round };
}

/** A response stream that keeps what the user would have seen. */
function recordingStream(): { stream: vscode.ChatResponseStream; markdown: string[]; progress: string[] } {
  const markdown: string[] = [];
  const progress: string[] = [];
  const stream = {
    markdown: (value: string | vscode.MarkdownString) => markdown.push(typeof value === 'string' ? value : value.value),
    progress: (value: string) => progress.push(value),
    anchor: () => undefined,
    button: () => undefined,
    filetree: () => undefined,
    reference: () => undefined,
    push: () => undefined,
  };
  return { stream: stream as unknown as vscode.ChatResponseStream, markdown, progress };
}

/** The text of the last tool result the model received. */
function lastToolResultText(messages: vscode.LanguageModelChatMessage[]): string {
  const last = messages[messages.length - 1];
  return last.content
    .flatMap((part) => (part instanceof vscode.LanguageModelToolResultPart ? part.content : []))
    .map((part) => (part instanceof vscode.LanguageModelTextPart ? part.value : ''))
    .join('\n');
}

suite('ArcLang agent', () => {
  suiteSetup(async () => {
    if (process.env.ARCLANG_BINARY) {
      await vscode.workspace.getConfiguration('arclang').update('binary', process.env.ARCLANG_BINARY, vscode.ConfigurationTarget.Global);
    }
    await vscode.extensions.getExtension('arclang.arclang')!.activate();
  });

  test('the agent reads the model, edits it through the compiler, checks it and concludes', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-agent-'));
    const file = path.join(dir, 'model.arc');
    fs.writeFileSync(file, MODEL);
    const document = await vscode.workspace.openTextDocument(file);
    await vscode.window.showTextDocument(document);

    const seen: Record<string, string> = {};
    const { model, rounds } = scriptedModel((round, messages) => {
      switch (round) {
        case 0:
          return [new vscode.LanguageModelToolCallPart('c1', 'arclang_model', { path: file, layer: 'Logical' })];
        case 1:
          seen.model = lastToolResultText(messages);
          return [
            new vscode.LanguageModelTextPart('I add a function to the sensor and trace the controller to the requirement.'),
            new vscode.LanguageModelToolCallPart('c2', 'arclang_edit', {
              path: file,
              requests: [
                { op: 'add', keyword: 'function', name: 'Sense lane', container: 'LC-Sensor' },
                { op: 'set', element: 'LC-Sensor', key: 'description', value: '"Lane camera"' },
                { op: 'add_trace', from: 'LC-CTL', kind: 'satisfies', to: 'REQ-1' },
              ],
            }),
          ];
        case 2:
          seen.edit = lastToolResultText(messages);
          return [new vscode.LanguageModelToolCallPart('c3', 'arclang_check', { path: file })];
        case 3:
          seen.check = lastToolResultText(messages);
          return [new vscode.LanguageModelTextPart('Done: the sensor has a function, the controller satisfies REQ-1, the model compiles.')];
        default:
          return [new vscode.LanguageModelTextPart('unexpected round')];
      }
    });

    const { stream, markdown, progress } = recordingStream();
    const tools = vscode.lm.tools.filter((t) => t.name.startsWith('arclang_')).map((t) => ({ name: t.name, description: t.description, inputSchema: t.inputSchema }));
    const messages = [vscode.LanguageModelChatMessage.User('Add a function to the sensor and trace the controller to REQ-1.')];
    const answer = await converse(model, messages, tools, stream, new vscode.CancellationTokenSource().token, undefined as unknown as vscode.ChatParticipantToolToken);

    assert.equal(rounds(), 4, 'four rounds: model, edit, check, conclusion');
    assert.ok(/LC-CTL \| LogicalComponent/.test(seen.model), `the model tool listed the elements: ${seen.model}`);
    assert.ok(/LC-Sensor \| LogicalComponent/.test(seen.model), 'ids derived from names are listed');
    assert.ok(/exit 0/.test(seen.edit), `the edit was applied: ${seen.edit}`);
    assert.ok(/exit 0/.test(seen.check), `the check ran: ${seen.check}`);
    const text = document.getText();
    assert.ok(text.includes('function "Sense lane"'), text);
    assert.ok(text.includes('description: "Lane camera"'), text);
    assert.ok(text.includes('trace "LC-CTL" satisfies "REQ-1"'), text);
    assert.ok(text.includes('description: "Controls" // kept'), 'comments and other declarations untouched');
    assert.ok(answer.startsWith('I add a function'), answer);
    assert.ok(answer.endsWith('the model compiles.'), answer);
    assert.deepEqual(progress, ['Running arclang_model', 'Running arclang_edit', 'Running arclang_check']);
    assert.ok(markdown.join('').includes('Done:'));
  });

  test('a refused edit comes back to the agent with the reason, and nothing changes', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-agent-'));
    const file = path.join(dir, 'model.arc');
    fs.writeFileSync(file, MODEL);
    const document = await vscode.workspace.openTextDocument(file);
    const seen: Record<string, string> = {};
    const { model } = scriptedModel((round, messages) => {
      if (round === 0) {
        return [new vscode.LanguageModelToolCallPart('c1', 'arclang_edit', { path: file, requests: [{ op: 'set', element: 'LC-CTL', key: 'description', value: '"ok"' }, { op: 'remove', element: 'Nobody' }] })];
      }
      seen.edit = lastToolResultText(messages);
      return [new vscode.LanguageModelTextPart('The compiler refused the second request; nothing was changed.')];
    });
    const { stream } = recordingStream();
    const tools = vscode.lm.tools.filter((t) => t.name.startsWith('arclang_')).map((t) => ({ name: t.name, description: t.description, inputSchema: t.inputSchema }));
    const answer = await converse(model, [vscode.LanguageModelChatMessage.User('x')], tools, stream, new vscode.CancellationTokenSource().token, undefined as unknown as vscode.ChatParticipantToolToken);
    assert.ok(/Refused at request 2/.test(seen.edit) && /Nobody/.test(seen.edit), seen.edit);
    assert.equal(document.getText(), MODEL, 'the whole list is refused: the first request is not applied either');
    assert.ok(answer.includes('refused'));
  });

  test('the loop stops after eight rounds of tool calls', async () => {
    const { model, rounds } = scriptedModel(() => [new vscode.LanguageModelToolCallPart('x', 'arclang_rules', {})]);
    const { stream } = recordingStream();
    await converse(model, [vscode.LanguageModelChatMessage.User('x')], [], stream, new vscode.CancellationTokenSource().token, undefined as unknown as vscode.ChatParticipantToolToken);
    assert.equal(rounds(), 8);
  });
});
