// The language models VS Code offers (GitHub Copilot and any other
// extension's), behind the same interface as the HTTP providers.
import * as vscode from 'vscode';
import { ChatEvent, ChatMessage, ChatOptions, ChatProvider } from './types';
import { ProviderError } from './http';

/** The model the setting names (by id, family or name), else the first one VS Code offers. */
export async function selectVscodeModel(wanted: string): Promise<vscode.LanguageModelChat | undefined> {
  const all = await vscode.lm.selectChatModels();
  if (!wanted) return all.find((m) => m.vendor === 'copilot') ?? all[0];
  return all.find((m) => m.id === wanted) ?? all.find((m) => m.family === wanted) ?? all.find((m) => m.name === wanted);
}

export function toLanguageModelMessages(messages: ChatMessage[]): vscode.LanguageModelChatMessage[] {
  const out: vscode.LanguageModelChatMessage[] = [];
  for (const m of messages) {
    if (m.role === 'system' || m.role === 'user') out.push(vscode.LanguageModelChatMessage.User(m.content));
    else if (m.role === 'assistant') out.push(vscode.LanguageModelChatMessage.Assistant([...(m.content ? [new vscode.LanguageModelTextPart(m.content)] : []), ...(m.toolCalls ?? []).map((c) => new vscode.LanguageModelToolCallPart(c.id, c.name, c.input))]));
    else out.push(vscode.LanguageModelChatMessage.User([new vscode.LanguageModelToolResultPart(m.toolCallId ?? '', [new vscode.LanguageModelTextPart(m.content)])]));
  }
  return out;
}

export class VscodeLmProvider implements ChatProvider {
  readonly id = 'vscode-lm' as const;
  readonly endpoint = 'vscode.lm';
  readonly model: string;

  constructor(private readonly chatModel: vscode.LanguageModelChat) {
    this.model = `${chatModel.vendor}/${chatModel.family}`;
  }

  async *chat(messages: ChatMessage[], options: ChatOptions = {}): AsyncGenerator<ChatEvent> {
    const source = new vscode.CancellationTokenSource();
    const onAbort = () => source.cancel();
    options.signal?.addEventListener('abort', onAbort, { once: true });
    try {
      const response = await this.chatModel.sendRequest(
        toLanguageModelMessages(messages),
        { tools: (options.tools ?? []).map((t) => ({ name: t.name, description: t.description, inputSchema: t.parameters })), justification: 'ArcLang: AI design assistance on the open model' },
        source.token,
      );
      for await (const part of response.stream) {
        if (options.signal?.aborted) throw new ProviderError('VS Code language model: cancelled', 'cancelled');
        if (part instanceof vscode.LanguageModelTextPart) yield { type: 'text', text: part.value };
        else if (part instanceof vscode.LanguageModelToolCallPart) yield { type: 'tool_call', call: { id: part.callId, name: part.name, input: (part.input ?? {}) as Record<string, unknown> } };
      }
      yield { type: 'done' };
    } catch (error) {
      if (error instanceof ProviderError) throw error;
      if (options.signal?.aborted) throw new ProviderError('VS Code language model: cancelled', 'cancelled');
      throw new ProviderError(`VS Code language model: ${error instanceof Error ? error.message : String(error)}`, 'http');
    } finally {
      options.signal?.removeEventListener('abort', onAbort);
      source.dispose();
    }
  }
}
