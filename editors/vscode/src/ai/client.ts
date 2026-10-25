// The provider the settings name, ready to use: the key from the secret
// storage, the consent asked once, every request written in the log
// (provider, model, endpoint, what was sent, how long it took; never a key).
import * as vscode from 'vscode';
import { createHttpProvider, describeConfig, missingPiece, ProviderError, redact, ChatEvent, ChatMessage, ChatOptions, ChatProvider, ProviderConfig } from './providers';
import { selectVscodeModel, VscodeLmProvider } from './providers/vscodeLm';
import { currentConfig, ensureConsent, logLine } from './settings';

export type Opened = { ok: true; provider: ChatProvider; config: ProviderConfig; label: string } | { ok: false; error: string; fix?: 'key' | 'configure' };

class LoggedProvider implements ChatProvider {
  constructor(
    private readonly inner: ChatProvider,
    private readonly purpose: string,
    private readonly secrets: (string | undefined)[],
  ) {}

  get id() {
    return this.inner.id;
  }
  get model() {
    return this.inner.model;
  }
  get endpoint() {
    return this.inner.endpoint;
  }

  async *chat(messages: ChatMessage[], options: ChatOptions = {}): AsyncGenerator<ChatEvent> {
    const chars = messages.reduce((n, m) => n + m.content.length + JSON.stringify(m.toolCalls ?? []).length, 0);
    const started = Date.now();
    logLine(`→ ${this.inner.id} ${this.inner.model} at ${this.inner.endpoint}: ${this.purpose}; ${messages.length} messages, ${chars} characters${options.tools?.length ? `, tools ${options.tools.map((t) => t.name).join(', ')}` : ''}`, this.secrets);
    let received = 0;
    const calls: string[] = [];
    try {
      for await (const event of this.inner.chat(messages, options)) {
        if (event.type === 'text') received += event.text.length;
        if (event.type === 'tool_call') calls.push(event.call.name);
        if (event.type === 'done') logLine(`← ${this.inner.id}: ${received} characters${calls.length ? `, tool calls ${calls.join(', ')}` : ''}${event.stopReason ? `, stop ${event.stopReason}` : ''}${event.usage?.input !== undefined ? `, tokens ${event.usage.input} in / ${event.usage.output ?? '?'} out` : ''}, ${Date.now() - started} ms`, this.secrets);
        yield event;
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      logLine(`✗ ${this.inner.id}: ${message} (${Date.now() - started} ms)`, this.secrets);
      throw new ProviderError(redact(message, this.secrets), error instanceof ProviderError ? error.kind : 'network', error instanceof ProviderError ? error.status : undefined);
    }
  }
}

/**
 * The configured provider for one purpose, or why there is none. Nothing is
 * sent when the provider is off, a key is missing, or the consent is refused.
 */
export async function openProvider(context: vscode.ExtensionContext, purpose: string, what: string, options: { automatic?: boolean } = {}): Promise<Opened> {
  const config = await currentConfig(context);
  const missing = missingPiece(config);
  if (missing) return { ok: false, error: missing, fix: /API key/.test(missing) ? 'key' : 'configure' };
  let provider: ChatProvider;
  try {
    if (config.provider === 'vscode-lm') {
      const model = await selectVscodeModel(config.model);
      if (!model) return { ok: false, error: config.model ? `VS Code offers no language model named ${config.model}` : 'VS Code offers no language model: sign in to GitHub Copilot, or choose another provider (ArcLang: Configure AI)', fix: 'configure' };
      provider = new VscodeLmProvider(model);
    } else {
      provider = createHttpProvider(config);
    }
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : String(error), fix: 'configure' };
  }
  if (!(await ensureConsent(context, config, what, options))) return { ok: false, error: 'not sent: the consent was refused' };
  const label = config.provider === 'vscode-lm' ? `VS Code · ${provider.model}` : describeConfig(config);
  return { ok: true, provider: new LoggedProvider(provider, purpose, [config.apiKey]), config, label };
}

/** A failure for the user, with a button that leads to the fix. */
export async function showOpenFailure(failure: Extract<Opened, { ok: false }>): Promise<void> {
  const action = failure.fix === 'key' ? 'Set API key' : failure.fix === 'configure' ? 'Configure AI' : undefined;
  const choice = await vscode.window.showWarningMessage(`ArcLang AI: ${failure.error}`, ...(action ? [action] : []));
  if (choice === 'Set API key') await vscode.commands.executeCommand('arclang.ai.setApiKey');
  if (choice === 'Configure AI') await vscode.commands.executeCommand('arclang.ai.configure');
}

/** "Test AI connection": a few tokens, the answer and the time it took. */
export async function testConnection(context: vscode.ExtensionContext): Promise<void> {
  const opened = await openProvider(context, 'connection test', 'Only the words "Reply with the single word OK." are sent.');
  if (!opened.ok) return showOpenFailure(opened);
  const started = Date.now();
  await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `ArcLang: testing ${opened.label}`, cancellable: true }, async (_progress, token) => {
    const controller = new AbortController();
    token.onCancellationRequested(() => controller.abort());
    try {
      let text = '';
      for await (const event of opened.provider.chat([{ role: 'user', content: 'Reply with the single word OK.' }], { signal: controller.signal, maxTokens: 256 })) if (event.type === 'text') text += event.text;
      vscode.window.showInformationMessage(`ArcLang AI: ${opened.label} answered "${text.trim().slice(0, 60)}" in ${Date.now() - started} ms (${opened.provider.endpoint}).`);
    } catch (error) {
      vscode.window.showErrorMessage(`ArcLang AI: ${opened.label} failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  });
}
