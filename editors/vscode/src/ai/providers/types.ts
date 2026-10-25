// One interface for every AI provider the extension can talk to: messages
// in, a stream of text and tool calls out. No SDK: `fetch` and the
// provider's own wire format. Nothing here imports VS Code, so the unit
// tests run the providers against a local fake server.

export type ProviderId = 'off' | 'vscode-lm' | 'arclang-hosted' | 'anthropic' | 'openai' | 'azure-openai' | 'mistral' | 'google' | 'ollama' | 'openai-compatible';

export type Role = 'system' | 'user' | 'assistant' | 'tool';

export interface ToolCall {
  id: string;
  name: string;
  input: Record<string, unknown>;
}

export interface ChatMessage {
  role: Role;
  content: string;
  /** An assistant turn's tool calls. */
  toolCalls?: ToolCall[];
  /** A tool turn: the call it answers, and the tool's name (Gemini and Ollama want it). */
  toolCallId?: string;
  toolName?: string;
}

/** A tool the model may call: a name, what it does, its input as JSON Schema. */
export interface ToolSpec {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
}

export type ChatEvent =
  | { type: 'text'; text: string }
  | { type: 'tool_call'; call: ToolCall }
  | { type: 'done'; stopReason?: string; usage?: { input?: number; output?: number } };

export interface ChatOptions {
  tools?: ToolSpec[];
  signal?: AbortSignal;
  maxTokens?: number;
  /** Omitted: the provider's own default (some models refuse any other value). */
  temperature?: number | null;
}

export interface ChatProvider {
  readonly id: ProviderId;
  readonly model: string;
  /** Where the requests go, for the log and the consent; never with a key. */
  readonly endpoint: string;
  chat(messages: ChatMessage[], options?: ChatOptions): AsyncIterable<ChatEvent>;
}

export interface ProviderConfig {
  provider: ProviderId;
  model: string;
  endpoint: string;
  apiKey?: string;
  azureDeployment?: string;
  apiVersion?: string;
  maxTokens: number;
  temperature: number | null;
  /** Longest wait for the first byte and between two chunks. */
  timeoutMs: number;
}
