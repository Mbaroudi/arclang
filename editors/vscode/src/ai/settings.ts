// The AI settings (arclang.ai.*), the keys in VS Code's secret storage, the
// first-use consent, and the "ArcLang AI" log. A key is never written to a
// setting, a log or a message.
import * as vscode from 'vscode';
import type { ContextScope } from './core/context';
import { AiSettings, describeConfig, displayUrl, needsConsent, ProviderConfig, ProviderId, providerInfo, redact, resolveConfig } from './providers';

export type SuggestMode = 'off' | 'onRequest' | 'afterEachEdit';

const SECTION = 'arclang.ai';

export function readSettings(): AiSettings & { vscodeModel: string } {
  const config = vscode.workspace.getConfiguration(SECTION);
  return {
    provider: config.get<string>('provider', 'vscode-lm'),
    model: config.get<string>('model', ''),
    endpoint: config.get<string>('endpoint', ''),
    azureDeployment: config.get<string>('azureDeployment', ''),
    apiVersion: config.get<string>('apiVersion', ''),
    hostedEndpoint: config.get<string>('hostedEndpoint', ''),
    maxTokens: config.get<number>('maxTokens', 4096),
    temperature: config.get<number | null>('temperature', null),
    timeoutSeconds: config.get<number>('timeoutSeconds', 90),
    vscodeModel: config.get<string>('vscodeModel', ''),
  };
}

export function suggestMode(): SuggestMode {
  const value = vscode.workspace.getConfiguration(SECTION).get<string>('suggestOnEdit', 'onRequest');
  return value === 'off' || value === 'afterEachEdit' ? value : 'onRequest';
}

export function contextScope(): ContextScope {
  const value = vscode.workspace.getConfiguration(SECTION).get<string>('contextScope', 'selectionAndNeighbours');
  return value === 'selection' || value === 'wholeModel' ? value : 'selectionAndNeighbours';
}

export function repairAttempts(): number {
  return Math.max(0, Math.min(5, vscode.workspace.getConfiguration(SECTION).get<number>('repairAttempts', 2)));
}

const keyName = (provider: ProviderId) => `arclang.ai.apiKey.${provider}`;

export async function apiKey(context: vscode.ExtensionContext, provider: ProviderId): Promise<string | undefined> {
  return (await context.secrets.get(keyName(provider))) || undefined;
}

/** The configuration as set, the key of its provider read from the secret storage. */
export async function currentConfig(context: vscode.ExtensionContext): Promise<ProviderConfig> {
  const settings = readSettings();
  const base = resolveConfig(settings);
  const key = base.provider === 'off' || base.provider === 'vscode-lm' ? undefined : await apiKey(context, base.provider);
  const config = resolveConfig(settings, key);
  return config.provider === 'vscode-lm' ? { ...config, model: settings.vscodeModel || settings.model, endpoint: 'vscode.lm' } : config;
}

// --- The log ------------------------------------------------------------------

let channel: vscode.OutputChannel | null = null;

export function aiLog(): vscode.OutputChannel {
  if (!channel) channel = vscode.window.createOutputChannel('ArcLang AI');
  return channel;
}

/** One line in the AI log, time-stamped, with anything shaped like a key removed. */
export function logLine(text: string, secrets: (string | undefined)[] = []): void {
  aiLog().appendLine(`[${new Date().toISOString().slice(11, 19)}] ${redact(text, secrets)}`);
}

// --- Consent --------------------------------------------------------------------

const CONSENT_KEY = 'arclang.ai.consented';

function consentId(config: ProviderConfig): string {
  return `${config.provider}|${config.endpoint}`;
}

/** Refusals of this session (per provider and endpoint): automatic requests do not ask again. */
const refusedInSession = new Set<string>();

/**
 * The first time the model goes to a provider (per provider and endpoint),
 * the user says yes: what is sent, to whom. Local providers are not asked.
 * Once refused, an `automatic` request (suggestions after each edit) is not
 * sent and does not ask again until VS Code restarts; a request the user
 * makes asks again.
 */
