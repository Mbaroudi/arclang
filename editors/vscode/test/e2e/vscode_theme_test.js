// Colour themes, Copy as image and the PowerPoint export in the real VS
// Code: every preset chosen from the toolbar's Theme menu and checked on
// every layer, light and dark; a custom theme refused for a reserved colour
// then saved for the team; the image read back from the macOS clipboard;
// the .pptx read back with python-pptx.
//   VSCODE_APP=... PLAYWRIGHT=... ARCLANG_BINARY=... [SHOTS=/dir] [PPTX_PYTHON=/venv/bin/python] node vscode_theme_test.js
const { _electron } = require(process.env.PLAYWRIGHT || 'playwright');
const { execFileSync } = require('child_process');
const fs = require('fs'); const os = require('os'); const path = require('path');
const { azerty } = require('./azerty');

const APP = process.env.VSCODE_APP || '/Applications/Visual Studio Code.app';
const EXT = path.resolve(__dirname, '../..');
const PRESETS = ['arclang-default', 'capella-classic', 'colorblind-safe', 'print-grayscale', 'high-contrast', 'monochrome-document'];
const preset = (id) => JSON.parse(fs.readFileSync(path.join(EXT, 'media', 'viewer', 'themes', `${id}.json`), 'utf8'));
const rows = []; const report = (name, ok, detail) => { rows.push({ name, ok }); console.log(`${ok ? 'ok  ' : 'FAIL'} ${name.padEnd(70)} ${detail || ''}`); };
const shot = async (win, name, clip) => { if (process.env.SHOTS) await win.screenshot({ path: path.join(process.env.SHOTS, `${name}.png`), ...(clip ? { clip } : {}) }); };

async function launch(dir, model, extra = {}) {
  const settings = { 'files.simpleDialog.enable': true, 'update.mode': 'none', 'extensions.autoCheckUpdates': false, 'extensions.autoUpdate': false, 'telemetry.telemetryLevel': 'off', 'workbench.startupEditor': 'none', 'security.workspace.trust.enabled': false, 'window.restoreWindows': 'none', ...(process.env.ARCLANG_BINARY ? { 'arclang.binary': process.env.ARCLANG_BINARY } : {}), ...extra };
  fs.mkdirSync(path.join(dir, 'u', 'User'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'u', 'User', 'settings.json'), JSON.stringify(settings));
  const work = path.join(dir, 'work');
  const app = await _electron.launch({ executablePath: path.join(APP, 'Contents/MacOS/Electron'), args: [`--user-data-dir=${dir}/u`, `--extensions-dir=${dir}/x`, `--extensionDevelopmentPath=${EXT}`, '--skip-welcome', '--skip-release-notes', work, model] });
  const win = await app.firstWindow();
  await app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].setBounds({ x: 0, y: 0, width: 1600, height: 1000 }));
  await win.waitForTimeout(4000);
  const command = async (name) => { await win.keyboard.press('Meta+Shift+P'); await win.waitForTimeout(300); await win.keyboard.type(name); await win.waitForTimeout(600); await win.keyboard.press('Enter'); await win.waitForTimeout(600); };
  await command('View: Close Secondary Side Bar');
  await command('ArcLang: Open the workbench');
  let page = null;
  for (let i = 0; i < 60 && !page; i++) {
    await win.waitForTimeout(500);
    for (const f of win.frames()) { try { if (await f.$('.arcviz-stage svg')) { page = f; break; } } catch { /* a frame going away */ } }
  }
  await win.waitForTimeout(1500);
  return { app, win, page, command, work };
}

function workspace() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arcvs-theme-'));
  const work = path.join(dir, 'work');
  fs.mkdirSync(work);
  const model = path.join(work, 'model.arc');
  fs.copyFileSync(path.join(EXT, 'media', 'reference.arc'), model);
  return { dir, work, model };
}

