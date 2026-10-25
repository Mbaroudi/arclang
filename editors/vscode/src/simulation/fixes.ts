// The one-click fixes of the simulation checklist. Nothing here goes through
// a shell; every value that reaches a program comes from the compiler's
// diagnosis and is checked first.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { output, tempDir } from '../cli';
import { Fix, safeImageName } from '../pure/simulation';
import { capture, launch } from './process';

const DOCKER_DESKTOP_WINDOWS = 'C:\\Program Files\\Docker\\Docker\\Docker Desktop.exe';

/** Runs a fix; true when it did what it says (a download opened counts as done). */
export async function applyFix(fix: Fix): Promise<boolean> {
  switch (fix.kind) {
    case 'installCompiler':
      await vscode.commands.executeCommand('arclang.installBinary');
      return true;
    case 'openUrl':
      return vscode.env.openExternal(vscode.Uri.parse(fix.url));
    case 'startDocker':
      return startDocker();
    case 'pullImage':
      return pullImage(fix.image);
    case 'installLibrary':
      return installLibrary(fix.image, fix.volume, fix.msl);
    case 'installExtension':
      await vscode.commands.executeCommand('workbench.extensions.installExtension', fix.id);
      return true;
    case 'terminal': {
      // The command is typed, not run: the user reads it and presses Enter.
      const terminal = vscode.window.createTerminal({ name: 'ArcLang simulation setup' });
      terminal.show();
      terminal.sendText(fix.command, false);
      return true;
    }
  }
}

async function startDocker(): Promise<boolean> {
  const started =
    process.platform === 'darwin'
      ? await launch('open', ['-a', 'Docker'])
      : process.platform === 'win32' && fs.existsSync(DOCKER_DESKTOP_WINDOWS)
        ? await launch(DOCKER_DESKTOP_WINDOWS, [])
        : false;
  if (!started) {
    vscode.window.showWarningMessage('ArcLang: start Docker Desktop yourself, then choose "Check again".');
    return false;
  }
  return vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Waiting for Docker to start…', cancellable: true }, async (_progress, token) => {
    for (let i = 0; i < 40 && !token.isCancellationRequested; i++) {
      const answer = await capture('docker', ['version', '--format', '{{.Server.Version}}']);
      if (answer.code === 0) return true;
      await new Promise((resolve) => setTimeout(resolve, 3000));
    }
    return false;
  });
}

async function pullImage(image: string): Promise<boolean> {
  if (!safeImageName(image)) {
    vscode.window.showErrorMessage('ArcLang: the image name is not valid; fix arclang.simulation.image.');
    return false;
  }
  const out = output();
  out.appendLine(`$ docker pull ${image}`);
  return vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `Downloading ${image} (about 1 GB, once)…`, cancellable: true }, async (progress, token) => {
    const result = await capture('docker', ['pull', image], {
      token,
      onData: (text) => {
        out.append(text);
        const line = text.trim().split('\n').pop() ?? '';
        if (line) progress.report({ message: line.slice(0, 80) });
      },
    });
    if (result.code === 0) {
      vscode.window.showInformationMessage('ArcLang: the OpenModelica image is ready.');
      return true;
    }
    if (result.code !== null) {
      out.show(true);
      vscode.window.showErrorMessage('ArcLang: the download failed. See the ArcLang output.');
    }
    return false;
  });
}

async function installLibrary(image: string, volume: string, msl: string): Promise<boolean> {
  if (!safeImageName(image) || !/^[A-Za-z0-9._-]+$/.test(volume) || !/^[0-9.]+$/.test(msl)) return false;
  const dir = tempDir();
  const out = output();
  try {
    fs.writeFileSync(
      path.join(dir, 'install.mos'),
      `if not loadModel(Modelica, {"${msl}"}) then installPackage(Modelica, "${msl}+maint.om", exactMatch=true); end if;\nprint(String(loadModel(Modelica, {"${msl}"})) + "\\n");\ngetErrorString();\n`,
    );
    const args = ['run', '--rm', '-v', `${dir}:/work`, '-v', `${volume}:/root/.openmodelica/libraries`, '-w', '/work', image, 'omc', 'install.mos'];
    out.appendLine(`$ docker ${args.join(' ')}`);
    return await vscode.window.withProgress(
      { location: vscode.ProgressLocation.Notification, title: `Installing the Modelica library ${msl} (a few minutes, once)…`, cancellable: true },
      async (_progress, token) => {
        const result = await capture('docker', args, { token, onData: (text) => out.append(text) });
        const loaded = result.code === 0 && /^true\s*$/m.test(result.output);
        if (loaded) vscode.window.showInformationMessage('ArcLang: the Modelica library is installed. Simulations now start at once.');
        else if (result.code !== null) {
          out.show(true);
          vscode.window.showErrorMessage('ArcLang: the Modelica library could not be installed (it is downloaded from the internet). See the ArcLang output.');
        }
        return loaded;
      },
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}
