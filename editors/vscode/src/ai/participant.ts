// The @arclang chat participant: the model talks, the compiler decides.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run, withTempFile } from '../cli';
import { clip, extractArcFence, parseCheckReport, renderTransitionPlan, summarizeCheck, TransitionPlan } from '../pure/arc';
import { documentExcerpt, HOUSE_RULES, metamodelReference, sampleModel } from './context';

interface ArcChatResult extends vscode.ChatResult {
  metadata: { command: string; hasModel: boolean; compiles: boolean };
}

interface PendingModel {
  text: string;
  uri?: vscode.Uri;
}

const pendingModels = new Map<string, PendingModel>();
let pendingCounter = 0;

const User = vscode.LanguageModelChatMessage.User;
const Assistant = vscode.LanguageModelChatMessage.Assistant;

/** The model the request is about: the active .arc editor, else a .arc reference of the prompt. */
async function targetDocument(request: vscode.ChatRequest): Promise<vscode.TextDocument | undefined> {
  const editor = vscode.window.activeTextEditor;
  if (editor && editor.document.languageId === 'arclang') return editor.document;
  for (const reference of request.references) {
    const value = reference.value;
    const uri = value instanceof vscode.Uri ? value : value instanceof vscode.Location ? value.uri : undefined;
    if (uri && uri.fsPath.endsWith('.arc')) return vscode.workspace.openTextDocument(uri);
  }
  return vscode.workspace.textDocuments.find((d) => d.languageId === 'arclang');
}

function diagnosticsText(document: vscode.TextDocument): string {
  const diagnostics = vscode.languages.getDiagnostics(document.uri);
  if (diagnostics.length === 0) return 'The language server reports no diagnostic on it.';
  const lines = diagnostics.slice(0, 80).map((d) => {
    const severity = ['error', 'warning', 'info', 'hint'][d.severity] ?? 'info';
    const code = typeof d.code === 'object' && d.code ? String(d.code.value) : d.code ? String(d.code) : '';
    return `- line ${d.range.start.line + 1} ${severity}${code ? ` [${code}]` : ''}: ${d.message}`;
  });
  return `Diagnostics of the language server (${diagnostics.length}):\n${lines.join('\n')}`;
}

function historyMessages(context: vscode.ChatContext): vscode.LanguageModelChatMessage[] {
  const messages: vscode.LanguageModelChatMessage[] = [];
  for (const turn of context.history.slice(-10)) {
    if (turn instanceof vscode.ChatRequestTurn) {
      messages.push(User(`${turn.command ? `/${turn.command} ` : ''}${turn.prompt}`));
    } else if (turn instanceof vscode.ChatResponseTurn) {
      const text = turn.response
        .filter((part): part is vscode.ChatResponseMarkdownPart => part instanceof vscode.ChatResponseMarkdownPart)
        .map((part) => part.value.value)
        .join('');
      if (text) messages.push(Assistant(clip(text, 12000)));
    }
  }
  return messages;
}

