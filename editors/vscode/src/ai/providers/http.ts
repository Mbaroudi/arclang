// What every HTTP provider shares: a POST that streams, with a timeout, the
// caller's cancellation, and errors that never carry a key.

/** The most an answer may carry: past it the stream is cut (a server gone wrong, not a model). */
export const MAX_RESPONSE_BYTES = 10 * 1024 * 1024;
/** The most one line or one event of a stream may hold before its end comes. */
export const MAX_EVENT_CHARS = 4 * 1024 * 1024;

export type ErrorKind = 'http' | 'timeout' | 'canceled' | 'network' | 'protocol' | 'config';

export class ProviderError extends Error {
  constructor(
    message: string,
    readonly kind: ErrorKind,
    readonly status?: number,
  ) {
    super(message);
    this.name = 'ProviderError';
  }
}

const KEY_PATTERNS: RegExp[] = [
  /\bsk-[A-Za-z0-9_-]{8,}/g,
  /\bsk-ant-[A-Za-z0-9_-]{8,}/g,
  /\bAIza[0-9A-Za-z_-]{20,}/g,
  /\b(Bearer)\s+[A-Za-z0-9._~+/=-]{8,}/gi,
  /\b((?:api[-_]?key|x-api-key|x-goog-api-key|key|token|access_token)["']?\s*[:=]\s*["']?)[^"'&\s,}]{6,}/gi,
];

/** The text with every given secret, and anything shaped like a key, replaced by [redacted]. */
export function redact(text: string, secrets: (string | undefined | null)[] = []): string {
  let out = String(text);
  for (const secret of secrets) {
    if (secret && secret.length >= 4) out = out.split(secret).join('[redacted]');
  }
  for (const pattern of KEY_PATTERNS) {
    out = out.replace(pattern, (_match, prefix?: string) => (typeof prefix === 'string' && prefix.length < 40 ? `${prefix}${/bearer/i.test(prefix) ? ' ' : ''}[redacted]` : '[redacted]'));
  }
  return out;
}

/** A URL without its user and password (`https://u:p@host`) nor its query string's secrets (Gemini can take `?key=`), for logs and the consent. */
export function displayUrl(url: string): string {
  try {
    const parsed = new URL(url);
    parsed.username = '';
    parsed.password = '';
    for (const name of [...parsed.searchParams.keys()]) if (/key|token|sig/i.test(name)) parsed.searchParams.set(name, 'redacted');
    return parsed.toString();
  } catch {
    return redact(url.replace(/\/\/[^/@\s]*@/, '//'));
  }
}

/** Whether this code runs in a browser's worker (the web extension): calls to a provider are then cross-origin requests. */
export function runsInBrowser(): boolean { // web hook
  return typeof (globalThis as { importScripts?: unknown }).importScripts === 'function';
}

/** Whether a host is the local machine. */
export function isLocalHost(hostname: string): boolean {
  return hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1' || hostname === '[::1]';
}

/**
 * Why an endpoint must not be called, or null. Keys and models travel in
 * clear over http: it is allowed for the local machine only.
 */
export function endpointProblem(endpoint: string): string | null {
  let parsed: URL;
  try {
    parsed = new URL(endpoint);
  } catch {
    return `the endpoint ${displayUrl(endpoint)} is not a URL`;
  }
  if (parsed.protocol === 'https:') return null;
  if (parsed.protocol === 'http:' && isLocalHost(parsed.hostname)) return null;
  if (parsed.protocol === 'http:') {
    return `the endpoint ${displayUrl(endpoint)} uses http:// to another machine: the key and the model would travel unencrypted. Use https:// (http:// is accepted for localhost, 127.0.0.1 and ::1 only)`;
  }
  return `the endpoint ${displayUrl(endpoint)} must be an https:// URL`;
}

export interface StreamRequest {
  url: string;
  headers: Record<string, string>;
  body: unknown;
  signal?: AbortSignal;
  timeoutMs: number;
  /** Values never to show in an error. */
  secrets: (string | undefined)[];
  /** "Anthropic", "OpenAI"…: who answered, for the messages. */
  label: string;
}

