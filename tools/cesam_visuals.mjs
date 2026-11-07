#!/usr/bin/env node
// The CESAM figures of docs/cesam.md, drawn from the real compiler so that
// none of them can say what the compiler does not:
//
//   cesam-hero.webp     the grid of the smart lock, three visions by six
//                       aspects, each cell with its count, its Arcadia levels
//                       and its maturity, from `arclang table --kind cesam_grid -f json`
//   cesam-states.webp   the three state machines of the three visions, as the
//                       viewer draws them (`arclang diagram --view cesam`)
//   cesam-visions.webp  one model seen three times: use cases, functions,
//                       components, from the same diagram set
//
//   cargo build --release
//   node tools/cesam_visuals.mjs [docs/images]
//
// Needs Playwright with Google Chrome (the `chrome` channel) and `cwebp`.
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
const WORK = fs.mkdtempSync(path.join(os.tmpdir(), 'cesam-visuals-'));
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');

const VISIONS = [
  { key: 'Operational', color: '#0f7b63', soft: '#e3f3ee', question: 'For whom, and for what?' },
  { key: 'Functional', color: '#1f5fbf', soft: '#e4edfb', question: 'What does it do?' },
  { key: 'Constructional', color: '#b4561a', soft: '#fbeadc', question: 'What is it made of?' },
];

function arclang(args) {
  const r = spawnSync(BIN, args, { cwd: ROOT, encoding: 'utf8', env: { ...process.env, NO_COLOR: '1' } });
  if (r.status !== 0) throw new Error(`arclang ${args.join(' ')} exited ${r.status}\n${r.stdout}${r.stderr}`);
  return r.stdout;
}
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

function toWebp(png, name, quality = 86) {
  fs.mkdirSync(OUT, { recursive: true });
  execFileSync('cwebp', ['-quiet', '-q', String(quality), png, '-o', path.join(OUT, `${name}.webp`)]);
  console.log(`  ✓ ${path.relative(ROOT, path.join(OUT, `${name}.webp`))}`);
}

async function shootHtml(browser, html, name, { width, selector = 'body', scale = 2 }) {
  const file = path.join(WORK, `${name}.html`);
  fs.writeFileSync(file, html);
  const page = await browser.newPage({ viewport: { width, height: 600 }, deviceScaleFactor: scale });
  await page.goto(`file://${file}`, { waitUntil: 'load' });
  await page.waitForTimeout(300);
  const png = path.join(WORK, `${name}.png`);
  await page.locator(selector).first().screenshot({ path: png });
  await page.close();
  return png;
}

const FONT = `font-family: -apple-system, "SF Pro Text", "Segoe UI", Inter, Helvetica, Arial, sans-serif;`;