/** The fill of the first box of `kind` drawn on the stage, and the sheet's colour. */
const fills = (page) => page.evaluate(() => {
  const out = {};
  for (const g of document.querySelectorAll('.arcviz-stage .av-node')) {
    const shape = g.querySelector(':scope > .av-shape');
    if (shape && !out[g.dataset.kind]) out[g.dataset.kind] = shape.getAttribute('fill');
  }
  const paper = document.querySelector('.arcviz-stage .av-paper');
  const svg = document.querySelector('.arcviz-stage svg.av-svg');
  return { kinds: out, paper: paper && paper.getAttribute('fill'), filter: svg ? getComputedStyle(svg).filter : null, ownDark: document.body.classList.contains('wb-theme-own-dark') };
});

async function layer(ctx, name) { await ctx.page.locator('.wb-layer', { hasText: name }).first().click(); await ctx.win.waitForTimeout(1800); }

async function chooseTheme(ctx, key) {
  await ctx.page.locator('#arcviz-theme-menu').click(); await ctx.win.waitForTimeout(400);
  const item = ctx.page.locator(`.arcviz-menu-pop:not([hidden]) [data-action="theme-${key}"]`);
  const found = (await item.count()) === 1;
  if (found) await item.click(); else await ctx.win.keyboard.press('Escape');
  await ctx.win.waitForTimeout(2500);
  return found;
}

const LAYERS = [['Operational', 'oab', 'operational_activity', 'operational_activity'], ['System', 'sab', 'function', 'system_function'], ['Logical', 'lab', 'function', 'logical_function'], ['Physical', 'pab', 'physical_node', 'physical_node']];

