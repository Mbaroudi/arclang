// The configured provider as a VS Code language model, so that the @arclang
// participant and /agent run on it unchanged. With `vscode-lm` the model of
// the chat request is used as before.
import * as vscode from 'vscode';
import { ChatMessage, ChatProvider } from './providers';

/** VS Code's messages as the provider layer's: text, tool calls, tool results. */
export function fromLanguageModelMessages(messages: readonly vscode.LanguageModelChatMessage[]): ChatMessage[] {
  const out: ChatMessage[] = [];
  for (const message of messages) {
    const parts = message.content as unknown[];
    const text = parts.filter((p): p is vscode.LanguageModelTextPart => p instanceof vscode.LanguageModelTextPart).map((p) => p.value).join('');
    const calls = parts.filter((p): p is vscode.LanguageModelToolCallPart => p instanceof vscode.LanguageModelToolCallPart);
    const results = parts.filter((p): p is vscode.LanguageModelToolResultPart => p instanceof vscode.LanguageModelToolResultPart);
    if (message.role === vscode.LanguageModelChatMessageRole.Assistant) {
      out.push({ role: 'assistant', content: text, toolCalls: calls.length ? calls.map((c) => ({ id: c.callId, name: c.name, input: (c.input ?? {}) as Record<string, unknown> })) : undefined });
      continue;
    }
    for (const result of results) {
      const content = (result.content as unknown[]).map((c) => (c instanceof vscode.LanguageModelTextPart ? c.value : typeof c === 'string' ? c : JSON.stringify(c))).join('');
      out.push({ role: 'tool', content, toolCallId: result.callId });
    }
    if (text || results.length === 0) out.push({ role: 'user', content: text });
  }
  // Tool turns carry their tool's name for the formats that want it.
  const names = new Map(out.flatMap((m) => (m.toolCalls ?? []).map((c) => [c.id, c.name] as const)));
  return out.map((m) => (m.role === 'tool' && m.toolCallId ? { ...m, toolName: names.get(m.toolCallId) } : m));
}

export function providerChatModel(provider: ChatProvider, label: string): vscode.LanguageModelChat {
  const model: Partial<vscode.LanguageModelChat> = {
    id: `arclang-${provider.id}-${provider.model}`,
    name: label,
    vendor: provider.id,
    family: provider.model,
    version: '1',
    maxInputTokens: 200000,
    countTokens: async (value: string | vscode.LanguageModelChatMessage) => Math.ceil((typeof value === 'string' ? value.length : JSON.stringify(value.content).length) / 4),
    sendRequest: async (messages, options, token) => {
      const controller = new AbortController();
      token?.onCancellationRequested(() => controller.abort());
      const tools = (options?.tools ?? []).map((t) => ({ name: t.name, description: t.description, parameters: (t.inputSchema ?? { type: 'object', properties: {} }) as Record<string, unknown> }));
      const events = provider.chat(fromLanguageModelMessages(messages), { tools, signal: controller.signal });
      const parts: (vscode.LanguageModelTextPart | vscode.LanguageModelToolCallPart)[] = [];
      // The stream is read once, eagerly, so that `stream` and `text` both see it.
      const pump = (async () => {
        for await (const event of events) {
          if (event.type === 'text') parts.push(new vscode.LanguageModelTextPart(event.text));
          if (event.type === 'tool_call') parts.push(new vscode.LanguageModelToolCallPart(event.call.id, event.call.name, event.call.input));
        }
      })();
      pump.catch(() => undefined); // read by the consumers below
      const replay = async function* <T>(map: (part: vscode.LanguageModelTextPart | vscode.LanguageModelToolCallPart) => T | undefined): AsyncGenerator<T> {
        let index = 0;
        let finished = false;
        pump.then(() => (finished = true), () => (finished = true));
        for (;;) {
          while (index < parts.length) {
            const value = map(parts[index++]);
            if (value !== undefined) yield value;
          }
          if (finished) {
            await pump; // rethrows the provider's error
            while (index < parts.length) {
              const value = map(parts[index++]);
              if (value !== undefined) yield value;
            }
            return;
          }
          await Promise.race([pump.catch(() => undefined), new Promise((r) => setTimeout(r, 15))]);
        }
      };
      return {
        stream: replay((p) => p),
        text: replay((p) => (p instanceof vscode.LanguageModelTextPart ? p.value : undefined)),
      } as unknown as vscode.LanguageModelChatResponse;
    },
  };
  return model as vscode.LanguageModelChat;
}
