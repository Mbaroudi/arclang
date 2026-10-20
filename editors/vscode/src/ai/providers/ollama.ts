// Ollama's own chat API on the local machine: no key, newline-delimited
// JSON, tools in the OpenAI shape with arguments as objects.
import { displayUrl, joinUrl, ndjsonLines, postStream, ProviderError } from './http';
import { ChatEvent, ChatMessage, ChatOptions, ChatProvider, ProviderConfig } from './types';

type WireMessage =
  | { role: 'system' | 'user'; content: string }
  | { role: 'assistant'; content: string; tool_calls?: { function: { name: string; arguments: Record<string, unknown> } }[] }
  | { role: 'tool'; content: string; tool_name?: string };

export function toOllama(messages: ChatMessage[]): WireMessage[] {
  return messages.map((m): WireMessage => {
    if (m.role === 'tool') return { role: 'tool', content: m.content, tool_name: m.toolName };
    if (m.role === 'assistant') return m.toolCalls?.length ? { role: 'assistant', content: m.content, tool_calls: m.toolCalls.map((c) => ({ function: { name: c.name, arguments: c.input } })) } : { role: 'assistant', content: m.content };
    return { role: m.role, content: m.content };
  });
}

interface Line {
  message?: { content?: string; tool_calls?: { function?: { name?: string; arguments?: Record<string, unknown> | string } }[] };
  done?: boolean;
  done_reason?: string;
  prompt_eval_count?: number;
  eval_count?: number;
  error?: string;
}

export class OllamaProvider implements ChatProvider {
  readonly id = 'ollama' as const;
  readonly model: string;
  readonly endpoint: string;
  private readonly url: string;

  constructor(private readonly config: ProviderConfig) {
    this.model = config.model;
    this.url = joinUrl(config.endpoint || 'http://localhost:11434', '/api/chat');
    this.endpoint = displayUrl(this.url);
  }

  async *chat(messages: ChatMessage[], options: ChatOptions = {}): AsyncGenerator<ChatEvent> {
    const ollamaOptions: Record<string, unknown> = { num_predict: options.maxTokens ?? this.config.maxTokens };
    const temperature = options.temperature !== undefined ? options.temperature : this.config.temperature;
    if (typeof temperature === 'number') ollamaOptions.temperature = temperature;
    const body: Record<string, unknown> = { model: this.model, messages: toOllama(messages), stream: true, options: ollamaOptions };
    if (options.tools?.length) body.tools = options.tools.map((t) => ({ type: 'function', function: { name: t.name, description: t.description, parameters: t.parameters } }));
    const headers: Record<string, string> = this.config.apiKey ? { authorization: `Bearer ${this.config.apiKey}` } : {};
    const chunks = postStream({ url: this.url, headers, body, signal: options.signal, timeoutMs: this.config.timeoutMs, secrets: [this.config.apiKey], label: 'Ollama' });
    let calls = 0;
    for await (const raw of ndjsonLines(chunks)) {
      const line = raw as Line;
      if (line.error) throw new ProviderError(`Ollama: ${line.error}`, 'http');
      if (line.message?.content) yield { type: 'text', text: line.message.content };
      for (const call of line.message?.tool_calls ?? []) {
        const args = call.function?.arguments;
        const input = typeof args === 'string' ? (JSON.parse(args || '{}') as Record<string, unknown>) : (args ?? {});
        yield { type: 'tool_call', call: { id: `ollama-${++calls}`, name: call.function?.name ?? '', input } };
      }
      if (line.done) {
        yield { type: 'done', stopReason: line.done_reason, usage: { input: line.prompt_eval_count, output: line.eval_count } };
        return;
      }
    }
    yield { type: 'done' };
  }
}