// 1. The grid as an infographic: what the compiler computed, cell by cell.
function heroHtml(grid) {
  const columns = grid.columns.filter((c) => c.key !== 'vision');
  const rows = grid.rows.map((row, i) => {
    const vision = VISIONS[i];
    const cells = columns.map((c) => {
      const cell = row.cells[c.key];
      const m = /^(\d+)(?:\s*\(([^)]*)\))?/.exec(cell.text || '');
      const count = m ? m[1] : cell.text;
      const levels = m && m[2] ? m[2] : '';
      const names = cell.links.slice(0, 3).map((l) => l.name);
      const more = cell.links.length - names.length;
      return `<td style="background:${vision.soft}">
        <div class="top"><span class="count" style="color:${vision.color}">${esc(count)}</span><span class="status ${esc(cell.status)}">${esc(cell.status)}</span></div>
        ${levels ? `<div class="levels">${esc(levels)}</div>` : '<div class="levels">&nbsp;</div>'}
        <div class="names">${names.map(esc).join(', ')}${more > 0 ? ` <span class="more">+${more}</span>` : ''}</div>
      </td>`;
    });
    return `<tr><th style="border-left: 6px solid ${vision.color}"><div class="vision" style="color:${vision.color}">${esc(row.name)}</div><div class="question">${esc(vision.question)}</div></th>${cells.join('')}</tr>`;
  });
  const s = grid.summary;
  return `<!doctype html><html><head><meta charset="utf-8"><style>
    body { margin: 0; background: #f6f4ef; color: #1b1f24; ${FONT} }
    .sheet { width: 1600px; padding: 48px 56px 44px; box-sizing: border-box; background:
      radial-gradient(1200px 500px at 15% -10%, #ffffff 0%, rgba(255,255,255,0) 60%), #f6f4ef; }
    .kicker { font-family: ui-monospace, "SF Mono", Menlo, monospace; font-size: 15px; letter-spacing: .14em; color: #6b7280; text-transform: uppercase; }
    h1 { font-size: 40px; letter-spacing: -0.02em; margin: 8px 0 6px; font-weight: 700; }
    .sub { font-size: 19px; color: #4b5563; margin: 0 0 28px; }
    .sub b { color: #0f7b63; font-weight: 600; }
    table { border-collapse: separate; border-spacing: 6px; width: 100%; table-layout: fixed; }
    thead th { font-size: 15px; font-weight: 600; color: #374151; text-align: left; padding: 0 12px 6px; letter-spacing: .01em; }
    thead th:first-child { width: 210px; }
    tbody th { text-align: left; vertical-align: top; padding: 14px 14px 14px 16px; background: #fff; border-radius: 10px; box-shadow: 0 1px 0 rgba(0,0,0,.04); }
    .vision { font-size: 21px; font-weight: 700; letter-spacing: -0.01em; }
    .question { font-size: 14px; color: #6b7280; margin-top: 6px; line-height: 1.3; }
    td { vertical-align: top; padding: 12px 14px 14px; border-radius: 10px; height: 128px; }
    .top { display: flex; align-items: baseline; justify-content: space-between; }
    .count { font-size: 38px; font-weight: 700; letter-spacing: -0.03em; line-height: 1; }
    .status { font-family: ui-monospace, Menlo, monospace; font-size: 12px; font-weight: 700; padding: 3px 8px; border-radius: 999px; text-transform: uppercase; letter-spacing: .06em; }
    .status.ok { background: #d8f1e4; color: #0b6b4f; } .status.warn { background: #fde9c2; color: #8a5a00; } .status.gap, .status.fail { background: #fbd9d6; color: #9b1c1c; }
    .levels { font-family: ui-monospace, Menlo, monospace; font-size: 12.5px; color: #6b7280; margin-top: 6px; }
    .names { font-size: 13.5px; color: #374151; margin-top: 8px; line-height: 1.35; }
    .more { color: #9ca3af; }
    .foot { display: flex; justify-content: space-between; margin-top: 18px; font-size: 14px; color: #6b7280; }
    .foot code { font-family: ui-monospace, Menlo, monospace; background: #e9e6df; padding: 2px 7px; border-radius: 5px; color: #1b1f24; }
  </style></head><body><div class="sheet">
    <div class="kicker">CESAM grid · computed by the compiler</div>
    <h1>One Arcadia model, read through the three visions of CESAM</h1>
    <p class="sub">The smart lock of <code>examples/cesam/smart-lock</code>: ${s.rows} visions × ${columns.length} aspects, <b>${s.ok} of ${s.rows} rows ok</b>, gate ${esc(s.gate_standard)} ${s.gate_passed ? 'passed' : 'failed'}. Each cell counts the elements that fill the view, by Arcadia level.</p>
    <table><thead><tr><th>Vision</th>${columns.map((c) => `<th>${esc(c.label)}</th>`).join('')}</tr></thead><tbody>${rows.join('')}</tbody></table>
    <div class="foot"><span><code>arclang table smart_lock.arc --kind cesam_grid</code> · the same grid in VS Code, Markdown, HTML, CSV and JSON</span><span>OA operational · SA system · LA logical · PA physical</span></div>
  </div></body></html>`;
}

