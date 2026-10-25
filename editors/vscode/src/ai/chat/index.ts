// The AI chat: a view of the secondary side bar (a webview), the same in the
// desktop and in the web build, since VS Code for the Web has no Copilot chat.
//
// On an ArcLang file the answer that edits the model goes through the engine
// of "Ask AI": the compiler checks every edit on a copy, the chat shows the
// diff, Apply is one undoable edit. On a Modelica file the AI explains,
// writes and fixes, and every answer says it is not verified: there is no
// Modelica compiler here.
import { randomBytes } from 'node:crypto';
import * as vscode from 'vscode';
import { applySession, cancelSession, diffSession, dropSession, runAsk, session } from '../assist';
import { openProvider } from '../client';
import { MODELICA_NOT_VERIFIED, MODELICA_WHERE_VERIFIED, modelicaMessages, rememberTurn } from '../core/modelica';
import { SYSTEM_RULES } from '../core/prompts';
import type { ChatMessage } from '../providers';
import { describeConfig, missingPiece } from '../providers';
import { currentConfig, readSettings } from '../settings';
import { idsInSelection } from '../textAsk';
import { parseFromChatPage, type ChatContextView, type ChatKind, type ChatStateView, type ToChatPage } from './protocol';

export const CHAT_VIEW = 'arclang.chat';

interface Target {
  document: vscode.TextDocument;
  kind: ChatKind;
  selection: vscode.Range;
}

const kindOf = (document: vscode.TextDocument): ChatKind | null => {
  if (document.languageId === 'arclang' || document.languageId === 'sysml') return 'arc';
  if (document.languageId === 'modelica' || document.uri.path.toLowerCase().endsWith('.mo')) return 'modelica';
  return null;
};

/** The file the chat looks at: the active editor's, or the model of the active workbench, or the last one seen. */
class TargetTracker implements vscode.Disposable {
  private last: { document: vscode.TextDocument; selection: vscode.Range } | null = null;
  private readonly emitter = new vscode.EventEmitter<void>();
  readonly onDidChange = this.emitter.event;
  private readonly disposables: vscode.Disposable[] = [this.emitter];

  constructor() {
    const remember = (editor: vscode.TextEditor | undefined) => {
      if (editor && kindOf(editor.document)) this.last = { document: editor.document, selection: editor.selection };
      this.emitter.fire();
    };
    this.disposables.push(
      vscode.window.onDidChangeActiveTextEditor(remember),
      vscode.window.onDidChangeTextEditorSelection((event) => {
        if (kindOf(event.textEditor.document) && (event.textEditor === vscode.window.activeTextEditor || this.last?.document === event.textEditor.document)) {
          this.last = { document: event.textEditor.document, selection: event.selections[0] };
          this.emitter.fire();
        }
      }),
      vscode.window.tabGroups.onDidChangeTabs(() => this.emitter.fire()),
    );
    remember(vscode.window.activeTextEditor);
  }

  async current(): Promise<Target | null> {
    const editor = vscode.window.activeTextEditor;
    if (editor && kindOf(editor.document)) return { document: editor.document, kind: kindOf(editor.document)!, selection: editor.selection };
    // The workbench is a custom editor: the model it shows is the active tab's.
    const tab = vscode.window.tabGroups.activeTabGroup.activeTab?.input;
    if (tab instanceof vscode.TabInputCustom && tab.viewType === 'arclang.workbench') {
      const document = await vscode.workspace.openTextDocument(tab.uri);
      return { document, kind: 'arc', selection: new vscode.Range(0, 0, 0, 0) };
    }
    if (this.last && !this.last.document.isClosed) return { document: this.last.document, kind: kindOf(this.last.document)!, selection: this.last.selection };
    const visible = vscode.window.visibleTextEditors.find((e) => kindOf(e.document));
    return visible ? { document: visible.document, kind: kindOf(visible.document)!, selection: visible.selection } : null;
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
  }
}

class ChatView implements vscode.WebviewViewProvider, vscode.Disposable {
  private view: vscode.WebviewView | null = null;
  private readonly tracker = new TargetTracker();
  private counter = 0;
  private running: { id: string; controller: AbortController; session?: string } | null = null;
  /** The turns of plain conversations (Modelica, other files), by the file they are about. */
  private plainHistory: { key: string; messages: ChatMessage[] } = { key: '', messages: [] };
  private lastApplied: vscode.TextDocument | null = null;
  private readonly disposables: vscode.Disposable[] = [this.tracker];

