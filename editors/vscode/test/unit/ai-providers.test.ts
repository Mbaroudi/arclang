// The providers against a local fake server that speaks each wire format:
// streaming, tool calls, the keys in the right header and never in an
// error, timeouts and cancellation. No network, no real key.
import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { createHttpProvider, displayUrl, endpointProblem, isLocalEndpoint, missingPiece, needsConsent, redact, resolveConfig, AiSettings, ChatEvent, ChatMessage, ProviderConfig, ProviderId, ToolSpec } from '../../src/ai/providers';
import { sseEvents, ndjsonLines, MAX_EVENT_CHARS, MAX_RESPONSE_BYTES, ProviderError } from '../../src/ai/providers/http';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { converse } from '../../src/ai/core/conversation';
import { FakeReply, FakeRequest, FakeServer, startFakeServer } from '../ai/fake-llm-server';

const KEY = 'sk-test-THIS-IS-A-FAKE-KEY-123456';
let server: FakeServer;
let reply: (request: FakeRequest) => FakeReply = () => ({ text: 'OK' });

before(async () => {
  server = await startFakeServer((request) => reply(request));
});
after(async () => {
  await server.close();
});

const settings = (provider: string, extra: Partial<AiSettings> = {}): AiSettings => ({ provider, model: '', endpoint: '', azureDeployment: '', apiVersion: '', maxTokens: 1024, temperature: null, timeoutSeconds: 10, ...extra });

/** The endpoint of the fake server for each provider, as a user would write it. */
function configFor(provider: ProviderId, extra: Partial<AiSettings> = {}): ProviderConfig {
  const endpoints: Partial<Record<ProviderId, string>> = {
    openai: `${server.url}/v1`,
    'openai-compatible': `${server.url}/v1`,
    mistral: `${server.url}/v1`,
    'azure-openai': server.url,
    anthropic: server.url,
    google: `${server.url}/v1beta`,
    ollama: server.url,
  };
  return resolveConfig(settings(provider, { endpoint: endpoints[provider], azureDeployment: provider === 'azure-openai' ? 'arc-gpt' : '', model: provider === 'openai-compatible' ? 'local-model' : '', ...extra }), KEY);
}

async function collect(stream: AsyncIterable<ChatEvent>): Promise<{ text: string; calls: { name: string; input: Record<string, unknown> }[]; done: ChatEvent | undefined }> {
  let text = '';
  const calls: { name: string; input: Record<string, unknown> }[] = [];
  let done: ChatEvent | undefined;
  for await (const event of stream) {
    if (event.type === 'text') text += event.text;
    if (event.type === 'tool_call') calls.push({ name: event.call.name, input: event.call.input });
    if (event.type === 'done') done = event;
  }
  return { text, calls, done };
}

const TOOL: ToolSpec = { name: 'propose_edits', description: 'Propose', parameters: { type: 'object', properties: { explanation: { type: 'string' }, requests: { type: 'array', items: { type: 'object', additionalProperties: true } } }, required: ['explanation'] } };
const MESSAGES: ChatMessage[] = [
  { role: 'system', content: 'You are a test.' },
  { role: 'user', content: 'Add a monitor' },
];
const PROPOSAL = { explanation: 'Adds "LC-MON" with ports {a, b}', requests: [{ op: 'add', keyword: 'component', name: 'Health Monitor', layer: 'logical_architecture' }] };

const ALL: ProviderId[] = ['openai', 'azure-openai', 'mistral', 'openai-compatible', 'anthropic', 'google', 'ollama'];