/** Every preset from the Theme menu, every layer coloured by it; in light and dark VS Code. */
async function presets(colorTheme, slug) {
  const { dir, model } = workspace();
  const ctx = await launch(dir, model, { 'workbench.colorTheme': colorTheme });
  const { win } = ctx;
  try {
    report(`${slug}: the workbench opens`, !!ctx.page);
    if (!ctx.page) throw new Error('no workbench page');
    await ctx.command('View: Close Primary Side Bar'); await ctx.command('View: Single Column Editor Layout'); await ctx.command('ArcLang: Open the workbench'); await ctx.win.waitForTimeout(2000);
    const items = await (async () => { await ctx.page.locator('#arcviz-theme-menu').click(); await win.waitForTimeout(400); const a = await ctx.page.locator('.arcviz-menu-pop:not([hidden]) [data-action^="theme-"]').evaluateAll((n) => n.map((x) => x.dataset.action)); await win.keyboard.press('Escape'); return a; })();
    report(`${slug}: the Theme menu lists the six presets and the editor`, PRESETS.every((p) => items.includes(`theme-preset:${p}`)) && items.includes('theme-edit'), items.join(' '));
    for (const id of PRESETS) {
      const theme = preset(id);
      const chosen = await chooseTheme(ctx, `preset:${id}`);
      const wrong = [];
      for (const [name, , kind, concept] of LAYERS) {
        await layer(ctx, name);
        const seen = await fills(ctx.page);
        const want = theme.concepts[concept].fill;
        const own = slug === 'dark' && theme.dark ? { ...theme, ...theme.dark, concepts: { ...theme.concepts, ...theme.dark.concepts } } : theme;
        const expected = own.concepts[concept].fill;
        if ((seen.kinds[kind] || '').toUpperCase() !== expected.toUpperCase()) wrong.push(`${name}: ${kind} ${seen.kinds[kind]} not ${expected}`);
        if (slug === 'dark') {
          // No dark variant: the editor's inversion. A dark variant: drawn as it is, on its own dark sheet.
          const inverted = /invert/.test(seen.filter || '');
          if (theme.dark ? inverted || !seen.ownDark || seen.paper !== theme.dark.background : !inverted) wrong.push(`${name}: dark rule (${seen.filter}, own ${seen.ownDark}, paper ${seen.paper})`);
        }
        if (name === 'Logical' || name === 'Physical') await shot(win, `vscode-${slug}-${id}-${name.toLowerCase()}`);
        void want;
      }
      report(`${slug} ${id}: chosen from the Theme menu, it colours every layer`, chosen && wrong.length === 0, wrong.join('; '));
      const setting = JSON.parse(fs.readFileSync(path.join(ctx.work, '.vscode', 'settings.json'), 'utf8'))['arclang.diagram.theme'];
      if (id === 'capella-classic') report(`${slug}: the choice is the workspace's setting`, setting === 'capella-classic', setting);
    }
    if (slug === 'dark') {
      // A narrow panel still draws dark: the rule holds at every width.
      const bounds = await ctx.app.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0].getBounds());
      await ctx.app.evaluate(({ BrowserWindow }, b) => BrowserWindow.getAllWindows()[0].setBounds({ ...b, width: 520 }), bounds);
      await win.waitForTimeout(2000);
      // The last preset chosen (monochrome document) has no dark variant.
      const narrow = await fills(ctx.page);
      report('dark 520 px: a theme without a dark variant is inverted on a narrow panel', /invert/.test(narrow.filter || ''), narrow.filter);
      await shot(win, 'vscode-dark-narrow-capella');
    }
  } catch (error) {
    report(`no error (${slug} presets)`, false, String(error).slice(0, 300));
    await win.screenshot({ path: path.join(process.env.SHOTS || os.tmpdir(), `vscode-theme-failure-${slug}.png`) }).catch(() => {});
  } finally {
    await ctx.app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

/** The PNG on the macOS clipboard: «class PNGf» in clipboard info, and its size from its header. */
function clipboardPng() {
  const info = execFileSync('/usr/bin/osascript', ['-e', 'clipboard info'], { encoding: 'utf8' });
  if (!/«class PNGf»|PNGf/.test(info)) return { info, size: null };
  const hex = execFileSync('/usr/bin/osascript', ['-e', 'the clipboard as «class PNGf»'], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
  const data = Buffer.from((/«data PNGf([0-9A-F]+)»/i.exec(hex) || [])[1] || '', 'hex');
  const png = data.length > 24 && data.readUInt32BE(0) === 0x89504e47;
  return { info: info.trim().slice(0, 120), size: png ? { width: data.readUInt32BE(16), height: data.readUInt32BE(20) } : null };
}

/** The theme editor, Copy as image, Copy as SVG and the PowerPoint export. */
async function editorAndExports() {
  const { dir, model, work } = workspace();
  const ctx = await launch(dir, model, { 'workbench.colorTheme': 'Default Light Modern' });
  const { win } = ctx;
  try {
    if (!ctx.page) throw new Error('no workbench page');
    const page = ctx.page;
    const press = await azerty(win);
    await ctx.command('View: Close Primary Side Bar'); await ctx.command('View: Single Column Editor Layout'); await ctx.command('ArcLang: Open the workbench'); await ctx.win.waitForTimeout(2000);
    await layer(ctx, 'Logical');

    // 1. The editor: a copy of Capella classic, a reserved colour refused, a good one saved for the team
    await page.locator('#arcviz-theme-menu').click(); await win.waitForTimeout(300);
    await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="theme-edit"]').click(); await win.waitForTimeout(800);
    const editor = page.locator('.wb-theme-editor');
    report('Theme > Edit or new theme opens the editor', await editor.isVisible());
    await page.locator('.wb-te-base').selectOption('preset:capella-classic');
    await page.locator('.wb-te-start button', { hasText: 'Duplicate' }).click(); await win.waitForTimeout(500);
    await page.locator('.wb-te-name').fill('Programme Astra'); await win.waitForTimeout(400);
    const hex = (key, n) => page.locator(`.wb-te-line[data-key="${key}"] input.wb-te-hex`).nth(n);
    await hex('system_actor', 1).fill('#E51400'); await win.waitForTimeout(900);
    const refused = await page.evaluate(() => ({ text: document.querySelector('.wb-te-verdict').textContent, refused: document.querySelector('.wb-te-verdict').classList.contains('is-refused'), disabled: [...document.querySelectorAll('.wb-te-actions button')].every((b) => b.disabled) }));
    report('a reserved colour is refused with the reason, and cannot be saved', refused.refused && /actor stroke #E51400 is the error red/.test(refused.text) && refused.disabled, refused.text.slice(0, 120));
    const preview = await page.evaluate(() => { const g = document.querySelector('.arcviz-stage .av-node[data-kind="logical_component"] > .av-shape'); return g && g.getAttribute('fill'); });
    report('the draft is previewed on the diagram as it is edited', (preview || '').toUpperCase() === '#96B1DA', preview);
    await shot(win, 'vscode-theme-editor-refused');
    await hex('system_actor', 1).fill('#4A4A97');
    await hex('logical_component', 0).fill('#8DA8D6');
    await hex('system_component', 0).fill('#8DA8D6');
    await hex('behaviour_component', 0).fill('#8DA8D6'); await win.waitForTimeout(900);
    const passes = await page.evaluate(() => ({ text: document.querySelector('.wb-te-verdict').textContent, ok: !document.querySelector('.wb-te-verdict').classList.contains('is-refused') }));
    report('with an allowed colour the theme passes every rule', passes.ok, passes.text.slice(0, 120));
    await shot(win, 'vscode-theme-editor-passes');
    await page.locator('.wb-te-actions button', { hasText: 'Save as team theme' }).click(); await win.waitForTimeout(3000);
    const file = path.join(work, '.arclang', 'themes', 'programme-astra.json');
    const saved = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : null;
    report('Save as team theme writes .arclang/themes/programme-astra.json', !!saved && saved.version === 1 && saved.name === 'Programme Astra' && saved.concepts.logical_component.fill === '#8DA8D6', file);
    const setting = JSON.parse(fs.readFileSync(path.join(work, '.vscode', 'settings.json'), 'utf8'))['arclang.diagram.theme'];
    report('the team theme becomes the workspace\'s theme', setting === 'team:programme-astra', setting);
    await page.locator('.wb-te-close').click(); await win.waitForTimeout(1500);
    const drawn = [];
    for (const [name, , kind, concept] of [LAYERS[1], LAYERS[2], LAYERS[3]]) {
      await layer(ctx, name);
      const seen = (await fills(page)).kinds;
      const want = saved.concepts[concept].fill;
      drawn.push(`${name} ${seen[kind]}`);
      if ((seen[kind] || '').toUpperCase() !== want.toUpperCase()) drawn.push('WRONG');
    }
    report('the saved team theme colours every layer', !drawn.includes('WRONG'), drawn.join(', '));
    await layer(ctx, 'Logical');

    // 2. Copy as image: the selection, then the whole view, as a PNG on the macOS clipboard
    execFileSync('/usr/bin/osascript', ['-e', 'set the clipboard to "-"']);
    const shape = await page.locator('.arcviz-stage [data-id="LC-CAM"] > .av-shape').first();
    const frame = await (await page.frameElement()).boundingBox();
    const b = await shape.boundingBox();
    await win.mouse.click(b.x + 20, b.y + 8); await win.waitForTimeout(600);
    const unit = await page.evaluate(() => { const r = document.querySelector('.arcviz-stage [data-id="LC-CAM"] > .av-shape'); return { w: Number(r.getAttribute('width')), h: Number(r.getAttribute('height')) }; });
    await press('Meta+Shift+c'); await win.waitForTimeout(3500);
    const selection = clipboardPng();
    report('⇧⌘C puts the selection on the clipboard as a PNG (clipboard info: «class PNGf»)', !!selection.size, selection.info);
    report('the selection\'s image is 2x the box and its margin', !!selection.size && Math.abs(selection.size.width - (unit.w + 32) * 2) <= 4 && Math.abs(selection.size.height - (unit.h + 32) * 2) <= 4, JSON.stringify(selection.size) + ` for ${unit.w}x${unit.h}`);
    await win.keyboard.press('Escape'); await page.locator('.arcviz-stage').click({ position: { x: 8, y: 8 } }); await win.waitForTimeout(600);
    await ctx.command('ArcLang: Copy the diagram as an image'); await win.waitForTimeout(3500);
    const whole = clipboardPng();
    const view = await page.evaluate(() => { const s = document.querySelector('.arcviz-stage svg.av-svg'); return { w: Number(s.getAttribute('width')), h: Number(s.getAttribute('height')) }; });
    report('with nothing selected the whole view is copied, 2x', !!whole.size && Math.abs(whole.size.width - view.w * 2) <= 4 && Math.abs(whole.size.height - view.h * 2) <= 4, `${JSON.stringify(whole.size)} for ${view.w}x${view.h}`);
    void frame;

    // 3. Copy as SVG
    await page.locator('#arcviz-export-menu').click(); await win.waitForTimeout(300);
    await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="copy-svg"]').click(); await win.waitForTimeout(1500);
    const svgText = await ctx.app.evaluate(({ clipboard }) => clipboard.readText());
    report('Copy as SVG puts the SVG markup on the clipboard, in the theme\'s colours', /^<svg /.test(svgText) && /#8DA8D6/i.test(svgText), svgText.slice(0, 40));

    // 4. Export PowerPoint: every diagram, one slide each
    const pptx = path.join(work, 'deck.pptx');
    await page.locator('#arcviz-export-menu').click(); await win.waitForTimeout(300);
    await page.locator('.arcviz-menu-pop:not([hidden]) [data-action="export-pptx"]').click(); await win.waitForTimeout(1200);
    const quick = win.locator('.quick-input-widget input').first();
    await quick.fill('Every diagram'); await win.waitForTimeout(400); await quick.press('Enter');
    let views = 0;
    for (let i = 0; i < 40; i++) {
      await win.waitForTimeout(1000);
      if (await quick.isVisible().catch(() => false)) {
        const value = await quick.inputValue();
        if (/\.pptx$/.test(value) || value.includes(path.sep)) { await quick.fill(pptx); await quick.press('Enter'); await win.waitForTimeout(2500); break; }
      }
    }
    views = await page.evaluate(() => document.querySelectorAll('.wb-layer').length);
    const exists = fs.existsSync(pptx);
    report('Export PowerPoint writes the file chosen in the save dialog', exists, pptx);
    if (exists && process.env.PPTX_PYTHON) {
      const script = path.join(dir, 'read.py');
      fs.writeFileSync(script, 'import sys\nfrom pptx import Presentation\np = Presentation(sys.argv[1])\nprint(len(p.slides), p.slide_width, p.slide_height)\nfor s in p.slides:\n    pics = [x for x in s.shapes if x.shape_type == 13]\n    print(repr(s.shapes.title.text), len(pics), pics[0].image.size if pics else None, len(s.notes_slide.notes_text_frame.text.splitlines()))\n');
      const out = execFileSync(process.env.PPTX_PYTHON, ['-I', script, pptx], { encoding: 'utf8', cwd: os.tmpdir() });
      const [head, ...slides] = out.trim().split('\n');
      const [count, w, h] = head.split(' ').map(Number);
      report('python-pptx reads one 16:9 slide per diagram, each with a title, a picture and notes', count >= views && count >= 6 && w === 12192000 && h === 6858000 && slides.every((line) => / 1 \(\d+, \d+\) \d+$/.test(line)), `${count} slides for ${views} layers; ${slides.slice(0, 3).join(' | ')}`);
      if (process.env.SHOTS) fs.copyFileSync(pptx, path.join(process.env.SHOTS, 'vscode-export.pptx'));
    }
  } catch (error) {
    report('no error (editor and exports)', false, String(error).slice(0, 300));
    await win.screenshot({ path: path.join(process.env.SHOTS || os.tmpdir(), 'vscode-theme-failure.png') }).catch(() => {});
  } finally {
    await ctx.app.close().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// ONLY=light|dark|editor runs one part.
(async () => {
  const only = process.env.ONLY;
  if (!only || only === 'light') await presets('Default Light Modern', 'light');
  if (!only || only === 'dark') await presets('Default Dark Modern', 'dark');
  if (!only || only === 'editor') await editorAndExports();
  const failed = rows.filter((r) => !r.ok); console.log(`\n${rows.length - failed.length}/${rows.length} checks in VS Code`);
  process.exit(failed.length ? 1 : 0);
})();