/** The message a provider put in its error body, else the body itself (clipped). */
export function errorMessageOf(body: string): string {
  try {
    const value = JSON.parse(body) as { error?: { message?: string } | string; message?: string };
    if (typeof value.error === 'string') return value.error;
    if (value.error && typeof value.error.message === 'string') return value.error.message;
    if (typeof value.message === 'string') return value.message;
  } catch {
    // not JSON
  }
  return body.trim().slice(0, 500) || '(empty answer)';
}

/**
 * POST the body as JSON and yield the answer's text as it arrives. The
 * timeout counts from the request to the first byte and between two chunks;
 * the caller's signal cancels at any point.
 */
export async function* postStream(request: StreamRequest): AsyncGenerator<string> {
  const controller = new AbortController();
  let timedOut = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const arm = () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      timedOut = true;
      controller.abort();
    }, request.timeoutMs);
  };
  const onAbort = () => controller.abort();
  if (request.signal?.aborted) throw new ProviderError(`${request.label}: canceled`, 'canceled');
  request.signal?.addEventListener('abort', onAbort, { once: true });
  const fail = (error: unknown): ProviderError => {
    if (error instanceof ProviderError) return error;
    if (request.signal?.aborted) return new ProviderError(`${request.label}: canceled`, 'canceled');
    if (timedOut) return new ProviderError(`${request.label}: no answer within ${Math.round(request.timeoutMs / 1000)} s`, 'timeout');
    const message = error instanceof Error ? `${error.message}${error.cause instanceof Error ? ` (${error.cause.message})` : ''}` : String(error);
    return new ProviderError(`${request.label}: ${redact(message, request.secrets)}`, 'network');
  };
  const problem = endpointProblem(request.url);
  if (problem) throw new ProviderError(`${request.label}: ${problem}`, 'config');
  try {
    arm();
    let response: Response;
    try {
      response = await fetch(request.url, {
        method: 'POST',
        headers: { 'content-type': 'application/json', accept: 'text/event-stream, application/x-ndjson, application/json', ...request.headers },
        body: JSON.stringify(request.body),
        signal: controller.signal,
        // A redirect would carry the key headers to wherever it points: never followed.
        redirect: 'error',
      });
    } catch (error) {
      if (!timedOut && !request.signal?.aborted && /redirect/i.test(causeText(error))) {
        throw new ProviderError(`${request.label}: the server answered with a redirect, which is not followed (the key would go along); set the final address in arclang.ai.endpoint`, 'http');
      }
      throw fail(error);
    }
    if (response.status >= 300 && response.status < 400) {
      throw new ProviderError(`${request.label}: the server answered with a redirect (${response.status}), which is not followed; set the final address in arclang.ai.endpoint`, 'http', response.status);
    }
    if (!response.ok) {
      let body = '';
      try {
        body = await readCapped(response, 64 * 1024);
      } catch {
        // the status alone
      }
      throw new ProviderError(`${request.label} answered ${response.status}: ${redact(errorMessageOf(body), request.secrets)}`, 'http', response.status);
    }
    if (!response.body) throw new ProviderError(`${request.label}: the answer has no body`, 'protocol');
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let received = 0;
    try {
      for (;;) {
        arm();
        let chunk: { done: boolean; value?: Uint8Array };
        try {
          chunk = await reader.read();
        } catch (error) {
          throw fail(error);
        }
        if (chunk.done) break;
        received += chunk.value?.byteLength ?? 0;
        if (received > MAX_RESPONSE_BYTES) throw new ProviderError(`${request.label}: the answer is larger than ${MAX_RESPONSE_BYTES / (1024 * 1024)} MB and was cut`, 'protocol');
        const text = chunk.value ? decoder.decode(chunk.value, { stream: true }) : '';
        if (text) yield text;
      }
      const rest = decoder.decode();
      if (rest) yield rest;
    } finally {
      reader.cancel().catch(() => undefined);
    }
  } finally {
    clearTimeout(timer);
    request.signal?.removeEventListener('abort', onAbort);
  }
}

/** The message of an error and of its causes. */
function causeText(error: unknown): string {
  const parts: string[] = [];
  for (let e: unknown = error, depth = 0; e instanceof Error && depth < 4; e = e.cause, depth += 1) parts.push(e.message);
  return parts.join(' ');
}

