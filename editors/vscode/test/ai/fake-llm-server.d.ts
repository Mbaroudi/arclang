export interface FakeRequest {
  format: 'openai' | 'anthropic' | 'google' | 'ollama' | null;
  url: string;
  headers: Record<string, string | string[] | undefined>;
  body: Record<string, unknown>;
  lastUser: string;
  all: string;
}
export interface FakeReply {
  text?: string;
  toolCalls?: { name: string; input: Record<string, unknown> }[];
  status?: number;
  errorBody?: string;
  delayMs?: number;
  chunkDelayMs?: number;
  hang?: boolean;
  redirectTo?: string;
  redirectStatus?: number;
  flood?: { bytes: number; newlines?: boolean };
}
export interface FakeServer {
  url: string;
  requests: FakeRequest[];
  close(): Promise<void>;
}
export function startFakeServer(script: (request: FakeRequest) => FakeReply | undefined, port?: number): Promise<FakeServer>;
export function arclangScenario(request: FakeRequest): FakeReply;
export function formatOf(url: string): string | null;