const INSTRUCTIONS: Record<string, string> = {
  model:
    'Write the model the user asks for. When a model is open, extend it and keep everything it already has. Structure the answer as: "Assumptions:" (2 to 4 lines), then the WHOLE model in one ```arc fence, then "Refine next:" with 3 numbered questions whose answers would deepen the model. Every requirement gets a trace that satisfies it and a test case that verifies it; every logical component is deployed on a physical node; hazards carry severity, exposure, controllability and an ASIL consistent with them.',
  fix: 'Fix the compiler errors and the findings listed, minimally: do not redesign, keep ids, names, comments and order. Return the WHOLE corrected file in one ```arc fence, then list what changed, one line per change, each naming the rule or error it answers.',
  explain: 'Explain what the user asks about the open model, from its text and the compiler diagnostics. When the question concerns one element, call arclang_impact for it before answering. Do not propose a rewrite unless asked.',
  review: 'Review the model as a design reviewer would: requirements coverage, traces, safety (hazards and ASIL), timing budgets, interfaces and deployment. Use only the compiler findings and the gate report below as facts; for each problem, propose a precise change (element, attribute, value). Do not rewrite the model.',
  transition: 'From the transition plan below, say what each step would create on the next Arcadia level and recommend the next step to apply. The user applies a step with the buttons under this answer; do not rewrite the model.',
  chat: 'Help with the request. When a fact about a model is needed (does it compile, what depends on an element, which attributes a kind has, which ids exist), call the arclang tools instead of guessing: arclang_model lists the elements with their exact ids. To change an open model, prefer arclang_edit with precise requests (add, set, move, trace, rename, remove) over rewriting the file; the compiler refuses what would not compile and the user confirms each gesture. Return a WHOLE file in one ```arc fence only when the model does not exist yet or must be rewritten.',
  agent:
    'Act as the ArcLang modelling agent on the open model, step by step: (1) read it with arclang_model (the whole model, then the layer or kind concerned) so you know the exact ids; (2) say in two or three lines what you will change and why; (3) apply the changes with arclang_edit, grouping the requests of one intention in one call (ports before the exchange that uses them, elements before the traces to them); (4) after each call, read the answer: a refusal names the request and the reason, fix that request and retry at most twice; (5) finish with arclang_check and, when requirements are involved, arclang_gate, and report what the compiler says, citing findings by their rule. Never claim a change was made unless an arclang_edit answer says it was applied. Do not rewrite the file.',
};

/** Streams one model answer, running the tools it calls, until it stops calling tools. */
async function converse(
  model: vscode.LanguageModelChat,
  messages: vscode.LanguageModelChatMessage[],
  tools: vscode.LanguageModelChatTool[],
  stream: vscode.ChatResponseStream,
  token: vscode.CancellationToken,
  toolInvocationToken: vscode.ChatParticipantToolToken,
): Promise<string> {
  let text = '';
  for (let round = 0; round < 8; round++) {
    const response = await model.sendRequest(messages, { tools, justification: 'ArcLang modelling with the compiler at hand' }, token);
    const calls: vscode.LanguageModelToolCallPart[] = [];
    let roundText = '';
    for await (const part of response.stream) {
      if (part instanceof vscode.LanguageModelTextPart) {
        roundText += part.value;
        stream.markdown(part.value);
      } else if (part instanceof vscode.LanguageModelToolCallPart) {
        calls.push(part);
      }
    }
    text += roundText;
    if (calls.length === 0) break;
    messages.push(Assistant([...(roundText ? [new vscode.LanguageModelTextPart(roundText)] : []), ...calls]));
    const results: vscode.LanguageModelToolResultPart[] = [];
    for (const call of calls) {
      stream.progress(`Running ${call.name}`);
      let content: unknown[];
      try {
        content = (await vscode.lm.invokeTool(call.name, { input: call.input, toolInvocationToken }, token)).content;
      } catch (error) {
        content = [new vscode.LanguageModelTextPart(`${call.name} failed: ${error instanceof Error ? error.message : String(error)}`)];
      }
      results.push(new vscode.LanguageModelToolResultPart(call.callId, content));
    }
    messages.push(User(results));
    stream.markdown('\n\n');
  }
  return text;
}

