// OpenAI's Chat Completions format, streamed: OpenAI, Azure OpenAI, Mistral
// and any OpenAI-compatible server (LM Studio, vLLM, llama.cpp, LiteLLM…).
import { displayUrl, joinUrl, parseArguments, parseJson, postStream, ProviderError, sseEvents } from './http';
import { ChatEvent, ChatMessage, ChatOptions, ChatProvider, ProviderConfig, ProviderId, ToolCall } from './types';

const LABELS: Partial<Record<ProviderId, string>> = { openai: 'OpenAI', 'azure-openai': 'Azure OpenAI', mistral: 'Mistral', 'openai-compatible': 'The OpenAI-compatible server' };

interface WireToolCall {
  id: string;
  type: 'function';
  function: { name: string; arguments: string };
}

type WireMessage =
  | { role: 'system' | 'user'; content: string }
  | { role: 'assistant'; content: string | null; tool_calls?: WireToolCall[] }
  | { role: 'tool'; tool_call_id: string; content: string };

export function toOpenAiMessages(messages: ChatMessage[]): WireMessage[] {
  return messages.map((m): WireMessage => {
    if (m.role === 'tool') return { role: 'tool', tool_call_id: m.toolCallId ?? '', content: m.content };
    if (m.role === 'assistant') {
      const calls = (m.toolCalls ?? []).map((c) => ({ id: c.id, type: 'function' as const, function: { name: c.name, arguments: JSON.stringify(c.input) } }));
      return calls.length ? { role: 'assistant', content: m.content || null, tool_calls: calls } : { role: 'assistant', content: m.content };
    }
    return { role: m.role, content: m.content };
  });
}

interface Delta {
  content?: string | null;
  tool_calls?: { index?: number; id?: string; function?: { name?: string; arguments?: string } }[];
}

interface Chunk {
  choices?: { delta?: Delta; finish_reason?: string | null }[];
  usage?: { prompt_tokens?: number; completion_tokens?: number } | null;
  error?: { message?: string } | string;
}

export class OpenAiProvider implements ChatProvider {
  readonly id: ProviderId;
  readonly model: string;
  readonly endpoint: string;
  private readonly url: string;
  private readonly headers: Record<string, string>;

  constructor(private readonly config: ProviderConfig) {
    this.id = config.provider;
    this.model = config.model;
    const key = config.apiKey;
    if (config.provider === 'azure-openai') {
      if (!config.endpoint) throw new ProviderError('Azure OpenAI needs arclang.ai.endpoint (https://<resource>.openai.azure.com)', 'config');
      const deployment = config.azureDeployment || config.model;
      if (!deployment) throw new ProviderError('Azure OpenAI needs arclang.ai.azureDeployment', 'config');
      this.url = `${joinUrl(config.endpoint, `/openai/deployments/${encodeURIComponent(deployment)}/chat/completions`)}?api-version=${encodeURIComponent(config.apiVersion || '2024-10-21')}`;
      this.headers = key ? { 'api-key': key } : {};
    } else {
      if (!config.endpoint) throw new ProviderError('an OpenAI-compatible provider needs arclang.ai.endpoint (for example http://localhost:1234/v1)', 'config');
      this.url = joinUrl(config.endpoint, '/chat/completions');
      this.headers = key ? { authorization: `Bearer ${key}` } : {};
    }
    this.endpoint = displayUrl(this.url);
  }

  async *chat(messages: ChatMessage[], options: ChatOptions = {}): AsyncGenerator<ChatEvent> {
    const label = LABELS[this.id] ?? 'The provider';
    const body: Record<string, unknown> = { model: this.model, messages: toOpenAiMessages(messages), stream: true };
    const maxTokens = options.maxTokens ?? this.config.maxTokens;
    // OpenAI's own API counts output tokens under a new name; the others keep the old one.
    if (maxTokens) body[this.id === 'openai' || this.id === 'azure-openai' ? 'max_completion_tokens' : 'max_tokens'] = maxTokens;
    const temperature = options.temperature !== undefined ? options.temperature : this.config.temperature;
    if (typeof temperature === 'number') body.temperature = temperature;
    if (options.tools?.length) {
      body.tools = options.tools.map((t) => ({ type: 'function', function: { name: t.name, description: t.description, parameters: t.parameters } }));
      body.tool_choice = 'auto';
    }
    if (this.id === 'openai') body.stream_options = { include_usage: true };
    const chunks = postStream({ url: this.url, headers: this.headers, body, signal: options.signal, timeoutMs: this.config.timeoutMs, secrets: [this.config.apiKey], label });
    const calls = new Map<number, { id: string; name: string; args: string }>();
    let stopReason: string | undefined;
    let usage: { input?: number; output?: number } | undefined;
    const flushCalls = function* (): Generator<ChatEvent> {
      for (const [index, call] of [...calls.entries()].sort((a, b) => a[0] - b[0])) {
        const out: ToolCall = { id: call.id || `call_${index}`, name: call.name, input: parseArguments(call.args) };
        yield { type: 'tool_call', call: out };
      }
      calls.clear();
    };
    for await (const event of sseEvents(chunks)) {
      if (event.data === '[DONE]') break;
      const chunk = parseJson(event.data) as Chunk;
      if (chunk.error) throw new ProviderError(`${label}: ${typeof chunk.error === 'string' ? chunk.error : (chunk.error.message ?? 'error')}`, 'http');
      if (chunk.usage) usage = { input: chunk.usage.prompt_tokens, output: chunk.usage.completion_tokens };
      for (const choice of chunk.choices ?? []) {
        const delta = choice.delta ?? {};
        if (delta.content) yield { type: 'text', text: delta.content };
        for (const [position, part] of (delta.tool_calls ?? []).entries()) {
          const index = part.index ?? position;
          const call = calls.get(index) ?? { id: '', name: '', args: '' };
          if (part.id) call.id = part.id;
          if (part.function?.name) call.name += part.function.name;
          if (part.function?.arguments) call.args += part.function.arguments;
          calls.set(index, call);
        }
        if (choice.finish_reason) stopReason = choice.finish_reason;
      }
    }
    yield* flushCalls();
    yield { type: 'done', stopReason, usage };
  }
}