/** The body as text, at most `limit` bytes of it. */
async function readCapped(response: Response, limit: number): Promise<string> {
  if (!response.body) return '';
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let text = '';
  let size = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done || !value) break;
      size += value.byteLength;
      text += decoder.decode(value.subarray(0, Math.max(0, value.byteLength - Math.max(0, size - limit))), { stream: true });
      if (size >= limit) break;
    }
  } finally {
    reader.cancel().catch(() => undefined);
  }
  return text;
}

const tooLong = (what: string): ProviderError => new ProviderError(`the provider sent ${what} longer than ${MAX_EVENT_CHARS / (1024 * 1024)} M characters without its end`, 'protocol');

export interface SseEvent {
  event?: string;
  data: string;
}

/** Server-sent events from text chunks cut anywhere. */
export async function* sseEvents(chunks: AsyncIterable<string>): AsyncGenerator<SseEvent> {
  let buffer = '';
  let event: string | undefined;
  let data: string[] = [];
  const flush = (): SseEvent | null => {
    if (data.length === 0) {
      event = undefined;
      return null;
    }
    const out = { event, data: data.join('\n') };
    event = undefined;
    data = [];
    return out;
  };
  const lines = function* (final: boolean): Generator<string> {
    for (;;) {
      const at = buffer.search(/\r\n|\n|\r/);
      if (at < 0) break;
      const width = buffer[at] === '\r' && buffer[at + 1] === '\n' ? 2 : 1;
      // A lone \r at the very end may be the first half of \r\n.
      if (!final && buffer[at] === '\r' && at === buffer.length - 1) break;
      const line = buffer.slice(0, at);
      buffer = buffer.slice(at + width);
      yield line;
    }
  };
  const handle = function* (line: string): Generator<SseEvent> {
    if (line === '') {
      const out = flush();
      if (out) yield out;
      return;
    }
    if (line.startsWith(':')) return;
    const colon = line.indexOf(':');
    const field = colon < 0 ? line : line.slice(0, colon);
    let value = colon < 0 ? '' : line.slice(colon + 1);
    if (value.startsWith(' ')) value = value.slice(1);
    if (field === 'event') event = value;
    else if (field === 'data') data.push(value);
  };
  let pending = 0;
  for await (const chunk of chunks) {
    buffer += chunk;
    for (const line of lines(false)) {
      pending = line === '' ? 0 : pending + line.length;
      if (pending > MAX_EVENT_CHARS) throw tooLong('an event');
      yield* handle(line);
    }
    if (buffer.length > MAX_EVENT_CHARS) throw tooLong('a line');
  }
  for (const line of lines(true)) yield* handle(line);
  if (buffer) yield* handle(buffer);
  const out = flush();
  if (out) yield out;
}

/** One JSON value per line (Ollama), from text chunks cut anywhere. */
export async function* ndjsonLines(chunks: AsyncIterable<string>): AsyncGenerator<unknown> {
  let buffer = '';
  for await (const chunk of chunks) {
    buffer += chunk;
    let at: number;
    while ((at = buffer.indexOf('\n')) >= 0) {
      const line = buffer.slice(0, at).trim();
      buffer = buffer.slice(at + 1);
      if (line) yield parseJson(line);
    }
    if (buffer.length > MAX_EVENT_CHARS) throw tooLong('a line');
  }
  if (buffer.trim()) yield parseJson(buffer.trim());
}

export function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    throw new ProviderError(`the provider sent text that is not JSON: ${text.slice(0, 120)}`, 'protocol');
  }
}

/** A tool call's arguments as an object; text that is not a JSON object is kept under `_raw`. */
export function parseArguments(text: string): Record<string, unknown> {
  if (!text.trim()) return {};
  try {
    const value = JSON.parse(text) as unknown;
    return value && typeof value === 'object' && !Array.isArray(value) ? (value as Record<string, unknown>) : { _raw: text };
  } catch {
    return { _raw: text };
  }
}

/** `base` without trailing slashes, `path` appended. */
export function joinUrl(base: string, path: string): string {
  return `${base.replace(/\/+$/, '')}${path.startsWith('/') ? path : `/${path}`}`;
}