/** Compiles a model the assistant wrote; asks for corrections while the compiler refuses it. */
async function verifyAndOffer(
  fence: string,
  answerText: string,
  target: vscode.TextDocument | undefined,
  model: vscode.LanguageModelChat,
  messages: vscode.LanguageModelChatMessage[],
  stream: vscode.ChatResponseStream,
  token: vscode.CancellationToken,
  toolInvocationToken: vscode.ChatParticipantToolToken,
): Promise<boolean> {
  const maxRepairs = vscode.workspace.getConfiguration('arclang').get<number>('ai.repairAttempts', 2);
  let text = fence;
  let lastAnswer = answerText;
  for (let attempt = 0; ; attempt++) {
    stream.progress('Compiling the model with arclang');
    const result = await withTempFile(text, 'model.arc', (tmp, dir) => run(['check', '--json', tmp], { cwd: dir, token }));
    const parsed = parseCheckReport(result.stdout);
    const compiles = result.code !== 127 && parsed !== null && parsed.errors.length === 0 && (result.code === 0 || parsed.findings.length > 0);
    if (compiles && parsed) {
      const findings = parsed.findings.filter((f) => f.severity !== 'info');
      stream.markdown(`\n\n**Compiled by arclang.** ${findings.length === 0 ? 'No finding.' : `${findings.length} finding${findings.length > 1 ? 's' : ''}:`}\n`);
      if (findings.length > 0) stream.markdown('```\n' + clip(summarizeCheck({ errors: [], findings }), 4000) + '\n```\n');
      offer(text, target, stream);
      return true;
    }
    const errors = result.code === 127 ? failureText(result) : parsed ? parsed.errors.join('\n') : failureText(result);
    if (result.code === 127 || attempt >= maxRepairs) {
      stream.markdown(`\n\n**The compiler refuses this model**${attempt > 0 ? ` after ${attempt} correction${attempt > 1 ? 's' : ''}` : ''}:\n\`\`\`\n${clip(errors, 3000)}\n\`\`\`\n`);
      offer(text, undefined, stream);
      return false;
    }
    messages.push(Assistant(lastAnswer));
    messages.push(User(`The arclang compiler refuses this model:\n${clip(errors, 6000)}\n\nReturn the WHOLE corrected model in one \`\`\`arc fence, changing only what these errors need. No commentary before the fence.`));
    stream.markdown(`\n\n**Correction ${attempt + 1}** (the compiler said: ${errors.split('\n')[0]})\n\n`);
    lastAnswer = await converse(model, messages, [], stream, token, toolInvocationToken);
    const corrected = extractArcFence(lastAnswer);
    if (!corrected) {
      stream.markdown('\n\nThe correction holds no model.\n');
      return false;
    }
    text = corrected;
  }
}

function offer(text: string, target: vscode.TextDocument | undefined, stream: vscode.ChatResponseStream): void {
  const id = `m${++pendingCounter}`;
  pendingModels.set(id, { text, uri: target?.uri });
  if (target) stream.button({ title: `Replace ${path.basename(target.uri.fsPath)} in the editor`, command: 'arclang.ai.apply', arguments: [id, 'replace'] });
  stream.button({ title: 'Open as a new file', command: 'arclang.ai.apply', arguments: [id, 'new'] });
}

async function applyPending(id: string, mode: 'replace' | 'new'): Promise<void> {
  const pending = pendingModels.get(id);
  if (!pending) {
    vscode.window.showInformationMessage('ArcLang: this model is no longer available; ask @arclang again.');
    return;
  }
  if (mode === 'replace' && pending.uri) {
    const document = await vscode.workspace.openTextDocument(pending.uri);
    const edit = new vscode.WorkspaceEdit();
    edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), pending.text);
    await vscode.workspace.applyEdit(edit);
    await vscode.window.showTextDocument(document, { preview: false });
    return;
  }
  const document = await vscode.workspace.openTextDocument({ language: 'arclang', content: pending.text });
  await vscode.window.showTextDocument(document, { preview: false });
}

