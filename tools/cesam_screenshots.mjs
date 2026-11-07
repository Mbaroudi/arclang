#!/usr/bin/env node
// Regenerates the screenshots of docs/cesam-tutorial.md from the real
// compiler, so that a picture of the page can no longer lie about it.
//
//   cargo build --release
//   node tools/cesam_screenshots.mjs [docs/images]
//
// Needs Playwright (npm i playwright) with Google Chrome installed (the
// `chrome` channel: no browser download), and `cwebp` on the PATH. Every
// shot is a page the compiler wrote (the diagram viewer, the HTML table) or
// a terminal transcript of a command, rendered at 2x and saved as WebP.
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BIN = process.env.ARCLANG_BIN || path.join(ROOT, 'target/release/arclang');
const OUT = path.resolve(process.argv[2] || path.join(ROOT, 'docs/images'));
const EXAMPLE = 'examples/cesam/smart-lock/smart_lock.arc';
const START = 'examples/cesam/exercises/01_start.arc';
const WORK = fs.mkdtempSync(path.join(os.tmpdir(), 'cesam-shots-'));

const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');

function arclang(args, { cwd = ROOT, ok = true } = {}) {
  const r = spawnSync(BIN, args, { cwd, encoding: 'utf8', env: { ...process.env, NO_COLOR: '1' } });
  if (ok && r.status !== 0) throw new Error(`arclang ${args.join(' ')} exited ${r.status}\n${r.stdout}${r.stderr}`);
  return (r.stdout || '') + (r.stderr || '');
}

const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

// A terminal transcript: the command on a prompt line, its real output under it.
function terminal(title, commands) {
  const lines = commands.map((cmd) => {
    const out = arclang(cmd.split(/\s+/).slice(1), { ok: false }).replace(/\s+$/, '');
    const colored = esc(out)
      .replace(/^(\s*)(error|✗|FAIL\b.*)$/gm, '$1<span class="err">$2</span>')
      .replace(/^(\s*(?:warning|⚠).*)$/gm, '<span class="warn">$1</span>')
      .replace(/^(\s*(?:✓|PASS\b|ok\b).*)$/gm, '<span class="ok">$1</span>')
      .replace(/(\[cesam\.[a-z-]+\])/g, '<span class="rule">$1</span>');
    return `<div class="cmd"><span class="prompt">$</span> ${esc(cmd)}</div><pre>${colored}</pre>`;
  });
  return `<!doctype html><html><head><meta charset="utf-8"><style>
    body { margin: 0; background: #e9ebee; font-family: ui-monospace, "SF Mono", Menlo, monospace; }
    .win { width: 1180px; margin: 24px; background: #15181d; color: #d8dde3; border-radius: 10px; box-shadow: 0 12px 32px rgba(0,0,0,.35); overflow: hidden; }
    .bar { display: flex; align-items: center; gap: 8px; padding: 10px 14px; background: #262a31; color: #9aa3ad; font-size: 13px; }
    .bar i { width: 12px; height: 12px; border-radius: 50%; display: inline-block; }
    .bar .t { margin-left: 10px; }
    .body { padding: 14px 18px 18px; font-size: 14.5px; line-height: 1.45; }
    .cmd { color: #ffffff; margin-top: 10px; } .cmd:first-child { margin-top: 0; }
    .prompt { color: #6fd39f; font-weight: 700; }
    pre { margin: 4px 0 0; white-space: pre-wrap; overflow-wrap: anywhere; }
    .err { color: #ff8e80; } .warn { color: #f0c46a; } .ok { color: #6fd39f; } .rule { color: #7fb2ff; }
  </style></head><body><div class="win">
    <div class="bar"><i style="background:#ff5f57"></i><i style="background:#febc2e"></i><i style="background:#28c840"></i><span class="t">${esc(title)}</span></div>
    <div class="body">${lines.join('')}</div></div></body></html>`;
}

async function shoot(browser, html, name, { selector = '.win', width = 1240 } = {}) {
  const file = path.join(WORK, `${name}.html`);
  fs.writeFileSync(file, html);
  const page = await browser.newPage({ viewport: { width, height: 800 }, deviceScaleFactor: 2 });
  await page.goto(`file://${file}`, { waitUntil: 'load' });
  await page.waitForTimeout(400);
  const png = path.join(WORK, `${name}.png`);
  await page.locator(selector).first().screenshot({ path: png });
  await page.close();
  toWebp(png, name);
}

function toWebp(png, name) {
  fs.mkdirSync(OUT, { recursive: true });
  execFileSync('cwebp', ['-quiet', '-q', '88', png, '-o', path.join(OUT, `${name}.webp`)]);
  console.log(`  ✓ ${path.relative(ROOT, path.join(OUT, `${name}.webp`))}`);
}

async function main() {
  if (!fs.existsSync(BIN)) throw new Error(`no compiler at ${BIN}: cargo build --release`);
  const browser = await chromium.launch({ channel: 'chrome' });

  // 1. The grid of the example, as the compiler's own HTML table.
  arclang(['table', EXAMPLE, '--kind', 'cesam_grid', '-f', 'html', '-o', path.join(WORK, 'grid.html')]);
  const grid = await browser.newPage({ viewport: { width: 1240, height: 800 }, deviceScaleFactor: 2 });
  await grid.goto(`file://${path.join(WORK, 'grid.html')}`, { waitUntil: 'load' });
  await grid.emulateMedia({ colorScheme: 'light' });
  await grid.locator('body').screenshot({ path: path.join(WORK, 'cesam-grid.png') });
  await grid.close();
  toWebp(path.join(WORK, 'cesam-grid.png'), 'cesam-grid');

  // 2. The lint of the starter: what CESAM misses, rule by rule.
  await shoot(browser, terminal('arclang — the starter of the exercises', [`arclang check ${START} --lint`]), 'cesam-check');

  // 3. The finished model: no advisory left, the gate passes.
  await shoot(browser, terminal('arclang — the finished model', [`arclang check ${EXAMPLE} --lint`, `arclang gate ${EXAMPLE}`]), 'cesam-gate');

  // 4. The CESAM diagram set in the viewer: the lifecycle (M&S), the use
  //    cases (MCB) and the constructional interactions (LAB).
  arclang(['diagram', EXAMPLE, '-f', 'html', '-o', path.join(WORK, 'views.html'), '--view', 'cesam']);
  const views = await browser.newPage({ viewport: { width: 1400, height: 900 }, deviceScaleFactor: 2 });
  await views.goto(`file://${path.join(WORK, 'views.html')}`, { waitUntil: 'load' });
  await views.waitForSelector('.arcviz-tab');
  await views.waitForTimeout(1500);
  const tabs = await views.evaluate(() => Array.from(document.querySelectorAll('.arcviz-tab')).map((t) => t.textContent));
  console.log('  views:', tabs.join(' | '));
  for (const [tab, name] of [['Modes and states', 'cesam-lifecycle'], ['Capabilities', 'cesam-use-cases'], ['Logical', 'cesam-constructional']]) {
    const button = views.locator('.arcviz-tab', { hasText: tab }).first();
    if (!(await button.count())) { console.log(`  (no tab "${tab}")`); continue; }
    await button.click();
    await views.waitForTimeout(1800);
    await views.locator('main').screenshot({ path: path.join(WORK, `${name}.png`) });
    toWebp(path.join(WORK, `${name}.png`), name);
  }
  await views.close();

  await browser.close();
  fs.rmSync(WORK, { recursive: true, force: true });
}

main().catch((e) => { console.error(e.message || e); process.exit(1); });
