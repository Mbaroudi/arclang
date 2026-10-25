// Anthropic's Messages API, streamed, with its native tool use.
import { displayUrl, joinUrl, parseArguments, parseJson, postStream, ProviderError, runsInBrowser, sseEvents } from './http';
import { ChatEvent, ChatMessage, ChatOptions, ChatProvider, ProviderConfig, ProviderId } from './types';

type Block =
  | { type: 'text'; text: string }
  | { type: 'tool_use'; id: string; name: string; input: Record<string, unknown> }
  | { type: 'tool_result'; tool_use_id: string; content: string };

interface WireMessage {
  role: 'user' | 'assistant';
  content: Block[];
}

/** The system text apart; the turns as Anthropic wants them: alternating, tool results in user turns. */
export function toAnthropic(messages: ChatMessage[]): { system: string; messages: WireMessage[] } {
  const system = messages.filter((m) => m.role === 'system').map((m) => m.content).join('\n\n');
  const out: WireMessage[] = [];
  const push = (role: 'user' | 'assistant', blocks: Block[]) => {
    if (blocks.length === 0) return;
    const last = out[out.length - 1];
    if (last && last.role === role) last.content.push(...blocks);
    else out.push({ role, content: blocks });
  };
  for (const m of messages) {
    if (m.role === 'system') continue;
    if (m.role === 'tool') push('user', [{ type: 'tool_result', tool_use_id: m.toolCallId ?? '', content: m.content }]);
    else if (m.role === 'assistant') push('assistant', [...(m.content ? [{ type: 'text' as const, text: m.content }] : []), ...(m.toolCalls ?? []).map((c) => ({ type: 'tool_use' as const, id: c.id, name: c.name, input: c.input }))]);
    else push('user', [{ type: 'text', text: m.content }]);
  }
  return { system, messages: out };
}

interface StreamEvent {
  type: string;
  index?: number;
  content_block?: { type: string; id?: string; name?: string; text?: string };
  delta?: { type?: string; text?: string; partial_json?: string; stop_reason?: string };
  message?: { usage?: { input_tokens?: number } };
  usage?: { output_tokens?: number };
  error?: { message?: string };
}

export class AnthropicProvider implements ChatProvider {
  readonly id: ProviderId;
  readonly model: string;
  readonly endpoint: string;
  private readonly url: string;

  constructor(private readonly config: ProviderConfig) {
    this.id = config.provider === 'arclang-hosted' ? 'arclang-hosted' : 'anthropic';
    this.model = config.model;
    this.url = joinUrl(config.endpoint || 'https://api.anthropic.com', '/v1/messages');
    this.endpoint = displayUrl(this.url);
  }

  async *chat(messages: ChatMessage[], options: ChatOptions = {}): AsyncGenerator<ChatEvent> {
    // The hosted provider is the site's gateway: it holds the key, the request carries none.
    const hosted = this.id === 'arclang-hosted';
    if (!hosted && !this.config.apiKey) throw new ProviderError('Anthropic needs an API key: run "ArcLang: Set AI API key"', 'config');
    const wire = toAnthropic(messages);
    const body: Record<string, unknown> = { model: this.model, max_tokens: options.maxTokens ?? this.config.maxTokens ?? 4096, messages: wire.messages, stream: true };
    if (wire.system) body.system = wire.system;
    const temperature = options.temperature !== undefined ? options.temperature : this.config.temperature;
    if (typeof temperature === 'number') body.temperature = temperature;
    if (options.tools?.length) body.tools = options.tools.map((t) => ({ name: t.name, description: t.description, input_schema: t.parameters }));
    const chunks = postStream({
      url: this.url,
      headers: hosted ? {} : { 'x-api-key': this.config.apiKey!, 'anthropic-version': '2023-06-01', ...(runsInBrowser() ? { 'anthropic-dangerous-direct-browser-access': 'true' } : {}) }, // web hook
      body,
      signal: options.signal,
      timeoutMs: this.config.timeoutMs,
      secrets: [this.config.apiKey],
      label: hosted ? 'ArcLang hosted AI' : 'Anthropic',
    });
    const tools = new Map<number, { id: string; name: string; json: string }>();
    let stopReason: string | undefined;
    const usage: { input?: number; output?: number } = {};
    for await (const event of sseEvents(chunks)) {
      const value = parseJson(event.data) as StreamEvent;
      switch (value.type) {
        case 'message_start':
          usage.input = value.message?.usage?.input_tokens;
          break;
        case 'content_block_start':
          if (value.content_block?.type === 'tool_use') tools.set(value.index ?? 0, { id: value.content_block.id ?? '', name: value.content_block.name ?? '', json: '' });
          else if (value.content_block?.type === 'text' && value.content_block.text) yield { type: 'text', text: value.content_block.text };
          break;
        case 'content_block_delta':
          if (value.delta?.type === 'text_delta' && value.delta.text) yield { type: 'text', text: value.delta.text };
          else if (value.delta?.type === 'input_json_delta') {
            const tool = tools.get(value.index ?? 0);
            if (tool) tool.json += value.delta.partial_json ?? '';
          }
          break;
        case 'content_block_stop': {
          const tool = tools.get(value.index ?? 0);
          if (tool) {
            tools.delete(value.index ?? 0);
            yield { type: 'tool_call', call: { id: tool.id, name: tool.name, input: parseArguments(tool.json) } };
          }
          break;
        }
        case 'message_delta':
          if (value.delta?.stop_reason) stopReason = value.delta.stop_reason;
          if (value.usage?.output_tokens !== undefined) usage.output = value.usage.output_tokens;
          break;
        case 'error':
          throw new ProviderError(`Anthropic: ${value.error?.message ?? 'error in the stream'}`, 'http');
        default:
          break; // ping, message_stop
      }
    }
    yield { type: 'done', stopReason, usage };
  }
}
