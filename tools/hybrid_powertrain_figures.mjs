// The figures of examples/automotive/hybrid-powertrain: one sheet per model
// (P1, P4 as published, P4 corrected), each a PNG and a PDF, and the three
// sheets in one PDF. A sheet is the interconnection view the compiler draws
// (`arclang diagram -f html`, exported as SVG by its viewer), the exchanges
// `arclang check` reports in red, and below it every part with its
// attributes and its ports, and every interface with its ends.
//
//   ARCLANG_BIN=target/release/arclang node tools/hybrid_powertrain_figures.mjs
//
// Needs Playwright with Chrome (`npm i playwright`, Chrome installed).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');

const ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const EXAMPLE = path.join(ROOT, 'examples/automotive/hybrid-powertrain');
const OUT = path.join(EXAMPLE, 'diagrams');
const BIN = process.env.ARCLANG_BIN || path.join(ROOT, 'target/release/arclang');

const MODELS = [
  { file: 'hybrid_powertrain_p1.arc', key: 'p1', title: 'Hybrid powertrain, P1',
    lede: 'The electric motor on the crankshaft of the combustion engine; a 48 V battery of 5 kWh.' },
  { file: 'hybrid_powertrain_p4.arc', key: 'p4-as-published', title: 'Hybrid powertrain, P4 as published',
    lede: 'The electric motor on the second axle; a 400 V battery of 10 kWh. The motor ports as printed in Fig. B.5 and in sec_4/system_definition_p4.sysml.' },
  { file: 'hybrid_powertrain_p4_fixed.arc', key: 'p4-corrected', title: 'Hybrid powertrain, P4 corrected',
    lede: 'The P4 architecture with the motor producing torque and receiving speed, as the P1 motor does.' },
];

const SOURCE = 'Model: Banagar, Kramsu, Könnö, Andwari, “Connecting systems engineering and domain expertise through a toolchain: A simulation-integrated MBSE approach”, Journal of Systems and Software 232 (2026) 112671, CC BY 4.0. Written in ArcLang; drawn and checked by the ArcLang compiler.';

const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** The text of a block opened at `start` (the `{`), braces balanced. */
function block(text, start) {
  let depth = 0;
  for (let i = start; i < text.length; i++) {
    if (text[i] === '{') depth++;
    if (text[i] === '}' && --depth === 0) return text.slice(start + 1, i);
  }
  throw new Error('unbalanced block');
}

/** The parts, their attributes and ports, and the interfaces of a model. */
function readModel(source) {
  const text = source.replace(/\/\/[^\n]*/g, '');
  const parts = [];
  for (const m of text.matchAll(/component "([^"]+)" \{/g)) {
    const body = block(text, m.index + m[0].length - 1);
    const attributes = [];
    for (const a of body.matchAll(/^\s*([a-z_]+):\s*([^\n]+?)\s*$/gm)) {
      if (a[1] !== 'id') attributes.push([a[1], a[2].replace(/^"|"$/g, '')]);
    }
    const ports = [...body.matchAll(/port (in|out|inout) "([^"]+)"/g)].map((p) => ({ direction: p[1], name: p[2] }));
    parts.push({ name: m[1], attributes, ports });
  }
  const interfaces = [...text.matchAll(/component_exchange "([^"]+)" \{ from_port: "([^"]+)" to_port: "([^"]+)" \}/g)]
    .map((e) => ({ label: e[1], from: e[2], to: e[3] }));
  return { name: (source.match(/name: "([^"]+)"/) || [])[1], parts, interfaces };
}

