// One exchange with the AI that may call tools: the read-only tools run
// here and their answers go back, until the AI answers or calls the final
// tool. Nothing an AI says is run except these named tools.
import type { ChatMessage, ChatProvider, ToolCall, ToolSpec } from '../providers/types';

export interface ConversationResult {
  text: string;
  /** The final tool's call, when the AI made it. */
  calls: ToolCall[];
  rounds: number;
}

export interface ConversationOptions {
  tools: ToolSpec[];
  /** The tools that end the exchange (propose_edits, propose_changes). */
  final: string[];
  /** Runs a read-only tool; its text goes back to the AI. */
  run: (call: ToolCall) => Promise<string>;
  signal?: AbortSignal;
  maxRounds?: number;
  onText?: (text: string) => void;
  maxTokens?: number;
}

export async function converse(provider: ChatProvider, messages: ChatMessage[], options: ConversationOptions): Promise<ConversationResult> {
  const history = [...messages];
  let text = '';
  const maxRounds = options.maxRounds ?? 5;
  for (let round = 1; round <= maxRounds; round++) {
    let roundText = '';
    const calls: ToolCall[] = [];
    for await (const event of provider.chat(history, { tools: options.tools, signal: options.signal, maxTokens: options.maxTokens })) {
      if (event.type === 'text') {
        roundText += event.text;
        options.onText?.(event.text);
      } else if (event.type === 'tool_call') {
        calls.push(event.call);
      }
    }
    text += roundText;
    const final = calls.filter((c) => options.final.includes(c.name));
    if (final.length || calls.length === 0) return { text, calls: final, rounds: round };
    history.push({ role: 'assistant', content: roundText, toolCalls: calls });
    for (const call of calls) {
      const known = options.tools.some((t) => t.name === call.name);
      let answer: string;
      try {
        answer = known ? await options.run(call) : `There is no tool named ${call.name}.`;
      } catch (error) {
        answer = `${call.name} failed: ${error instanceof Error ? error.message : String(error)}`;
      }
      history.push({ role: 'tool', content: answer, toolCallId: call.id, toolName: call.name });
    }
  }
  return { text, calls: [], rounds: maxRounds };
}