for (const provider of ALL) {
  test(`${provider}: streams text reassembled from small chunks`, async () => {
    reply = () => ({ text: 'Le moniteur surveille la caméra — 80 ms, "quoted" {braces}\nline two' });
    const out = await collect(createHttpProvider(configFor(provider)).chat(MESSAGES));
    assert.equal(out.text, 'Le moniteur surveille la caméra — 80 ms, "quoted" {braces}\nline two');
    assert.equal(out.done?.type, 'done');
  });

  test(`${provider}: a tool call arrives whole, with its arguments as an object`, async () => {
    reply = () => ({ text: 'Proposing.', toolCalls: [{ name: 'propose_edits', input: PROPOSAL }] });
    const out = await collect(createHttpProvider(configFor(provider)).chat(MESSAGES, { tools: [TOOL] }));
    assert.deepEqual(out.calls, [{ name: 'propose_edits', input: PROPOSAL }]);
    const request = server.requests[server.requests.length - 1];
    const sent = JSON.stringify(request.body);
    assert.match(sent, /propose_edits/, 'the tool is declared in the request');
  });

  test(`${provider}: an error answer names the status and never the key`, async () => {
    reply = () => ({ status: 401, errorBody: JSON.stringify({ error: { message: `invalid x-api-key ${KEY} (Bearer ${KEY})` } }) });
    await assert.rejects(collect(createHttpProvider(configFor(provider)).chat(MESSAGES)), (error: unknown) => {
      assert.ok(error instanceof ProviderError);
      assert.equal(error.status, 401);
      assert.ok(!error.message.includes(KEY), error.message);
      assert.ok(!error.message.includes('THIS-IS-A-FAKE'), error.message);
      return true;
    });
  });
}

test('the key goes in each provider\'s own header, never in the URL or the body', async () => {
  reply = () => ({ text: 'OK' });
  const expected: [ProviderId, string, string][] = [
    ['openai', 'authorization', `Bearer ${KEY}`],
    ['mistral', 'authorization', `Bearer ${KEY}`],
    ['openai-compatible', 'authorization', `Bearer ${KEY}`],
    ['azure-openai', 'api-key', KEY],
    ['anthropic', 'x-api-key', KEY],
    ['google', 'x-goog-api-key', KEY],
  ];
  for (const [provider, header, value] of expected) {
    await collect(createHttpProvider(configFor(provider)).chat(MESSAGES));
    const request = server.requests[server.requests.length - 1];
    assert.equal(request.headers[header], value, provider);
    assert.ok(!request.url.includes(KEY), `${provider}: the key is not in the URL`);
    assert.ok(!JSON.stringify(request.body).includes(KEY), `${provider}: the key is not in the body`);
  }
  const anthropic = server.requests.find((r) => r.format === 'anthropic')!;
  assert.equal(anthropic.headers['anthropic-version'], '2023-06-01');
});

test('each format carries the system text, the tools and the turns as its API wants them', async () => {
  reply = () => ({ text: 'OK' });
  const history: ChatMessage[] = [
    { role: 'system', content: 'SYSTEM RULES' },
    { role: 'user', content: 'first' },
    { role: 'assistant', content: 'calling', toolCalls: [{ id: 'c1', name: 'read_elements', input: { ids: ['LC-CAM'] } }] },
    { role: 'tool', content: 'the camera source', toolCallId: 'c1', toolName: 'read_elements' },
  ];
  const last = async (provider: ProviderId) => {
    await collect(createHttpProvider(configFor(provider)).chat(history, { tools: [TOOL] }));
    return server.requests[server.requests.length - 1].body as Record<string, any>;
  };
  const anthropic = await last('anthropic');
  assert.equal(anthropic.system, 'SYSTEM RULES');
  assert.deepEqual(anthropic.messages.map((m: { role: string }) => m.role), ['user', 'assistant', 'user']);
  assert.equal(anthropic.messages[1].content[1].type, 'tool_use');
  assert.equal(anthropic.messages[2].content[0].type, 'tool_result');
  assert.equal(anthropic.messages[2].content[0].tool_use_id, 'c1');
  assert.equal(anthropic.tools[0].input_schema.type, 'object');
  const openai = await last('openai');
  assert.equal(openai.messages[0].role, 'system');
  assert.equal(openai.messages[2].tool_calls[0].function.arguments, JSON.stringify({ ids: ['LC-CAM'] }));
  assert.equal(openai.messages[3].tool_call_id, 'c1');
  assert.equal(openai.max_completion_tokens, 1024);
  assert.equal(openai.temperature, undefined, 'no temperature unless set');
  const mistral = await last('mistral');
  assert.equal(mistral.max_tokens, 1024);
  const google = await last('google');
  assert.equal(google.systemInstruction.parts[0].text, 'SYSTEM RULES');
  assert.equal(google.contents[1].role, 'model');
  assert.equal(google.contents[2].parts[0].functionResponse.name, 'read_elements');
  assert.equal(JSON.stringify(google.tools).includes('additionalProperties'), false, 'Gemini gets its schema subset');
  const ollama = await last('ollama');
  assert.equal(ollama.messages[2].tool_calls[0].function.arguments.ids[0], 'LC-CAM');
  assert.equal(ollama.messages[3].role, 'tool');
  const azure = server.requests.filter((r) => r.format === 'openai').map((r) => r.url).find((u) => u.includes('/openai/deployments/'));
  assert.ok(!azure || /api-version=/.test(azure));
});