export async function ensureConsent(context: Pick<vscode.ExtensionContext, 'globalState'>, config: ProviderConfig, what: string, options: { automatic?: boolean } = {}): Promise<boolean> {
  const ask = vscode.workspace.getConfiguration(SECTION).get<boolean>('confirmBeforeSending', true);
  if (!ask || !needsConsent(config)) return true;
  const given = context.globalState.get<string[]>(CONSENT_KEY, []);
  const id = consentId(config);
  if (given.includes(id)) return true;
  const label = describeConfig(config);
  if (options.automatic && refusedInSession.has(id)) {
    logLine(`not sent to ${label}: the consent was refused in this session (suggestions are proposed on request only)`);
    return false;
  }
  const choice = await vscode.window.showWarningMessage(
    `ArcLang AI: send model content to ${label}?`,
    { modal: true, detail: `${what}\n\nIt goes to ${displayUrl(config.endpoint)}. Keys stay in VS Code's secret storage; each request is listed in the "ArcLang AI" output. Nothing is applied to the model without your Apply.` },
    'Send',
    'Send and do not ask again',
  );
  if (choice === 'Send and do not ask again') await context.globalState.update(CONSENT_KEY, [...given, id]);
  logLine(`consent for ${label} at ${displayUrl(config.endpoint)}: ${choice ?? 'refused'}`);
  const granted = choice === 'Send' || choice === 'Send and do not ask again';
  if (granted) refusedInSession.delete(id);
  else refusedInSession.add(id);
  return granted;
}

// --- Commands: keys, test, quick configuration ------------------------------------

const HTTP_PROVIDERS: ProviderId[] = ['anthropic', 'openai', 'azure-openai', 'mistral', 'google', 'ollama', 'openai-compatible'];

/** The providers a visitor can pick: no VS Code language models in a browser (no Copilot there). */
function providerChoices(): ProviderId[] { // web hook
  const web = vscode.env.uiKind === vscode.UIKind.Web;
  return [...(web ? [] : (['vscode-lm'] as ProviderId[])), 'arclang-hosted', ...HTTP_PROVIDERS, 'off'];
}

/**
 * In the web playground the page's content security policy lets the browser
 * reach ONE provider: the one the visitor configured. The page remembers it
 * (a cookie the server reads when it serves the page again) when this command
 * is there; on the desktop there is nothing to tell.
 */
export async function allowProviderInBrowser(provider: ProviderId): Promise<void> { // web hook
  if (vscode.env.uiKind !== vscode.UIKind.Web) return;
  try {
    await vscode.commands.executeCommand('arclangWeb.allowProvider', provider);
  } catch {
    // not the playground page: no such command
  }
}

async function pickProvider(placeHolder: string, only: ProviderId[] = HTTP_PROVIDERS): Promise<ProviderId | undefined> {
  const current = resolveConfig(readSettings()).provider;
  const items = only.map((id) => ({ label: providerInfo(id)?.label ?? id, description: id === current ? 'current' : id, id }));
  items.sort((a, b) => (a.id === current ? -1 : b.id === current ? 1 : 0));
  return (await vscode.window.showQuickPick(items, { placeHolder }))?.id;
}

export async function setApiKey(context: vscode.ExtensionContext, providerArg?: unknown): Promise<void> {
  const provider = typeof providerArg === 'string' && HTTP_PROVIDERS.includes(providerArg as ProviderId) ? (providerArg as ProviderId) : await pickProvider('The provider whose API key to store');
  if (!provider) return;
  const info = providerInfo(provider);
  const key = await vscode.window.showInputBox({
    title: `ArcLang: API key for ${info?.label ?? provider}`,
    prompt: `Stored in VS Code's secret storage (the OS keychain), never in settings.${info?.keyHelp ? ` Get one at ${info.keyHelp}.` : ''}`,
    password: true,
    ignoreFocusOut: true,
    validateInput: (value) => (value.trim().length < 8 ? 'This does not look like an API key.' : /\s/.test(value.trim()) ? 'An API key has no spaces.' : null),
  });
  if (!key) return;
  await context.secrets.store(keyName(provider), key.trim());
  logLine(`API key stored for ${info?.label ?? provider}`);
  await allowProviderInBrowser(provider); // web hook
  vscode.window.showInformationMessage(`ArcLang: the API key for ${info?.label ?? provider} is stored in the secret storage.`);
}

export async function clearApiKey(context: vscode.ExtensionContext): Promise<void> {
  const provider = await pickProvider('The provider whose API key to delete');
  if (!provider) return;
  await context.secrets.delete(keyName(provider));
  logLine(`API key deleted for ${providerInfo(provider)?.label ?? provider}`);
  vscode.window.showInformationMessage(`ArcLang: the API key for ${providerInfo(provider)?.label ?? provider} is deleted.`);
}

async function update(key: string, value: unknown): Promise<void> {
  await vscode.workspace.getConfiguration(SECTION).update(key, value, vscode.ConfigurationTarget.Global);
}

