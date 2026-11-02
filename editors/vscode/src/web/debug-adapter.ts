// The model debugger in the browser: the same Debug Adapter Protocol as
// `arclang debug --dap`, run by the WebAssembly compiler in this worker (an
// inline adapter). There is no file to read in WebAssembly: the launch
// request carries the model's text, read here from the workspace.
import * as vscode from 'vscode';
import type { Wasm } from './wasm';

interface Answer {
  messages: vscode.DebugProtocolMessage[];
  done: boolean;
}

interface Request {
  command?: string;
  arguments?: { program?: unknown; source?: unknown };
}

class WasmDebugAdapter implements vscode.DebugAdapter {
  private readonly sent = new vscode.EventEmitter<vscode.DebugProtocolMessage>();
  readonly onDidSendMessage = this.sent.event;
  private session: InstanceType<Wasm['DebugSession']> | null;
  // One request at a time, in order: a launch waits for the model's text.
  private queue: Promise<void> = Promise.resolve();

  constructor(wasm: Wasm) {
    this.session = new wasm.DebugSession();
  }

  handleMessage(message: vscode.DebugProtocolMessage): void {
    this.queue = this.queue.then(() => this.answer(message as Request)).catch((error: unknown) => {
      this.sent.fire({ type: 'event', event: 'output', body: { category: 'stderr', output: `The debugger failed: ${error instanceof Error ? error.message : String(error)}\n` } } as vscode.DebugProtocolMessage);
    });
  }

  private async answer(request: Request): Promise<void> {
    if (!this.session) return;
    const program = request.arguments?.program;
    if (request.command === 'launch' && typeof program === 'string' && typeof request.arguments?.source !== 'string') {
      request = { ...request, arguments: { ...request.arguments, source: await readText(program) } };
    }
    const answer = JSON.parse(this.session.handle(JSON.stringify(request))) as Answer;
    for (const message of answer.messages) this.sent.fire(message);
    if (answer.done) this.dispose();
  }

  dispose(): void {
    this.session?.free();
    this.session = null;
  }
}

/** The model's text, or undefined: the adapter then answers that it cannot read it. */
async function readText(program: string): Promise<string | undefined> {
  try {
    return new TextDecoder().decode(await vscode.workspace.fs.readFile(vscode.Uri.file(program)));
  } catch {
    return undefined;
  }
}

export function wasmDebugAdapter(wasm: Wasm): () => vscode.DebugAdapterDescriptor {
  return () => new vscode.DebugAdapterInlineImplementation(new WasmDebugAdapter(wasm));
}