test('Azure OpenAI calls its deployment with an api-version', async () => {
  reply = () => ({ text: 'OK' });
  await collect(createHttpProvider(configFor('azure-openai', { apiVersion: '2025-01-01-preview' })).chat(MESSAGES));
  assert.match(server.requests[server.requests.length - 1].url, /^\/openai\/deployments\/arc-gpt\/chat\/completions\?api-version=2025-01-01-preview$/);
});

test('cancellation stops a stream in the middle', async () => {
  reply = () => ({ text: 'x'.repeat(400), chunkDelayMs: 20 });
  const controller = new AbortController();
  const provider = createHttpProvider(configFor('openai-compatible'));
  let received = '';
  await assert.rejects(
    (async () => {
      for await (const event of provider.chat(MESSAGES, { signal: controller.signal })) {
        if (event.type === 'text') received += event.text;
        if (received.length > 20) controller.abort();
      }
    })(),
    (error: unknown) => error instanceof ProviderError && error.kind === 'canceled',
  );
  assert.ok(received.length < 400);
});

test('a server that never answers times out with a clear message', async () => {
  reply = () => ({ hang: true });
  const config = { ...configFor('anthropic'), timeoutMs: 300 };
  const started = Date.now();
  await assert.rejects(collect(createHttpProvider(config).chat(MESSAGES)), (error: unknown) => error instanceof ProviderError && error.kind === 'timeout' && /no answer within/.test(error.message));
  assert.ok(Date.now() - started < 3000);
});

test('an unreachable endpoint is a network error without the key', async () => {
  const config = resolveConfig(settings('openai-compatible', { endpoint: 'http://127.0.0.1:9/v1', model: 'm' }), KEY);
  await assert.rejects(collect(createHttpProvider(config).chat(MESSAGES)), (error: unknown) => error instanceof ProviderError && error.kind === 'network' && !error.message.includes(KEY));
});

test('the agent loop runs a read-only tool and stops at the final tool', async () => {
  let round = 0;
  reply = (request) => {
    round++;
    if (round === 1) return { text: 'Let me read it.', toolCalls: [{ name: 'read_elements', input: { ids: ['LC-CAM'] } }] };
    assert.match(request.all, /CAMERA SOURCE/, 'the tool answer went back');
    return { toolCalls: [{ name: 'propose_edits', input: PROPOSAL }] };
  };
  for (const provider of ['anthropic', 'openai-compatible', 'google', 'ollama'] as ProviderId[]) {
    round = 0;
    const ran: string[] = [];
    const result = await converse(createHttpProvider(configFor(provider)), MESSAGES, {
      tools: [TOOL, { name: 'read_elements', description: 'read', parameters: { type: 'object', properties: { ids: { type: 'array', items: { type: 'string' } } } } }],
      final: ['propose_edits'],
      run: async (call) => {
        ran.push(call.name);
        return 'CAMERA SOURCE: component "Camera" { id: "LC-CAM" }';
      },
    });
    assert.deepEqual(ran, ['read_elements'], provider);
    assert.equal(result.rounds, 2, provider);
    assert.deepEqual(result.calls[0].input, PROPOSAL, provider);
  }
});