// 2. A diagram of the viewer, cropped to its drawing.
async function canvas(views, tab, diagram, name) {
  await views.locator('.arcviz-tab', { hasText: tab }).first().click();
  await views.waitForTimeout(500);
  // A tab with several diagrams shows a selector; a tab with one does not.
  const select = views.locator('select').first();
  let label = diagram;
  if (await select.isVisible()) {
    label = (await select.locator('option').allTextContents()).find((t) => t.trim().startsWith(diagram));
    if (!label) throw new Error(`no diagram "${diagram}" under "${tab}"`);
    await select.selectOption({ label: label.trim() });
    await views.waitForTimeout(1500);
  }
  await views.keyboard.press('0');
  await views.waitForTimeout(600);
  const box = await views.evaluate(() => {
    const stage = document.querySelector('.arcviz-stage');
    const nodes = Array.from(stage.querySelectorAll('svg g, svg rect, svg path, svg text, svg circle, svg ellipse, svg polygon'))
      .map((e) => e.getBoundingClientRect()).filter((r) => r.width > 0 && r.height > 0 && r.width < 4000);
    const s = stage.getBoundingClientRect();
    let x1 = Infinity, y1 = Infinity, x2 = -Infinity, y2 = -Infinity;
    for (const r of nodes) { x1 = Math.min(x1, r.left); y1 = Math.min(y1, r.top); x2 = Math.max(x2, r.right); y2 = Math.max(y2, r.bottom); }
    const pad = 10;
    const g = stage.querySelector('svg g[transform]');
    const m = /scale\(([\d.]+)/.exec(g?.getAttribute('transform') || '');
    return { x: Math.max(s.left, x1 - pad), y: Math.max(s.top, y1 - pad), width: Math.min(s.right, x2 + pad) - Math.max(s.left, x1 - pad), height: Math.min(s.bottom, y2 + pad) - Math.max(s.top, y1 - pad), scale: m ? Number(m[1]) : 1 };
  });
  const png = path.join(WORK, `${name}.png`);
  await views.screenshot({ path: png, clip: box });
  return { png, width: box.width, height: box.height, scale: box.scale, label };
}

// 3. Three drawings side by side, each under the colour of its vision.
function panelsHtml(title, subtitle, panels, note) {
  const base = Math.min(...panels.map((p) => p.scale));
  const widths = panels.map((p) => p.width * base / p.scale);
  const k = Math.min(1, 1800 / widths.reduce((a, b) => a + b, 0));
  const items = panels.map((p, i) => {
    p = { ...p, width: widths[i] * k };
    const v = VISIONS[i];
    return `<figure>
      <figcaption><span class="dot" style="background:${v.color}"></span><b style="color:${v.color}">${esc(v.key)} vision</b><span class="name">${esc(p.title)}</span></figcaption>
      <div class="frame" style="border-top: 4px solid ${v.color}"><img src="file://${p.png}" style="width:${Math.round(p.width)}px"></div>
      <p class="under">${esc(p.under)}</p>
    </figure>`;
  });
  return `<!doctype html><html><head><meta charset="utf-8"><style>
    body { margin: 0; background: #f6f4ef; color: #1b1f24; ${FONT} }
    .sheet { display: inline-block; padding: 40px 48px 36px; box-sizing: border-box; background: #f6f4ef; }
    .kicker { font-family: ui-monospace, Menlo, monospace; font-size: 14px; letter-spacing: .14em; color: #6b7280; text-transform: uppercase; }
    h1 { font-size: 32px; letter-spacing: -0.02em; margin: 6px 0 4px; }
    .sub { font-size: 17px; color: #4b5563; margin: 0 0 24px; }
    .row { display: flex; gap: 28px; align-items: flex-start; }
    figure { margin: 0; flex: 0 0 auto; }
    figcaption { display: flex; align-items: center; gap: 8px; font-size: 15px; margin-bottom: 10px; }
    .dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }
    .name { color: #4b5563; }
    .frame { background: #fff; border-radius: 10px; box-shadow: 0 10px 28px rgba(20,24,30,.10), 0 1px 0 rgba(0,0,0,.05); padding: 14px; display: inline-block; }
    .frame img { display: block; height: auto; }
    .under { font-size: 14px; color: #4b5563; margin: 10px 0 0; max-width: 420px; line-height: 1.4; }
    .note { margin-top: 22px; font-size: 14px; color: #6b7280; }
    .note code { font-family: ui-monospace, Menlo, monospace; background: #e9e6df; padding: 2px 7px; border-radius: 5px; color: #1b1f24; }
  </style></head><body><div class="sheet">
    <div class="kicker">${esc(subtitle)}</div><h1>${esc(title)}</h1>
    <div class="row">${items.join('')}</div>
    <p class="note">${note}</p>
  </div></body></html>`;
}

async function main() {
  if (!fs.existsSync(BIN)) throw new Error(`no compiler at ${BIN}: cargo build --release`);
  const browser = await chromium.launch({ channel: 'chrome' });

  const grid = JSON.parse(arclang(['table', EXAMPLE, '--kind', 'cesam_grid', '-f', 'json']));
  toWebp(await shootHtml(browser, heroHtml(grid), 'cesam-hero', { width: 1600, selector: '.sheet' }), 'cesam-hero');

  const viewsFile = path.join(WORK, 'views.html');
  arclang(['diagram', EXAMPLE, '-f', 'html', '-o', viewsFile, '--view', 'cesam']);
  const views = await browser.newPage({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: 2 });
  await views.goto(`file://${viewsFile}`, { waitUntil: 'load' });
  await views.waitForSelector('.arcviz-tab');
  await views.waitForTimeout(1500);

  const states = [
    { ...(await canvas(views, 'Modes and states', 'Lifecycle', 'lifecycle')), title: 'Lock lifecycle', under: 'The phases of the system\'s life, each with its operational context: installation, service, maintenance.' },
    { ...(await canvas(views, 'Modes and states', 'Functional modes', 'modes')), title: 'Lock modes', under: 'The functional modes, each realizing a phase: pairing belongs to installation, locked and unlocked to service.' },
    { ...(await canvas(views, 'Modes and states', 'Technical configurations', 'configurations')), title: 'Lock configurations', under: 'The technical configurations, each realizing a mode: radio only is how pairing is built.' },
  ];
  toWebp(await shootHtml(browser, panelsHtml('The states of the three visions, tied together', 'arclang diagram --view cesam · Modes and states',
    states, 'A configuration <code>realizes</code> a mode, a mode <code>realizes</code> a phase; <code>cesam.role</code> warns when a link points the wrong way, <code>cesam.lifecycle-context</code> and <code>cesam.mode-configuration</code> when a phase or a mode stands alone.'),
    'cesam-states', { width: 2000, selector: '.sheet' }), 'cesam-states');

  const visions = [
    { ...(await canvas(views, 'Capabilities', 'Stakeholders and use cases', 'use-cases')), title: 'Stakeholders and use cases', under: 'The mission, the use cases that serve it, the actors each involves.' },
    { ...(await canvas(views, 'System', 'Environment diagram', 'functions')), title: 'Functional environment', under: 'The system as a black box: its functions, and the actors it exchanges with.' },
    { ...(await canvas(views, 'Logical', 'Constructional interactions', 'components')), title: 'Constructional interactions', under: 'The components, the functions allocated to each, and the exchanges between them.' },
  ];
  toWebp(await shootHtml(browser, panelsHtml('One model, three visions', 'arclang diagram --view cesam · the CESAM diagram set',
    visions, 'Nothing is drawn twice: the operational vision is the Arcadia operational analysis, the functional vision the system analysis, the constructional vision the logical and physical architectures. The grid, the rules and these diagrams read the layers you already write.'),
    'cesam-visions', { width: 2000, selector: '.sheet' }), 'cesam-visions');

  await views.close();
  await browser.close();
  fs.rmSync(WORK, { recursive: true, force: true });
}

main().catch((e) => { console.error(e.stack || e.message || e); process.exit(1); });