/** The quick configuration of the status bar item. */
export async function configure(context: vscode.ExtensionContext): Promise<void> {
  const config = await currentConfig(context);
  const info = providerInfo(config.provider);
  type Item = vscode.QuickPickItem & { run?: () => Thenable<unknown> | Promise<unknown> };
  const items: Item[] = [
    { label: '$(server) Provider', description: config.provider === 'off' ? 'off' : (info?.label ?? config.provider), run: async () => {
      const id = await pickProvider('The AI provider of ArcLang', providerChoices());
      if (!id) return;
      await update('provider', id);
      await update('model', '');
      await allowProviderInBrowser(id); // web hook
      const chosen = providerInfo(id);
      if (chosen?.needsKey && !(await apiKey(context, id))) await setApiKey(context, id);
      if ((id === 'openai-compatible' || id === 'azure-openai') && !readSettings().endpoint) await vscode.commands.executeCommand('workbench.action.openSettings', 'arclang.ai.endpoint');
    } },
    { label: '$(symbol-class) Model', description: config.model || '(default)', run: () => pickModel(context) },
  ];
  if (config.provider !== 'vscode-lm' && config.provider !== 'off' && config.provider !== 'arclang-hosted') {
    items.push({ label: '$(link) Endpoint', description: (config.endpoint && displayUrl(config.endpoint)) || '(none)', run: () => vscode.commands.executeCommand('workbench.action.openSettings', 'arclang.ai.endpoint') });
    items.push({ label: '$(key) Set API key', description: config.apiKey ? 'stored' : info?.needsKey ? 'missing' : 'optional', run: () => setApiKey(context, config.provider) });
  }
  items.push(
    { label: '$(pulse) Test AI connection', run: () => vscode.commands.executeCommand('arclang.ai.testConnection') },
    { label: '$(lightbulb) Suggestions while editing', description: suggestMode(), run: async () => {
      const mode = await vscode.window.showQuickPick(['off', 'onRequest', 'afterEachEdit'].map((m) => ({ label: m, description: m === suggestMode() ? 'current' : m === 'onRequest' ? 'the consequences are shown; the AI is asked on a click' : m === 'afterEachEdit' ? 'the AI proposes after each edit with consequences' : 'nothing' })), { placeHolder: 'Impact-driven AI suggestions' });
      if (mode) await update('suggestOnEdit', mode.label);
    } },
    { label: '$(selection) Context sent', description: contextScope(), run: async () => {
      const scope = await vscode.window.showQuickPick(['selection', 'selectionAndNeighbours', 'wholeModel'], { placeHolder: 'What the AI is shown around a selection' });
      if (scope) await update('contextScope', scope);
    } },
    { label: '$(output) Show the AI log', run: async () => aiLog().show(true) },
    { label: '$(gear) All AI settings', run: () => vscode.commands.executeCommand('workbench.action.openSettings', 'arclang.ai') },
  );
  const choice = await vscode.window.showQuickPick(items, { placeHolder: `ArcLang AI: ${describeConfig(config)}` });
  await choice?.run?.();
}

export async function pickModel(context: vscode.ExtensionContext): Promise<void> {
  const config = await currentConfig(context);
  if (config.provider === 'vscode-lm') {
    const models = await vscode.lm.selectChatModels();
    if (models.length === 0) {
      vscode.window.showInformationMessage('ArcLang: VS Code offers no language model (sign in to GitHub Copilot, or choose another provider).');
      return;
    }
    const pick = await vscode.window.showQuickPick(models.map((m) => ({ label: m.name, description: `${m.vendor} · ${m.family}`, detail: m.id, id: m.id })), { placeHolder: 'The VS Code language model ArcLang uses' });
    if (pick) await update('vscodeModel', pick.id);
    return;
  }
  const known = providerInfo(config.provider)?.models ?? [];
  const items = [...known.map((m) => ({ label: m, description: m === config.model ? 'current' : '' })), { label: '$(edit) Other…', description: 'type a model name' }];
  const pick = await vscode.window.showQuickPick(items, { placeHolder: `The model of ${providerInfo(config.provider)?.label ?? config.provider}` });
  if (!pick) return;
  if (pick.label.startsWith('$(edit)')) {
    const name = await vscode.window.showInputBox({ prompt: 'Model name, as the provider spells it', value: config.model });
    if (name !== undefined) await update('model', name.trim());
    return;
  }
  await update('model', pick.label);
}
