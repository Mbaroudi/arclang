// What a visitor asks the AI in the playground, played against the REAL
// assistant (the hosted gateway), each scenario checked on its outcome: the
// model compiles, the change is written with Keep/Undo, another system is a
// new model, the Modelica file is written and OpenModelica runs it. Run before
// a release and on arclang.fr after a deploy, so that a gap is found here and
// not by a visitor.
//   BASE=https://arclang.fr/playground/vscode/ PLAYWRIGHT=.../playwright node test/web/ai_scenarios_test.js
//   SIM_IMAGE=arclang-omc-test also simulates the model the AI wrote (docker, OpenModelica).
//   ONLY=calc,phone runs some scenarios; SHOTS=dir keeps a screenshot of each.
const { chromium } = require(process.env.PLAYWRIGHT || 'playwright');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const BASE = process.env.BASE || 'http://127.0.0.1:8767/playground/vscode/';
const SHOTS = process.env.SHOTS;
const ONLY = process.env.ONLY ? process.env.ONLY.split(',') : null;
const rows = [];
const check = (scenario, name, ok, detail = '') => {
  rows.push({ scenario, name, ok });
  console.log(`${ok ? 'ok  ' : 'FAIL'} [${scenario}] ${name.padEnd(60)} ${detail}`.trimEnd());
};

async function open(browser) {
  const context = await browser.newContext({ viewport: { width: 1600, height: 1000 }, permissions: ['clipboard-read', 'clipboard-write'] });
  const page = await context.newPage();
  await page.goto(BASE, { waitUntil: 'domcontentloaded' });
  let chat = null;
  for (let i = 0; i < 60 && !chat; i++) {
    await page.waitForTimeout(1000);
    for (const f of page.frames()) if (await f.evaluate(() => !!document.querySelector('.chat-composer')).catch(() => false)) chat = f;
  }
  if (!chat) throw new Error('the chat did not open');
  await page.waitForTimeout(4000);
  return { context, page, chat };
}

async function command(page, title) {
  await page.keyboard.press('F1');
  await page.waitForTimeout(600);
  await page.keyboard.type(title);
  await page.waitForTimeout(700);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(1500);
}

/** Sends a question, accepts the hosted provider once, waits until the answer is done; the answer's text. */
async function ask(page, chat, question) {
  const before = await chat.$$eval('.chat-turn.ai', (l) => l.length);
  await chat.fill('textarea', question);
  await chat.press('textarea', 'Enter');
  await page.waitForTimeout(2500);
  const consent = page.getByRole('button', { name: /Send and do not ask again/ });
  if (await consent.count()) await consent.first().click().catch(() => {});
  for (let i = 0; i < 120; i++) {
    await page.waitForTimeout(3000);
    const busy = await chat.evaluate(() => !document.querySelector('.chat-stop')?.hasAttribute('hidden'));
    if (!busy) break;
  }
  await page.waitForTimeout(2500);
  const turns = await chat.$$('.chat-turn.ai');
  const last = turns[turns.length - 1];
  return { text: last && turns.length > before ? await last.innerText() : '', node: last };
}

/**
 * The text of a text editor (the first one is the .arc; the diagrams are
 * beside it): read page by page from what the editor draws, each line by its
 * number, from the top to the bottom.
 */
async function editorText(page, nth = 0) {
  await page.keyboard.press('Escape');
  const editors = await page.$$('.part.editor .monaco-editor');
  const editor = editors[nth < 0 ? editors.length + nth : nth];
  if (!editor) return '';
  await (await editor.$('.view-lines')).click({ position: { x: 60, y: 8 }, force: true, timeout: 10000 });
  await page.keyboard.press('Meta+ArrowUp');
  // Read from the top: wait until the editor draws line 1 (a slow page scrolls late).
  for (let i = 0; i < 30; i++) {
    const top = await editor.evaluate((root) => [...root.querySelectorAll('.margin-view-overlays .line-numbers')].some((n) => n.textContent.trim() === '1'));
    if (top) break;
    await page.keyboard.press('Meta+ArrowUp');
    await page.waitForTimeout(200);
  }
  const lines = new Map();
  for (let round = 0; round < 400; round++) {
    await page.waitForTimeout(120);
    const seen = await editor.evaluate((root) => {
      const numbers = new Map([...root.querySelectorAll('.margin-view-overlays .line-numbers')].map((n) => [n.parentElement.style.top, Number(n.textContent.trim())]));
      return [...root.querySelectorAll('.view-lines .view-line')].map((l) => [numbers.get(l.style.top), l.textContent.replace(/ /g, ' ')]).filter(([n]) => Number.isFinite(n));
    });
    const before = lines.size;
    for (const [n, text] of seen) lines.set(n, text);
    if (lines.size === before && round > 0) break;
    await page.keyboard.press('PageDown');
  }
  await page.keyboard.press('Meta+ArrowUp');
  return [...lines.entries()].sort((a, b) => a[0] - b[0]).map(([, text]) => text).join('\n');
}

