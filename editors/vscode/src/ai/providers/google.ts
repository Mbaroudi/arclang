// Google's Gemini API (generativelanguage), streamed as server-sent events,
// with its function calling.
import { displayUrl, joinUrl, parseJson, postStream, ProviderError, sseEvents } from './http';
import { ChatEvent, ChatMessage, ChatOptions, ChatProvider, ProviderConfig } from './types';

type Part = { text: string } | { functionCall: { name: string; args: Record<string, unknown> } } | { functionResponse: { name: string; response: { content: string } } };

interface Content {
  role: 'user' | 'model';
  parts: Part[];
}

export function toGemini(messages: ChatMessage[]): { system: string; contents: Content[] } {
  const system = messages.filter((m) => m.role === 'system').map((m) => m.content).join('\n\n');
  const contents: Content[] = [];
  const push = (role: 'user' | 'model', parts: Part[]) => {
    if (parts.length === 0) return;
    const last = contents[contents.length - 1];
    if (last && last.role === role) last.parts.push(...parts);
    else contents.push({ role, parts });
  };
  for (const m of messages) {
    if (m.role === 'system') continue;
    if (m.role === 'tool') push('user', [{ functionResponse: { name: m.toolName ?? 'tool', response: { content: m.content } } }]);
    else if (m.role === 'assistant') push('model', [...(m.content ? [{ text: m.content }] : []), ...(m.toolCalls ?? []).map((c) => ({ functionCall: { name: c.name, args: c.input } }))]);
    else push('user', [{ text: m.content }]);
  }
  return { system, contents };
}

const UNSUPPORTED = new Set(['additionalProperties', '$schema', 'default', 'examples', 'title', 'minItems', 'maxItems', 'pattern']);

/** JSON Schema as Gemini's OpenAPI subset takes it. */
export function geminiSchema(schema: unknown): unknown {
  if (Array.isArray(schema)) return schema.map(geminiSchema);
  if (!schema || typeof schema !== 'object') return schema;
  const out: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(schema as Record<string, unknown>)) {
    if (UNSUPPORTED.has(key)) continue;
    out[key] = key === 'properties' && value && typeof value === 'object' ? Object.fromEntries(Object.entries(value).map(([k, v]) => [k, geminiSchema(v)])) : geminiSchema(value);
  }
  return out;
}

interface StreamChunk {
  candidates?: { content?: { parts?: ({ text?: string; functionCall?: { name: string; args?: Record<string, unknown> } })[] }; finishReason?: string }[];
  usageMetadata?: { promptTokenCount?: number; candidatesTokenCount?: number };
  error?: { message?: string };
}

export class GoogleProvider implements ChatProvider {
  readonly id = 'google' as const;
  readonly model: string;
  readonly endpoint: string;
  private readonly url: string;

  constructor(private readonly config: ProviderConfig) {
    this.model = config.model;
    this.url = `${joinUrl(config.endpoint || 'https://generativelanguage.googleapis.com/v1beta', `/models/${encodeURIComponent(config.model)}:streamGenerateContent`)}?alt=sse`;
    this.endpoint = displayUrl(this.url);
  }

  async *chat(messages: ChatMessage[], options: ChatOptions = {}): AsyncGenerator<ChatEvent> {
    if (!this.config.apiKey) throw new ProviderError('Google Gemini needs an API key: run "ArcLang: Set AI API key"', 'config');
    const wire = toGemini(messages);
    const generationConfig: Record<string, unknown> = { maxOutputTokens: options.maxTokens ?? this.config.maxTokens };
    const temperature = options.temperature !== undefined ? options.temperature : this.config.temperature;
    if (typeof temperature === 'number') generationConfig.temperature = temperature;
    const body: Record<string, unknown> = { contents: wire.contents, generationConfig };
    if (wire.system) body.systemInstruction = { parts: [{ text: wire.system }] };
    if (options.tools?.length) body.tools = [{ functionDeclarations: options.tools.map((t) => ({ name: t.name, description: t.description, parameters: geminiSchema(t.parameters) })) }];
    const chunks = postStream({ url: this.url, headers: { 'x-goog-api-key': this.config.apiKey }, body, signal: options.signal, timeoutMs: this.config.timeoutMs, secrets: [this.config.apiKey], label: 'Google Gemini' });
    let calls = 0;
    let stopReason: string | undefined;
    let usage: { input?: number; output?: number } | undefined;
    for await (const event of sseEvents(chunks)) {
      const chunk = parseJson(event.data) as StreamChunk;
      if (chunk.error) throw new ProviderError(`Google Gemini: ${chunk.error.message ?? 'error'}`, 'http');
      if (chunk.usageMetadata) usage = { input: chunk.usageMetadata.promptTokenCount, output: chunk.usageMetadata.candidatesTokenCount };
      for (const candidate of chunk.candidates ?? []) {
        for (const part of candidate.content?.parts ?? []) {
          if (part.text) yield { type: 'text', text: part.text };
          if (part.functionCall) yield { type: 'tool_call', call: { id: `gemini-${++calls}`, name: part.functionCall.name, input: part.functionCall.args ?? {} } };
        }
        if (candidate.finishReason) stopReason = candidate.finishReason;
      }
    }
    yield { type: 'done', stopReason, usage };
  }
}
