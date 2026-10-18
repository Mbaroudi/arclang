// Finding the compiler, and helping the user install it.
import * as vscode from 'vscode';
import { binary, output, run } from './cli';

const RELEASES = 'https://github.com/Mbaroudi/arclang/releases/latest';

/** The compiler's version line, or null when it cannot run. */
export async function compilerVersion(): Promise<string | null> {
  const result = await run(['--version']);
  if (result.code !== 0) return null;
  return result.stdout.trim() || null;
}

let warned = false;

/** Tells the user once when the binary is missing, with ways to fix it. */
export async function warnIfMissing(): Promise<boolean> {
  const version = await compilerVersion();
  if (version) return true;
  if (warned) return false;
  warned = true;
  const choice = await vscode.window.showWarningMessage(
    `ArcLang: cannot run \`${binary()}\`. The extension needs the arclang compiler.`,
    'Install or locate…',
  );
  if (choice) await installBinary();
  return false;
}

/** The "Install or locate the arclang compiler" command. */
export async function installBinary(): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      { label: '$(folder-opened) Locate an installed arclang…', detail: 'Choose the executable; the setting arclang.binary is set to its path.', id: 'locate' },
      { label: '$(cloud-download) Download a release', detail: `${RELEASES} (macOS and Linux archives; put arclang on the PATH)`, id: 'download' },
      { label: '$(terminal) Build from source', detail: 'git clone https://github.com/Mbaroudi/arclang && cargo build --release --features native', id: 'build' },
    ],
    { placeHolder: 'The arclang compiler is not found. How do you want to get it?' },
  );
  if (!choice) return;
  if (choice.id === 'locate') {
    const picked = await vscode.window.showOpenDialog({ canSelectMany: false, openLabel: 'Use this arclang', title: 'The arclang executable' });
    if (!picked?.[0]) return;
    await vscode.workspace.getConfiguration('arclang').update('binary', picked[0].fsPath, vscode.ConfigurationTarget.Global);
    const version = await compilerVersion();
    if (version) vscode.window.showInformationMessage(`ArcLang: using ${version}.`);
    else vscode.window.showErrorMessage(`ArcLang: ${picked[0].fsPath} does not answer to --version.`);
    return;
  }
  if (choice.id === 'download') {
    await vscode.env.openExternal(vscode.Uri.parse(RELEASES));
    return;
  }
  const out = output();
  out.appendLine('Build the ArcLang compiler from source:');
  out.appendLine('  git clone https://github.com/Mbaroudi/arclang');
  out.appendLine('  cd arclang && cargo build --release --features native');
  out.appendLine('  # then set arclang.binary to <repo>/target/release/arclang, or copy it on the PATH');
  out.show(true);
}
