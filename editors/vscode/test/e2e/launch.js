// VS Code under Playwright, whatever its version. Recent VS Code builds turn
// off Electron's --inspect (the fuse Playwright's _electron.launch needs), so
// VS Code is started with Chromium's debugging port and Playwright attaches
// to it; older builds work the same way. The user data directory must have a
// short path: VS Code's socket in it must fit macOS's 104 characters.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const { spawn } = require('child_process');
const fs = require('fs'); const path = require('path'); const net = require('net');

/** A free TCP port on the loopback. */
function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, '127.0.0.1', () => { const { port } = server.address(); server.close(() => resolve(port)); });
    server.on('error', reject);
  });
}

/** The executable of a VS Code application: `Code` in recent builds, `Electron` in older ones. */
function executableOf(app) {
  const found = ['Code', 'Electron'].map((name) => path.join(app, 'Contents/MacOS', name)).find((p) => fs.existsSync(p));
  if (!found) throw new Error(`no VS Code executable in ${app}`);
  return found;
}

/**
 * Launch VS Code with `args` and return `{ win, close }`: `win` is the
 * workbench page, sized `width` x `height`.
 */
async function launchVSCode(app, args, { env = process.env, width = 1600, height = 1000, timeoutMs = 60000 } = {}) {
  const port = await freePort();
  const child = spawn(executableOf(app), [`--remote-debugging-port=${port}`, ...args], { env, stdio: 'ignore' });
  const deadline = Date.now() + timeoutMs;
  let browser = null;
  while (!browser && Date.now() < deadline) {
    await new Promise((r) => setTimeout(r, 500));
    if (child.exitCode !== null) throw new Error(`VS Code exited (code ${child.exitCode}): is the user data directory path short enough?`);
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`).catch(() => null);
  }
  if (!browser) throw new Error('VS Code did not open its debugging port');
  let win = null;
  while (!win && Date.now() < deadline) {
    win = browser.contexts().flatMap((c) => c.pages()).find((p) => /workbench/.test(p.url())) || null;
    if (!win) await new Promise((r) => setTimeout(r, 500));
  }
  if (!win) throw new Error('no VS Code window');
  await win.setViewportSize({ width, height });
  const close = async () => {
    await browser.close().catch(() => undefined);
    child.kill();
    await new Promise((r) => (child.exitCode !== null ? r() : child.once('exit', r)));
  };
  return { win, close };
}

module.exports = { launchVSCode };
