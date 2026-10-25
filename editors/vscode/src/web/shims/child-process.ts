// A browser cannot start a process: the web build never reaches this (the
// WebAssembly backend answers every compiler call), and says so if it does.
export function execFile(): never {
  throw new Error('child_process is not available in the browser build');
}
export const spawn = execFile;
export const exec = execFile;