test('the settings resolve to each provider\'s defaults', () => {
  const anthropic = resolveConfig(settings('anthropic'));
  assert.equal(anthropic.model, 'claude-opus-5-5');
  assert.equal(anthropic.endpoint, 'https://api.anthropic.com');
  assert.equal(resolveConfig(settings('ollama')).endpoint, 'http://localhost:11434');
  assert.equal(resolveConfig(settings('nonsense')).provider, 'off');
  assert.equal(resolveConfig(settings('openai', { temperature: 7 })).temperature, 2);
  assert.match(missingPiece(resolveConfig(settings('anthropic'))) ?? '', /API key/);
  assert.equal(missingPiece(resolveConfig(settings('anthropic'), KEY)), null);
  assert.equal(missingPiece(resolveConfig(settings('ollama'))), null, 'Ollama needs no key');
  assert.match(missingPiece(resolveConfig(settings('openai-compatible'))) ?? '', /model|endpoint/);
  assert.match(missingPiece(resolveConfig(settings('off'))) ?? '', /off/);
});

test('consent is asked for clouds, not for the local machine', () => {
  assert.equal(needsConsent(resolveConfig(settings('anthropic'), KEY)), true);
  assert.equal(needsConsent(resolveConfig(settings('ollama'))), false);
  assert.equal(needsConsent(resolveConfig(settings('openai-compatible', { endpoint: 'http://localhost:1234/v1', model: 'm' }))), false);
  assert.equal(needsConsent(resolveConfig(settings('openai-compatible', { endpoint: 'https://llm.example.com/v1', model: 'm' }))), true);
  assert.equal(needsConsent(resolveConfig(settings('vscode-lm'))), false);
  assert.equal(isLocalEndpoint('http://127.0.0.1:8080'), true);
});

test('redaction removes the given secret and what looks like a key', () => {
  const text = `failed with key ${KEY}, header Authorization: Bearer abcdefghijklmnop, url ?key=AIzaSyA1234567890123456789012345 api_key="zzzzzzzzzz"`;
  const out = redact(text, [KEY]);
  assert.ok(!out.includes(KEY));
  assert.ok(!out.includes('abcdefghijklmnop'));
  assert.ok(!out.includes('AIzaSyA1234567890'));
  assert.ok(!out.includes('zzzzzzzzzz'));
  assert.match(out, /\[redacted\]/);
});

test('server-sent events survive chunks cut anywhere, \\r\\n and comments', async () => {
  const raw = ': comment\r\nevent: a\r\ndata: {"x":1}\r\n\r\ndata: line1\ndata: line2\n\ndata: [DONE]\n\n';
  for (const size of [1, 2, 3, 5, 50]) {
    const chunks = (async function* () {
      for (let i = 0; i < raw.length; i += size) yield raw.slice(i, i + size);
    })();
    const events = [];
    for await (const e of sseEvents(chunks)) events.push(e);
    assert.deepEqual(events, [{ event: 'a', data: '{"x":1}' }, { event: undefined, data: 'line1\nline2' }, { event: undefined, data: '[DONE]' }], `chunks of ${size}`);
  }
  const lines = [];
  for await (const l of ndjsonLines((async function* () { yield '{"a":'; yield '1}\n{"b"'; yield ':2}'; })())) lines.push(l);
  assert.deepEqual(lines, [{ a: 1 }, { b: 2 }]);
});

test('a redirect is never followed: the key headers stay with the endpoint', async () => {
  let elsewhere = 0;
  const other = await startFakeServer(() => {
    elsewhere += 1;
    return { text: 'OK' };
  });
  try {
    for (const provider of ['anthropic', 'openai', 'azure-openai', 'google'] as ProviderId[]) {
      reply = (request) => ({ redirectTo: `${other.url.replace('127.0.0.1', 'localhost')}${request.url}` });
      await assert.rejects(collect(createHttpProvider(configFor(provider)).chat(MESSAGES)), (error: unknown) => {
        assert.ok(error instanceof ProviderError, String(error));
        assert.match(error.message, /redirect/);
        assert.ok(!error.message.includes(KEY), error.message);
        return true;
      });
    }
    assert.equal(elsewhere, 0, 'nothing reached the address the redirect named');
  } finally {
    await other.close();
  }
});

