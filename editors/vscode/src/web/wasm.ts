// Loads the ArcLang compiler, the same Rust core as the `arclang` binary,
// built to WebAssembly (`wasm/`, see docs/WEB_PLAYGROUND.md). The glue is
// bundled in the extension; the module itself is read from the extension's
// own files, so nothing is fetched from another host.
import * as vscode from 'vscode';
import { initSync } from '../../../../wasm/pkg/arclang_wasm.js';
import * as bindings from '../../../../wasm/pkg/arclang_wasm.js';

export type Wasm = typeof bindings;

let loaded: Wasm | null = null;

export async function loadWasm(context: vscode.ExtensionContext): Promise<Wasm> {
  if (loaded) return loaded;
  const bytes = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(context.extensionUri, 'dist', 'web', 'arclang_wasm_bg.wasm'));
  initSync({ module: bytes });
  loaded = bindings;
  return loaded;
}
