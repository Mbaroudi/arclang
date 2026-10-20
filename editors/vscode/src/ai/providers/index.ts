// The providers the settings can name, their defaults, and the factory.
import { AnthropicProvider } from './anthropic';
import { GoogleProvider } from './google';
import { endpointProblem, isLocalHost, ProviderError } from './http';
import { OllamaProvider } from './ollama';
import { OpenAiProvider } from './openai';
import { ChatProvider, ProviderConfig, ProviderId } from './types';

export * from './types';
export { ProviderError, redact, displayUrl, endpointProblem } from './http';

export interface ProviderInfo {
  id: ProviderId;
  label: string;
  /** The model when `arclang.ai.model` is empty; the first of `models`. */
  models: string[];
  endpoint: string;
  /** Needs a key in the secret storage before anything is sent. */
  needsKey: boolean;
  /** Sends the model to a server of someone else (asks first, see arclang.ai.confirmBeforeSending). */
  cloud: boolean;
  keyHelp?: string;
}

export const PROVIDERS: ProviderInfo[] = [
  { id: 'vscode-lm', label: 'VS Code language models (Copilot…)', models: [], endpoint: 'vscode.lm', needsKey: false, cloud: false },
  { id: 'anthropic', label: 'Anthropic Claude', models: ['claude-opus-5-5', 'claude-sonnet-5-5', 'claude-haiku-5-5'], endpoint: 'https://api.anthropic.com', needsKey: true, cloud: true, keyHelp: 'console.anthropic.com → API keys' },
  { id: 'openai', label: 'OpenAI', models: ['gpt-5', 'gpt-5-mini', 'gpt-4.1'], endpoint: 'https://api.openai.com/v1', needsKey: true, cloud: true, keyHelp: 'platform.openai.com → API keys' },
  { id: 'azure-openai', label: 'Azure OpenAI', models: [], endpoint: '', needsKey: true, cloud: true, keyHelp: 'Azure portal → your OpenAI resource → Keys and Endpoint' },
  { id: 'mistral', label: 'Mistral AI', models: ['mistral-large-latest', 'codestral-latest', 'mistral-small-latest'], endpoint: 'https://api.mistral.ai/v1', needsKey: true, cloud: true, keyHelp: 'console.mistral.ai → API keys' },
  { id: 'google', label: 'Google Gemini', models: ['gemini-2.5-pro', 'gemini-2.5-flash'], endpoint: 'https://generativelanguage.googleapis.com/v1beta', needsKey: true, cloud: true, keyHelp: 'aistudio.google.com → Get API key' },
  { id: 'ollama', label: 'Ollama (local)', models: ['llama3.1', 'qwen2.5-coder', 'mistral-nemo'], endpoint: 'http://localhost:11434', needsKey: false, cloud: false },
  { id: 'openai-compatible', label: 'OpenAI-compatible server', models: [], endpoint: '', needsKey: false, cloud: true },
];

export function providerInfo(id: ProviderId): ProviderInfo | undefined {
  return PROVIDERS.find((p) => p.id === id);
}

export function isProviderId(value: unknown): value is ProviderId {
  return value === 'off' || PROVIDERS.some((p) => p.id === value);
}

/** Settings as read, before the defaults of the provider fill the gaps. */
export interface AiSettings {
  provider: string;
  model: string;
  endpoint: string;
  azureDeployment: string;
  apiVersion: string;
  maxTokens: number;
  temperature: number | null;
  timeoutSeconds: number;
}

/** The configuration a provider is built from: the settings with each provider's defaults. */
export function resolveConfig(settings: AiSettings, apiKey?: string): ProviderConfig {
  const provider: ProviderId = isProviderId(settings.provider) ? settings.provider : 'off';
  const info = providerInfo(provider);
  const model = settings.model.trim() || (provider === 'azure-openai' ? settings.azureDeployment.trim() : info?.models[0] ?? '');
  return {
    provider,
    model,
    endpoint: settings.endpoint.trim() || info?.endpoint || '',
    apiKey: apiKey || undefined,
    azureDeployment: settings.azureDeployment.trim() || undefined,
    apiVersion: settings.apiVersion.trim() || undefined,
    maxTokens: Math.max(256, Math.min(200000, Math.round(settings.maxTokens || 4096))),
    temperature: typeof settings.temperature === 'number' && Number.isFinite(settings.temperature) ? Math.max(0, Math.min(2, settings.temperature)) : null,
    timeoutMs: Math.max(5, Math.min(600, settings.timeoutSeconds || 90)) * 1000,
  };
}

/** Whether requests go to the local machine (no consent asked: nothing leaves it). */
export function isLocalEndpoint(endpoint: string): boolean {
  try {
    return isLocalHost(new URL(endpoint).hostname);
  } catch {
    return false;
  }
}

/** Whether the first request to this configuration must be confirmed. */
export function needsConsent(config: ProviderConfig): boolean {
  if (config.provider === 'off' || config.provider === 'vscode-lm') return false;
  if (config.provider === 'ollama' && (isLocalEndpoint(config.endpoint) || !config.endpoint)) return false;
  return !isLocalEndpoint(config.endpoint);
}

/** What a configuration lacks before anything can be sent; null when complete. */
export function missingPiece(config: ProviderConfig): string | null {
  if (config.provider === 'off') return 'no AI provider is set (arclang.ai.provider is "off")';
  if (config.provider === 'vscode-lm') return null;
  const info = providerInfo(config.provider);
  if (info?.needsKey && !config.apiKey) return `${info.label} needs an API key: run "ArcLang: Set AI API key"`;
  if (!config.model) return `${info?.label ?? config.provider} needs a model (arclang.ai.model${config.provider === 'azure-openai' ? ' or arclang.ai.azureDeployment' : ''})`;
  if (!config.endpoint) return `${info?.label ?? config.provider} needs an endpoint (arclang.ai.endpoint)`;
  const problem = endpointProblem(config.endpoint);
  if (problem) return `${info?.label ?? config.provider}: ${problem}`;
  return null;
}

/** An HTTP provider for the configuration (vscode-lm is built by the extension, it needs VS Code). */
export function createHttpProvider(config: ProviderConfig): ChatProvider {
  const missing = missingPiece(config);
  if (missing) throw new ProviderError(missing, 'config');
  switch (config.provider) {
    case 'anthropic':
      return new AnthropicProvider(config);
    case 'google':
      return new GoogleProvider(config);
    case 'ollama':
      return new OllamaProvider(config);
    case 'openai':
    case 'azure-openai':
    case 'mistral':
    case 'openai-compatible':
      return new OpenAiProvider(config);
    default:
      throw new ProviderError(`the provider ${config.provider} is not an HTTP provider`, 'config');
  }
}

/** "Anthropic Claude · claude-opus-5-5": for the status bar, the log and the consent. */
export function describeConfig(config: ProviderConfig): string {
  if (config.provider === 'off') return 'AI off';
  const info = providerInfo(config.provider);
  return `${info?.label ?? config.provider}${config.model ? ` · ${config.model}` : ''}`;
}