test('http:// is refused for another machine, accepted for the local one', async () => {
  assert.equal(endpointProblem('https://api.example.com/v1'), null);
  for (const local of ['http://localhost:11434', 'http://127.0.0.1:8080/v1', 'http://[::1]:1234/v1']) assert.equal(endpointProblem(local), null, local);
  for (const remote of ['http://llm.example.com/v1', 'http://192.168.1.20:11434', 'http://localhost.example.com', 'ftp://x.example.com', 'file:///etc/passwd', 'not a url']) {
    assert.ok(endpointProblem(remote), remote);
  }
  assert.match(endpointProblem('http://llm.example.com/v1') ?? '', /https:\/\//);
  const remote = resolveConfig(settings('openai-compatible', { endpoint: 'http://llm.example.com/v1', model: 'm' }), KEY);
  assert.match(missingPiece(remote) ?? '', /unencrypted/);
  assert.throws(() => createHttpProvider(remote), (error: unknown) => error instanceof ProviderError && error.kind === 'config');
  const ollama = resolveConfig(settings('ollama', { endpoint: 'http://10.0.0.5:11434' }));
  assert.match(missingPiece(ollama) ?? '', /https/);
  assert.equal(missingPiece(resolveConfig(settings('ollama'))), null, 'the local Ollama is fine');
});

test('the user and password of an endpoint never show in the log or the consent', () => {
  const shown = displayUrl('https://alice:s3cretpass@llm.example.com/v1?key=AIzaSyA1234567890123456789012345');
  assert.ok(!shown.includes('alice') && !shown.includes('s3cretpass') && !shown.includes('AIzaSy'), shown);
  assert.match(shown, /^https:\/\/llm\.example\.com\/v1/);
  assert.ok(!displayUrl('https://bob:pw@[bad').includes('bob'));
});

test('an answer past 10 MB is cut, and so is a line that never ends', async () => {
  assert.equal(MAX_RESPONSE_BYTES, 10 * 1024 * 1024);
  reply = () => ({ flood: { bytes: MAX_RESPONSE_BYTES + 1024 * 1024, newlines: true } });
  await assert.rejects(collect(createHttpProvider(configFor('openai')).chat(MESSAGES)), (error: unknown) => error instanceof ProviderError && error.kind === 'protocol' && /larger than 10 MB/.test(error.message));
  reply = () => ({ flood: { bytes: MAX_EVENT_CHARS + 1024 * 1024 } });
  await assert.rejects(collect(createHttpProvider(configFor('anthropic')).chat(MESSAGES)), (error: unknown) => error instanceof ProviderError && error.kind === 'protocol' && /without its end/.test(error.message));
  reply = () => ({ flood: { bytes: MAX_EVENT_CHARS + 1024 * 1024 } });
  await assert.rejects(collect(createHttpProvider(configFor('ollama')).chat(MESSAGES)), (error: unknown) => error instanceof ProviderError && /without its end|larger than/.test(error.message));
});

test('the settings that widen what is sent, or where, are user (machine) settings only', () => {
  const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, '..', '..', 'package.json'), 'utf8'));
  const properties = manifest.contributes.configuration.properties ?? Object.assign({}, ...manifest.contributes.configuration.map((c: { properties: object }) => c.properties));
  for (const key of ['arclang.ai.provider', 'arclang.ai.endpoint', 'arclang.ai.azureDeployment', 'arclang.ai.apiVersion', 'arclang.ai.confirmBeforeSending', 'arclang.ai.suggestOnEdit', 'arclang.ai.contextScope']) {
    assert.equal(properties[key].scope, 'machine', key);
  }
});
