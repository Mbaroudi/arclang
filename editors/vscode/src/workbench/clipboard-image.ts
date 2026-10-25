// A PNG on the system clipboard, as an image other applications paste
// (PowerPoint, Keynote, Word, Google Slides): VS Code's clipboard API holds
// text only, and a webview may be refused the browser's. The bytes go to a
// private temporary file (0600, in a directory of its own) that the
// platform's own tool reads, run without a shell; the file is deleted after.
import { execFile } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';

const TIMEOUT_MS = 15000;

function run(file: string, args: string[], input?: Buffer): Promise<void> {
  return new Promise((resolve, reject) => {
    const child = execFile(file, args, { timeout: TIMEOUT_MS, windowsHide: true, shell: false }, (error, _stdout, stderr) => {
      if (error) reject(new Error(`${path.basename(file)}: ${stderr.toString().trim() || error.message}`));
      else resolve();
    });
    if (input) child.stdin?.end(input);
  });
}

/** The commands that put `png` (a file) on the clipboard on this platform, in the order tried. */
export function clipboardCommands(platform: NodeJS.Platform, png: string): { file: string; args: string[]; stdin?: boolean }[] {
  if (platform === 'darwin') {
    // AppleScript reads the file as PNG data: «class PNGf» is what Office and Keynote paste as a picture.
    const script = `set the clipboard to (read (POSIX file ${JSON.stringify(png)}) as «class PNGf»)`;
    return [{ file: '/usr/bin/osascript', args: ['-e', script] }];
  }
  if (platform === 'win32') {
    // The path goes in as an argument, never into the script's text.
    const script = 'param([string]$p) Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $i = [System.Drawing.Image]::FromFile($p); try { [System.Windows.Forms.Clipboard]::SetImage($i) } finally { $i.Dispose() }';
    return [{ file: 'powershell.exe', args: ['-NoProfile', '-NonInteractive', '-STA', '-Command', `& {${script}}`, png] }];
  }
  return [
    { file: 'wl-copy', args: ['--type', 'image/png'], stdin: true },
    { file: 'xclip', args: ['-selection', 'clipboard', '-t', 'image/png', '-i'], stdin: true },
  ];
}

/** Put the PNG `bytes` on the system clipboard as an image. */
export async function writeImageToClipboard(bytes: Buffer, platform: NodeJS.Platform = process.platform): Promise<void> {
  if (bytes.length < 8 || bytes.readUInt32BE(0) !== 0x89504e47) throw new Error('not a PNG image');
  const dir = await mkdtemp(path.join(os.tmpdir(), 'arclang-copy-'));
  const file = path.join(dir, 'diagram.png');
  try {
    await writeFile(file, bytes, { mode: 0o600, flag: 'wx' });
    const failures: string[] = [];
    for (const command of clipboardCommands(platform, file)) {
      try {
        await run(command.file, command.args, command.stdin ? bytes : undefined);
        return;
      } catch (error) {
        failures.push(error instanceof Error ? error.message : String(error));
      }
    }
    throw new Error(platform === 'linux' ? `install wl-clipboard (Wayland) or xclip (X11) to copy images: ${failures.join('; ')}` : failures.join('; '));
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}