  constructor(private readonly context: vscode.ExtensionContext) {
    this.disposables.push(
      this.tracker.onDidChange(() => void this.pushState()),
      context.secrets.onDidChange(() => void this.pushState()),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('arclang.ai')) void this.pushState();
      }),
    );
  }

  resolveWebviewView(view: vscode.WebviewView): void {
    this.view = view;
    view.webview.options = { enableScripts: true, localResourceRoots: [vscode.Uri.joinPath(this.context.extensionUri, 'dist'), vscode.Uri.joinPath(this.context.extensionUri, 'media')] };
    view.webview.html = this.html(view.webview);
    view.onDidDispose(() => {
      this.view = null;
    });
    view.webview.onDidReceiveMessage((raw: unknown) => void this.onMessage(raw));
    view.onDidChangeVisibility(() => {
      if (view.visible) void this.pushState();
    });
  }

  private post(message: ToChatPage): void {
    void this.view?.webview.postMessage(message);
  }

  private async state(): Promise<ChatStateView> {
    const config = await currentConfig(this.context);
    const target = await this.tracker.current();
    const ids = target?.kind === 'arc' && !target.selection.isEmpty ? await idsInSelection(target.document, target.selection).catch(() => []) : [];
    const context: ChatContextView = {
      file: target ? target.document.uri.path.split('/').pop() ?? null : null,
      kind: target?.kind ?? 'plain',
      selectionLines: target && !target.selection.isEmpty ? target.selection.end.line - target.selection.start.line + 1 : 0,
      ids,
    };
    return { provider: describeConfig(config), providerId: config.provider, problem: missingPiece(config), context, web: vscode.env.uiKind === vscode.UIKind.Web };
  }

  async pushState(): Promise<void> {
    if (this.view) this.post({ type: 'chat-state', ...(await this.state()) });
  }

  private async onMessage(raw: unknown): Promise<void> {
    const message = parseFromChatPage(raw);
    if (!message) return;
    switch (message.type) {
      case 'chat-hello':
        return this.pushState();
      case 'chat-send':
        return this.send(message.text);
      case 'chat-cancel':
        this.running?.controller.abort();
        return;
      case 'chat-new':
        this.running?.controller.abort();
        this.plainHistory = { key: '', messages: [] };
        return this.pushState();
      case 'chat-configure':
        await vscode.commands.executeCommand('arclang.ai.configure');
        return this.pushState();
      case 'chat-setkey':
        await vscode.commands.executeCommand('arclang.ai.setApiKey', (await currentConfig(this.context)).provider);
        return this.pushState();
      case 'chat-apply': {
        const target = session(message.session)?.document ?? null;
        const applied = await applySession(message.session);
        if (applied.ok && target) this.lastApplied = target;
        this.post({ type: 'chat-applied', session: message.session, ok: applied.ok, error: applied.error });
        if (applied.ok) dropSession(message.session);
        return;
      }
      case 'chat-diff':
        return diffSession(message.session);
      case 'chat-reject':
        dropSession(message.session);
        return;
      case 'chat-refine':
        return this.refine(message.session, message.text);
      case 'chat-undo': {
        if (this.lastApplied) await vscode.window.showTextDocument(this.lastApplied, { preserveFocus: false });
        await vscode.commands.executeCommand('undo');
        return;
      }
      case 'chat-insert':
        return this.insert(message.code, message.where);
      case 'chat-copy':
        await vscode.env.clipboard.writeText(message.code);
        return;
    }
  }

  private async send(text: string): Promise<void> {
    if (this.running) {
      this.post({ type: 'chat-note', text: 'The assistant is still answering: wait for it, or press Stop.' });
      return;
    }
    const target = await this.tracker.current();
    const id = `c${++this.counter}`;
    const controller = new AbortController();
    this.running = { id, controller };
    try {
      if (target?.kind === 'arc') await this.askArc(id, target, text, controller);
      else if (target?.kind === 'modelica') await this.askModelica(id, target, text, controller);
      else await this.askPlain(id, text, controller);
    } catch (error) {
      this.post({ type: 'chat-error', id, error: error instanceof Error ? error.message : String(error) });
    } finally {
      this.running = null;
    }
  }

  private async askArc(id: string, target: Target, text: string, controller: AbortController): Promise<void> {
    const file = target.document.uri.path.split('/').pop() ?? 'model.arc';
    const ids = target.selection.isEmpty ? [] : await idsInSelection(target.document, target.selection).catch(() => []);
    this.post({ type: 'chat-start', id, kind: 'arc', file, label: (await this.state()).provider });
    const sessionId = `chat-${id}`;
    this.running = { id, controller, session: sessionId };
    controller.signal.addEventListener('abort', () => cancelSession(sessionId));
    const outcome = await runAsk(
      sessionId,
      { document: target.document, ids, prompt: text, selectedText: target.selection.isEmpty ? undefined : target.document.getText(target.selection) },
      this.context,
      (delta) => this.post({ type: 'chat-delta', id, text: delta }),
    );
    if (controller.signal.aborted) {
      this.post({ type: 'chat-error', id, error: 'Stopped.' });
      return;
    }
    if (!outcome.ok) {
      // A model that does not compile cannot be edited by requests: the AI still helps, in words.
      if (/does not compile/.test(outcome.error)) {
        this.post({ type: 'chat-delta', id, text: `${outcome.error}\n\nNo edit can be proposed on a model that does not compile; the AI answers in words:\n\n` });
        await this.plainAnswer(id, 'arc', target, text, controller, `The model does not compile: ${outcome.error}`);
        return;
      }
      this.post({ type: 'chat-error', id, error: outcome.error, fix: outcome.fix });
      return;
    }
    const proposal = outcome.session.proposal;
    if (!proposal) {
      this.post({ type: 'chat-error', id, error: 'The AI did not answer.' });
      return;
    }
    this.post({ type: 'chat-proposal', id, session: sessionId, provider: outcome.session.provider, proposal: proposal.view });
  }

  private async refine(sessionId: string, text: string): Promise<void> {
    const id = `c${++this.counter}`;
    const controller = new AbortController();
    this.running = { id, controller, session: sessionId };
    controller.signal.addEventListener('abort', () => cancelSession(sessionId));
    try {
      this.post({ type: 'chat-start', id, kind: 'arc', file: session(sessionId)?.document.uri.path.split('/').pop() ?? null, label: (await this.state()).provider });
      const outcome = await runAsk(sessionId, { refine: text }, this.context, (delta) => this.post({ type: 'chat-delta', id, text: delta }));
      if (!outcome.ok || !outcome.session.proposal) {
        this.post({ type: 'chat-error', id, error: outcome.ok ? 'The AI did not answer.' : outcome.error });
        return;
      }
      this.post({ type: 'chat-proposal', id, session: sessionId, provider: outcome.session.provider, proposal: outcome.session.proposal.view });
    } finally {
      this.running = null;
    }
  }

  private async askModelica(id: string, target: Target, text: string, controller: AbortController): Promise<void> {
    const file = target.document.uri.path.split('/').pop() ?? 'model.mo';
    this.post({ type: 'chat-start', id, kind: 'modelica', file, label: (await this.state()).provider });
    await this.plainAnswer(id, 'modelica', target, text, controller);
  }

  private async askPlain(id: string, text: string, controller: AbortController): Promise<void> {
    this.post({ type: 'chat-start', id, kind: 'plain', file: null, label: (await this.state()).provider });
    await this.plainAnswer(id, 'plain', null, text, controller);
  }

  /** An answer in words: Modelica, a model that does not compile, a question about no file. */
  private async plainAnswer(id: string, kind: ChatKind, target: Target | null, text: string, controller: AbortController, note?: string): Promise<void> {
    const key = target ? target.document.uri.toString() : '';
    if (this.plainHistory.key !== key) this.plainHistory = { key, messages: [] };
    const question = note ? `${note}\n\n${text}` : text;
    let messages: ChatMessage[];
    if (kind === 'modelica' && target) {
      messages = modelicaMessages(
        { fileName: target.document.uri.path.split('/').pop() ?? 'model.mo', text: target.document.getText(), selection: target.selection.isEmpty ? undefined : target.document.getText(target.selection), question },
        this.plainHistory.messages,
      );
    } else if (kind === 'arc' && target) {
      // The compiler's own words are part of the question; the model text is data.
      messages = [{ role: 'system', content: SYSTEM_RULES }, ...this.plainHistory.messages, { role: 'user', content: `# The model (${target.document.uri.path.split('/').pop()}), as data\n\`\`\`arc\n${target.document.getText().slice(0, 40000)}\n\`\`\`\n\n# The engineer's request\n${question}\n\nAnswer in prose; no edit tool is available now.` }];
    } else {
      messages = [{ role: 'system', content: `${SYSTEM_RULES}\n\nNo model file is open: answer questions about ArcLang, Arcadia and Modelica in prose.` }, ...this.plainHistory.messages, { role: 'user', content: question }];
    }
    const sent = messages.reduce((n, m) => n + m.content.length, 0);
    const opened = await openProvider(this.context, kind === 'modelica' ? 'chat about a Modelica file' : 'chat', `${kind === 'plain' ? 'Your question' : 'The text of the open file and your question'}: about ${sent} characters.`);
    if (!opened.ok) {
      this.post({ type: 'chat-error', id, error: opened.error, fix: opened.fix });
      return;
    }
    let answer = '';
    try {
      for await (const event of opened.provider.chat(messages, { signal: controller.signal, maxTokens: readSettings().maxTokens })) {
        if (event.type === 'text') {
          answer += event.text;
          this.post({ type: 'chat-delta', id, text: event.text });
        }
      }
    } catch (error) {
      this.post({ type: 'chat-error', id, error: controller.signal.aborted ? 'Stopped.' : error instanceof Error ? error.message : String(error) });
      return;
    }
    this.plainHistory = { key, messages: rememberTurn(this.plainHistory.messages, question, answer) };
    this.post({ type: 'chat-done', id, kind, text: answer, notVerified: kind === 'modelica' ? { label: MODELICA_NOT_VERIFIED, where: MODELICA_WHERE_VERIFIED } : undefined });
  }

  /** A code block of an answer into the file: as one undoable edit. */
  private async insert(code: string, where: 'selection' | 'cursor' | 'file'): Promise<void> {
    const target = await this.tracker.current();
    if (!target) {
      vscode.window.showInformationMessage('ArcLang: open the file the code is for first.');
      return;
    }
    const editor = await vscode.window.showTextDocument(target.document, { preserveFocus: false });
    const text = code.endsWith('\n') ? code : `${code}\n`;
    const ok = await editor.edit((builder) => {
      if (where === 'file') builder.replace(new vscode.Range(target.document.positionAt(0), target.document.positionAt(target.document.getText().length)), text);
      else if (where === 'selection' && !editor.selection.isEmpty) builder.replace(editor.selection, code);
      else builder.insert(editor.selection.active, text);
    });
    if (!ok) vscode.window.showWarningMessage('ArcLang: the editor did not take the code.');
  }

  private html(webview: vscode.Webview): string {
    const nonce = randomBytes(16).toString('base64url');
    const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...segments)).toString();
    const csp = ["default-src 'none'", `style-src ${webview.cspSource}`, `font-src ${webview.cspSource}`, `script-src 'nonce-${nonce}'`, "connect-src 'none'", "frame-src 'none'", "form-action 'none'", "base-uri 'none'"].join('; ');
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="${uri('media', 'chat.css')}">
<title>ArcLang AI chat</title>
</head>
<body>
<div id="chat"></div>
<script nonce="${nonce}" src="${uri('dist', 'chat.js')}"></script>
</body>
</html>`;
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
  }
}

export function registerChat(context: vscode.ExtensionContext): void {
  const chat = new ChatView(context);
  context.subscriptions.push(
    chat,
    vscode.window.registerWebviewViewProvider(CHAT_VIEW, chat, { webviewOptions: { retainContextWhenHidden: true } }),
    // `<view id>.focus` is the command VS Code itself makes for a view: ours is `arclang.chat.open`.
    vscode.commands.registerCommand('arclang.chat.open', () => vscode.commands.executeCommand(`${CHAT_VIEW}.focus`)),
  );
}