/** The `interface.direction` findings of `arclang check`: exchange label → messages, and the ports concerned. */
function findings(modelPath) {
  let out = '';
  try {
    out = execFileSync(BIN, ['check', modelPath], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (e) {
    out = (e.stdout || '') + (e.stderr || '');
  }
  const byExchange = new Map();
  const ports = new Set();
  for (const line of out.split('\n')) {
    const m = line.match(/exchange direction: component_exchange '([^']+)' (leaves|arrives on) '([^']+)', an? (\w+) port/);
    if (!m) continue;
    const said = `${m[2] === 'leaves' ? 'leaves' : 'arrives on'} ${m[3]}, an ${m[4]} port`;
    byExchange.set(m[1], [...(byExchange.get(m[1]) || []), said]);
    ports.add(m[3]);
  }
  const version = execFileSync(BIN, ['--version'], { encoding: 'utf8' }).trim();
  return { byExchange, ports, version };
}

/** The interconnection view as the viewer exports it. */
async function interconnectionSvg(browser, modelPath, work) {
  const html = path.join(work, path.basename(modelPath, '.arc') + '.html');
  execFileSync(BIN, ['diagram', modelPath, '-f', 'html', '-o', html], { stdio: 'ignore' });
  const page = await browser.newPage({ viewport: { width: 1600, height: 1000 }, acceptDownloads: true });
  await page.goto('file://' + html);
  await page.waitForTimeout(800);
  await page.getByText('Interconnection', { exact: true }).first().click();
  await page.waitForTimeout(400);
  await page.getByText('Export', { exact: true }).click();
  const [download] = await Promise.all([page.waitForEvent('download'), page.getByText('Save as SVG', { exact: true }).click()]);
  const file = path.join(work, path.basename(modelPath, '.arc') + '.svg');
  await download.saveAs(file);
  await page.close();
  return fs.readFileSync(file, 'utf8');
}

/** The SVG with the reported exchanges and ports drawn in red. */
function highlight(svg, found) {
  let s = svg.replace(/<svg /, '<svg style="width:100%;height:auto" ');
  for (const label of found.byExchange.keys()) {
    s = s.replace(new RegExp(`(<g class="av-edge"[^>]*)(>)(<title>Connection: ${label.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}</title>)`), '$1 data-finding="1"$2$3');
  }
  for (const port of found.ports) {
    const [owner, name] = port.split('.');
    s = s.replace(new RegExp(`(<g class="av-port" data-id="[^"]*::${owner}\\.${name}")`), '$1 data-finding="1"');
  }
  return s;
}

const CSS = `
  * { box-sizing: border-box; }
  body { margin: 0; background: #fff; font-family: "Helvetica Neue", Arial, sans-serif; color: #1c2430; }
  .sheet { width: 1600px; padding: 44px 56px 36px; display: flex; flex-direction: column; gap: 18px; page-break-after: always; }
  .sheet:last-child { page-break-after: auto; }
  header { display: flex; justify-content: space-between; align-items: flex-start; border-bottom: 3px solid #1c2430; padding-bottom: 14px; }
  h1 { margin: 0; font-size: 34px; letter-spacing: -0.02em; }
  .lede { margin: 6px 0 0; font-size: 16px; color: #4a5566; max-width: 980px; }
  .verdict { font-size: 15px; font-weight: 600; padding: 10px 16px; border-radius: 6px; white-space: nowrap; }
  .verdict.ok { background: #e6f4ea; color: #1e6b34; border: 1px solid #a8d5b5; }
  .verdict.bad { background: #fdecea; color: #a12a1f; border: 1px solid #f1b0a8; }
  .figure { border: 1px solid #d6dce5; border-radius: 6px; padding: 10px 18px; background: #fff; }
  .figure svg [data-finding] path:not(.av-hit), .figure svg [data-finding] polyline, .figure svg [data-finding] polygon { stroke: #d0352a !important; stroke-width: 2.4 !important; }
  .figure svg [data-finding] polygon, .figure svg g.av-port[data-finding] rect, .figure svg g.av-port[data-finding] path { fill: #d0352a !important; stroke: #d0352a !important; }
  .figure svg [data-finding] text { fill: #d0352a !important; font-weight: 700; }
  .caption { display: flex; gap: 22px; font-size: 13px; color: #4a5566; margin-top: 4px; }
  .caption i { display: inline-block; width: 22px; height: 0; border-top: 2px solid #1f4e9c; vertical-align: middle; margin-right: 6px; }
  .caption i.red { border-color: #d0352a; border-top-width: 3px; }
  .details { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; flex: 1; align-items: start; }
  footer { margin-top: auto; }
  .parts { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; align-content: start; }
  .part { border: 1px solid #d6dce5; border-radius: 6px; padding: 9px 11px; font-size: 12.5px; }
  .part h3 { margin: 0 0 4px; font-size: 14px; }
  .attr { color: #4a5566; font-size: 12px; margin-bottom: 4px; }
  .attr b { color: #1c2430; font-weight: 600; }
  .ports { margin: 0; padding: 0; list-style: none; font-family: Menlo, monospace; font-size: 11.5px; line-height: 1.55; }
  .ports .dir { display: inline-block; width: 40px; color: #6b7686; }
  .ports .bad { color: #d0352a; font-weight: 700; }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th { text-align: left; font-size: 11px; text-transform: uppercase; letter-spacing: 0.04em; color: #6b7686; border-bottom: 1px solid #d6dce5; padding: 4px 6px; }
  td:first-child { white-space: nowrap; }
  td { padding: 4px 6px; border-bottom: 1px solid #eef1f5; font-family: Menlo, monospace; font-size: 11px; vertical-align: top; }
  tr.bad td { color: #a12a1f; background: #fdf3f2; }
  td.note { font-family: "Helvetica Neue", Arial, sans-serif; font-size: 11px; }
  footer { font-size: 11.5px; color: #6b7686; border-top: 1px solid #d6dce5; padding-top: 10px; display: flex; justify-content: space-between; gap: 30px; }
  footer code { font-family: Menlo, monospace; color: #1c2430; }
`;

function sheet(model, data, svg, found) {
  const count = [...found.byExchange.values()].flat().length;
  const verdict = count
    ? `<div class="verdict bad">arclang check: ${count} finding${count > 1 ? 's' : ''}, rule interface.direction</div>`
    : '<div class="verdict ok">arclang check: every interface goes from a producer to a consumer</div>';
  const parts = data.parts.map((p) => `
    <div class="part"><h3>${esc(p.name)}</h3>
      ${p.attributes.length ? `<div class="attr">${p.attributes.map(([k, v]) => `${esc(k)} <b>${esc(v)}</b>`).join(' · ')}</div>` : ''}
      <ul class="ports">${p.ports.map((port) => {
        const bad = found.ports.has(`${p.name}.${port.name}`);
        return `<li class="${bad ? 'bad' : ''}"><span class="dir">${port.direction}</span>${esc(port.name)}</li>`;
      }).join('')}</ul></div>`).join('');
  const rows = data.interfaces.map((i) => {
    const said = found.byExchange.get(i.label);
    return `<tr class="${said ? 'bad' : ''}"><td>${esc(i.label)}</td><td>${esc(i.from)}</td><td>${esc(i.to)}</td><td class="note">${said ? esc(said.join('; ')) : 'out → in'}</td></tr>`;
  }).join('');
  return `<section class="sheet">
  <header><div><h1>${esc(model.title)}</h1><p class="lede">${esc(model.lede)}</p></div>${verdict}</header>
  <div class="figure">${svg}
    <div class="caption"><span><i></i>interface, from an out port to an in port</span>${count ? '<span><i class="red"></i>interface joining two inputs or two outputs, and the port at fault</span>' : ''}<span>Interconnection view: <code>arclang diagram ${esc(model.file)} -f html</code></span></div>
  </div>
  <div class="details">
    <div class="parts">${parts}</div>
    <div><table><thead><tr><th>Interface</th><th>From</th><th>To</th><th>Check</th></tr></thead><tbody>${rows}</tbody></table></div>
  </div>
  <footer><span>${esc(SOURCE)}</span><span style="white-space:nowrap"><code>${esc(found.version)}</code></span></footer>
</section>`;
}

async function main() {
  fs.mkdirSync(OUT, { recursive: true });
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-figures-'));
  const browser = await chromium.launch({ channel: 'chrome' });
  const sheets = [];
  const heights = [];
  for (const model of MODELS) {
    const modelPath = path.join(EXAMPLE, model.file);
    const data = readModel(fs.readFileSync(modelPath, 'utf8'));
    const found = findings(modelPath);
    const svg = highlight(await interconnectionSvg(browser, modelPath, work), found);
    const html = sheet(model, data, svg, found);
    sheets.push(html);
    const page = await browser.newPage({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: 2 });
    await page.setContent(`<!doctype html><html><head><meta charset="utf-8"><style>${CSS}</style></head><body>${html}</body></html>`);
    const height = await page.evaluate(() => Math.ceil(document.querySelector('.sheet').getBoundingClientRect().height));
    heights.push(height);
    await page.screenshot({ path: path.join(OUT, `${model.key}.png`), fullPage: true });
    await page.pdf({ path: path.join(OUT, `${model.key}.pdf`), width: '1600px', height: `${height + 2}px`, printBackground: true, pageRanges: '1' });
    await page.close();
    console.log(`${model.key}: ${[...found.byExchange.values()].flat().length} finding(s)`);
  }
  const page = await browser.newPage();
  const tallest = Math.max(...heights);
  await page.setContent(`<!doctype html><html><head><meta charset="utf-8"><style>${CSS} .sheet { height: ${tallest}px; }</style></head><body>${sheets.join('\n')}</body></html>`);
  await page.pdf({ path: path.join(OUT, 'hybrid-powertrain.pdf'), width: '1600px', height: `${tallest + 2}px`, printBackground: true });
  await browser.close();
  fs.rmSync(work, { recursive: true, force: true });
  console.log(`written to ${path.relative(ROOT, OUT)}`);
}

await main();
