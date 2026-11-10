// Offers the ArcLang MCP server (mcp-server/, Python) to agent mode when it
// is installed: setting arclang.mcp.command, else `arclang-mcp` on the PATH.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { binary, workspaceTrusted } from './cli';
import { splitCommand } from './pure/arc';
import { mcpEnvironment } from './pure/mcp-env';

function onPath(name: string): string | null {
  const names = process.platform === 'win32' ? [`${name}.exe`, `${name}.cmd`, `${name}.bat`, name] : [name];
  for (const dir of (process.env.PATH ?? '').split(path.delimiter)) {
    if (!dir) continue;
    for (const candidate of names) {
      const full = path.join(dir, candidate);
      try {
        fs.accessSync(full, fs.constants.X_OK);
        return full;
      } catch {
        // not here
      }
    }
  }
  return null;
}

export function registerMcpProvider(context: vscode.ExtensionContext): void {
  const changed = new vscode.EventEmitter<void>();
  context.subscriptions.push(
    changed,
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('arclang.mcp') || event.affectsConfiguration('arclang.binary')) changed.fire();
    }),
    vscode.workspace.onDidGrantWorkspaceTrust(() => changed.fire()),
    // The server keeps to the folders that are open: it is started again when they change.
    vscode.workspace.onDidChangeWorkspaceFolders(() => changed.fire()),
    vscode.lm.registerMcpServerDefinitionProvider('arclang.mcp', {
      onDidChangeMcpServerDefinitions: changed.event,
      provideMcpServerDefinitions() {
        if (!workspaceTrusted()) return [];
        const configuration = vscode.workspace.getConfiguration('arclang');
        const configured = splitCommand(configuration.get<string>('mcp.command', '').trim());
        const extraArgs = configuration.get<string[]>('mcp.args', []);
        const command = configured[0] ?? onPath('arclang-mcp');
        if (!command) return [];
        const args = [...configured.slice(1), ...extraArgs];
        const folders = (vscode.workspace.workspaceFolders ?? []).filter((folder) => folder.uri.scheme === 'file').map((folder) => folder.uri.fsPath);
        const env = mcpEnvironment({ folders, binary: path.isAbsolute(binary()) ? binary() : null, inherited: process.env, delimiter: path.delimiter });
        return [new vscode.McpStdioServerDefinition('ArcLang', command, args, env, context.extension.packageJSON.version)];
      },
    }),
  );
}