async function statusErrors(page) {
  // The Problems item of the status bar: its first number is the errors.
  const text = await page.evaluate(() => document.getElementById('status.problems')?.textContent ?? '');
  const numbers = text.match(/\d+/g);
  return numbers ? Number(numbers[0]) : null;
}

async function shot(page, name) {
  if (SHOTS) await page.screenshot({ path: path.join(SHOTS, `${name}.png`) });
}

const SCENARIOS = {
  // No file open: a new model, compiled before it is shown, created in one click.
  async calc(browser) {
    const { context, page, chat } = await open(browser);
    await command(page, 'View: Close All Editors');
    const answer = await ask(page, chat, 'modelise une calculatrice scientifique');
    const block = await chat.$('.chat-turn.ai:last-child .chat-code');
    check('calc', 'a whole model comes back', !!block, answer.text.slice(0, 80).replace(/\n/g, ' '));
    check('calc', 'the compiler accepted it before it is shown', /✓ compiles/.test(answer.text));
    const create = await chat.$('.chat-turn.ai:last-child button:has-text("Create model")');
    check('calc', 'Create model is offered', !!create);
    if (create) {
      await create.click();
      await page.waitForTimeout(2000);
      await page.keyboard.press('Enter');
      await page.waitForTimeout(9000);
      check('calc', 'the created model has no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    }
    // Then the simulation, asked as a visitor asks it.
    const sim = await ask(page, chat, 'rajoute la simulation openmodelica');
    await shot(page, 'calc-simulation');
    const written = await chat.$('.chat-turn.ai:last-child .chat-proposal.written');
    check('calc', 'the simulation is added, not explained', !!written, sim.text.slice(0, 100).replace(/\n/g, ' '));
    const files = await chat.$$eval('.chat-turn.ai:last-child .chat-files .chat-link', (l) => l.map((x) => x.textContent));
    check('calc', 'the Modelica package is written next to the model', files.some((f) => /\.mo$/.test(f)), files.join(', '));
    if (written) {
      await (await chat.$('.chat-turn.ai:last-child button:has-text("Keep")')).click();
      await page.waitForTimeout(2500);
      check('calc', 'the model still has no error after the simulation', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
      if (process.env.SIM_IMAGE && files.length) await simulate(page, chat, files);
    }
    await context.close();
  },

  // A model open, another system asked: a new model, the open one untouched.
  async phone(browser) {
    const { context, page, chat } = await open(browser);
    const before = await editorText(page);
    const answer = await ask(page, chat, 'modélise moi un téléphone like Nokia 3310');
    await shot(page, 'phone');
    check('phone', 'another system is a new model', !!(await chat.$('.chat-turn.ai:last-child button:has-text("Create model")')), answer.text.slice(0, 80).replace(/\n/g, ' '));
    check('phone', 'the new model compiles', /✓ compiles/.test(answer.text));
    check('phone', 'the open model is not changed', (await editorText(page)) === before);
    await context.close();
  },

  // A change to the open model: written in the file, Keep/Undo, Undo puts it back.
  async edit(browser) {
    const { context, page, chat } = await open(browser);
    const before = await editorText(page);
    await ask(page, chat, 'Ajoute une exigence REQ-003 : le contrôleur de freinage répond en moins de 50 ms, et trace-la vers le Brake Controller.');
    await shot(page, 'edit');
    check('edit', 'the change is written in the file', !!(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')));
    const after = await editorText(page);
    check('edit', 'the file holds REQ-003', /REQ-003/.test(after));
    check('edit', 'Keep / Undo above the change', (await page.$$eval('.codelens-decoration', (l) => l.map((x) => x.textContent).join(' '))).includes('Keep'));
    const undo = await chat.$('.chat-turn.ai:last-child button:has-text("Undo")');
    if (undo) {
      await undo.click();
      await page.waitForTimeout(2000);
      check('edit', 'Undo puts the file back as it was', (await editorText(page)) === before);
    }
    await context.close();
  },

  // Requirements attached: they become the model's, ids kept.
  async requirements(browser) {
    const fixture = path.join(__dirname, '..', 'fixtures', 'chat', 'requirements.docx');
    const { context, page, chat } = await open(browser);
    await command(page, 'View: Close All Editors');
    await chat.setInputFiles('input[type=file]', fixture);
    await page.waitForTimeout(3000);
    check('requirements', 'the Word file is attached', (await chat.$$eval('.chat-attachment', (l) => l.map((x) => x.textContent))).some((t) => /requirements\.docx/.test(t)));
    const answer = await ask(page, chat, 'Construis le modèle du système décrit par les exigences jointes, en gardant leurs ids.');
    check('requirements', 'the model keeps the requirement ids', /REQ-1/.test(answer.text) && /REQ-2/.test(answer.text));
    check('requirements', 'the model compiles', /✓ compiles/.test(answer.text));
    await context.close();
  },

  // Questions: guided answers, with the real commands, never "no".
  async guide(browser) {
    const { context, page, chat } = await open(browser);
    const embedded = await ask(page, chat, 'comment générer du code embarqué en C depuis mon modèle et le tester ?');
    check('guide', 'embedded C: the export and SIL', /export/i.test(embedded.text) && /\bsil\b/i.test(embedded.text), embedded.text.slice(0, 60).replace(/\n/g, ' '));
    const cesam = await ask(page, chat, 'comment appliquer la méthode CESAM sur mon modèle ?');
    check('guide', 'CESAM: method "cesam", stakeholders, the grid', /cesam/.test(cesam.text) && /stakeholder/.test(cesam.text));
    const cosim = await ask(page, chat, 'peut-on faire une co-simulation dans OMSimulator ?');
    check('guide', 'co-simulation: arclang cosim', /cosim/.test(cosim.text) && !/^Non\b/.test(cosim.text.trim()));
    await context.close();
  },
};

/** The model and the files the AI wrote, simulated by OpenModelica in the image the server runs. */
async function simulate(page, chat, files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-ai-sim-'));
  // The model is the editor's text; the files are opened from the chat card one by one.
  fs.writeFileSync(path.join(dir, 'model.arc'), await editorText(page, 0));
  for (const file of files) {
    const link = await chat.$(`.chat-files .chat-link:has-text("${file}")`);
    if (!link) continue;
    await link.scrollIntoViewIfNeeded().catch(() => {});
    await link.click({ force: true, timeout: 10000 });
    await page.waitForTimeout(2500);
    fs.mkdirSync(path.join(dir, path.dirname(file)), { recursive: true });
    // The file opens beside: the last text editor.
    fs.writeFileSync(path.join(dir, file), await editorText(page, -1));
  }
  let out = '';
  try {
    out = execFileSync('docker', ['run', '--rm', '--platform', 'linux/amd64', '-v', `${dir}:/job`, '-w', '/job', process.env.SIM_IMAGE, 'arclang', 'verify', 'model.arc'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 600000 });
  } catch (error) {
    out = `${error.stdout ?? ''}${error.stderr ?? ''}`;
  }
  const ran = /Simulated in/.test(out);
  check('calc', 'OpenModelica runs the simulation the AI wrote', ran, ran ? (out.match(/\d+ verdict\(s\)[^\n]*/) ?? [''])[0] : out.split('\n').filter((l) => /error|Error/.test(l)).slice(0, 2).join(' | ').slice(0, 200));
  if (SHOTS) fs.cpSync(dir, path.join(SHOTS, 'calc-files'), { recursive: true });
}

(async () => {
  const browser = await chromium.launch({ channel: 'chrome' });
  for (const [name, run] of Object.entries(SCENARIOS)) {
    if (ONLY && !ONLY.includes(name)) continue;
    try {
      await run(browser);
    } catch (error) {
      check(name, 'the scenario ran', false, error.message.split('\n')[0]);
    }
  }
  await browser.close();
  const failed = rows.filter((r) => !r.ok);
  console.log(`\n${rows.length - failed.length}/${rows.length} checks passed${failed.length ? `; failed: ${failed.map((r) => `[${r.scenario}] ${r.name}`).join('; ')}` : ''}`);
  process.exit(failed.length ? 1 : 0);
})();
