import * as vscode from 'vscode';

/** The gate standard setting, as the CLI spells it. */
export function gateStandard(): string {
  return vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
}