function handler(context: vscode.ExtensionContext): vscode.ChatRequestHandler {
  return async (request, chatContext, stream, token): Promise<ArcChatResult> => {
    const command = request.command ?? 'chat';
    const target = await targetDocument(request);
    const messages: vscode.LanguageModelChatMessage[] = [];
    const reference = await metamodelReference();
    const sample = sampleModel(context);
    messages.push(User(`${HOUSE_RULES}\n\n# ArcLang reference\n${reference}${sample ? `\n# A sample model that compiles\n\`\`\`arc\n${sample}\`\`\`` : ''}`));
    messages.push(...historyMessages(chatContext));
    if (target) {
      messages.push(User(`# The open model: ${path.basename(target.uri.fsPath)}\n\`\`\`arc\n${documentExcerpt(target)}\n\`\`\`\n${diagnosticsText(target)}`));
    } else if (command === 'fix' || command === 'review' || command === 'transition' || command === 'explain') {
      stream.markdown('Open a `.arc` model first, or attach one to the request with #file.');
      return { metadata: { command, hasModel: false, compiles: false } };
    }

    let plans: TransitionPlan[] | null = null;
    if (target && !target.isUntitled && (command === 'review' || command === 'transition')) {
      const file = target.uri.fsPath;
      if (target.isDirty) await target.save();
      if (command === 'review') {
        stream.progress('Running arclang check and the production gate');
        const check = await run(['check', '--json', file], { cwd: path.dirname(file), token });
        const standard = vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
        const gate = await run(['gate', '--standard', standard, file], { cwd: path.dirname(file), token });
        const parsed = parseCheckReport(check.stdout);
        messages.push(User(`# arclang check\n${parsed ? summarizeCheck(parsed) : failureText(check)}\n\n# arclang gate --standard ${standard} (${gate.code === 0 ? 'PASS' : 'FAIL'})\n${clip(gate.stdout + gate.stderr, 12000)}`));
      } else {
        stream.progress('Planning the transitions');
        const plan = await run(['transition', file, '--plan'], { cwd: path.dirname(file), token });
        try {
          plans = JSON.parse(plan.stdout) as TransitionPlan[];
          messages.push(User(`# arclang transition --plan\n${renderTransitionPlan(plans, path.basename(file))}`));
        } catch {
          messages.push(User(`# arclang transition --plan failed\n${failureText(plan)}`));
        }
      }
    }

    messages.push(User(`${INSTRUCTIONS[command] ?? INSTRUCTIONS.chat}\n\nRequest: ${request.prompt || '(no text)'}`));

    const wanted = new Set(request.toolReferences.map((r) => r.name));
    const tools: vscode.LanguageModelChatTool[] = vscode.lm.tools
      .filter((tool) => (tool.name.startsWith('arclang_') && command !== 'model' && command !== 'fix') || wanted.has(tool.name))
      .map((tool) => ({ name: tool.name, description: tool.description, inputSchema: tool.inputSchema }));

    const answer = await converse(request.model, messages, tools, stream, token, request.toolInvocationToken);
    const fence = extractArcFence(answer);
    let compiles = false;
    if (fence && (command === 'model' || command === 'fix' || command === 'chat')) {
      compiles = await verifyAndOffer(fence, answer, target, request.model, messages, stream, token, request.toolInvocationToken);
    }
    if (command === 'transition' && plans && target) {
      for (const plan of plans) {
        if (plan.candidates.some((c) => c.realized_by.length === 0) && !plan.blocked) {
          stream.button({ title: `Apply ${plan.step}`, command: 'arclang.transitionApply', arguments: [target.uri, plan.step] });
        }
      }
    }
    return { metadata: { command, hasModel: fence !== null, compiles } };
  };
}

export function registerParticipant(context: vscode.ExtensionContext): void {
  const participant = vscode.chat.createChatParticipant('arclang.assistant', handler(context));
  participant.iconPath = vscode.Uri.joinPath(context.extensionUri, 'media', 'icon.svg');
  participant.followupProvider = {
    provideFollowups(result: vscode.ChatResult) {
      const metadata = (result as ArcChatResult).metadata;
      if (!metadata) return [];
      const followups: vscode.ChatFollowup[] = [];
      if (metadata.hasModel && !metadata.compiles) followups.push({ prompt: 'Fix the compiler errors', command: 'fix', label: 'Fix the errors' });
      if (metadata.hasModel && metadata.compiles) followups.push({ prompt: 'Review this model', command: 'review', label: 'Review the model' });
      if (metadata.command === 'review') followups.push({ prompt: 'Fix the findings you listed', command: 'fix', label: 'Fix the findings' });
      if (metadata.command !== 'transition') followups.push({ prompt: 'What would the next Arcadia transition create?', command: 'transition', label: 'Plan the next transition' });
      return followups;
    },
  };
  context.subscriptions.push(participant, vscode.commands.registerCommand('arclang.ai.apply', applyPending));
}
