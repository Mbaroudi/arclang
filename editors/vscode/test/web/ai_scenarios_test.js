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

/** The workspace's files as the playground keeps them (IndexedDB, saved on every change): path → text. */
async function storedFiles(page) {
  await page.waitForTimeout(1500);
  return page.evaluate(() => new Promise((resolve) => {
    const open = indexedDB.open('arclang-playground', 1);
    open.onerror = () => resolve({});
    open.onsuccess = () => {
      const store = open.result.transaction('files').objectStore('files');
      const keys = store.getAllKeys();
      const values = store.getAll();
      values.onsuccess = () => resolve(Object.fromEntries(keys.result.map((k, i) => [k, new TextDecoder().decode(values.result[i])])));
      values.onerror = () => resolve({});
    };
  }));
}

/** The text of a file of the workspace, by its name or the end of its path. */
async function fileText(page, name) {
  const files = await storedFiles(page);
  const key = Object.keys(files).find((k) => k === name || k.endsWith(`/${name}`));
  return key ? files[key] : '';
}

/** The file the chat is about (its context line). */
async function chatFile(chat) {
  return (await chat.$eval('.chat-context .chat-file', (n) => n.textContent.trim()).catch(() => '')) || '';
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

/** Workbenches on screen that draw a diagram, and those left blank. */
async function drawnWorkbenches(page) {
  let n = 0;
  for (const f of page.frames()) if (await f.evaluate(() => !!document.querySelector('.wb-problems-head') && document.querySelectorAll('svg g').length > 5).catch(() => false)) n++;
  return n;
}
async function blankWorkbenches(page) {
  let n = 0;
  for (const f of page.frames()) if (await f.evaluate(() => !!document.querySelector('.wb-problems-head') && document.querySelectorAll('svg g').length === 0).catch(() => false)) n++;
  return n;
}

/** Answers in French: a beginner who writes French reads French. */
const french = (text) => (text.match(/\b(le|la|les|vous|votre|est|une|des|pour|dans)\b/gi) ?? []).length >= 5;

/** The error notifications on screen (what stops a visitor). */
async function errorToasts(page) {
  return page.$$eval('.notification-toast', (l) => l.filter((t) => t.querySelector('.codicon-error')).map((t) => t.innerText.replace(/\s+/g, ' ').slice(0, 160))).catch(() => []);
}

/** Asks and times it; the step is logged for the review of the journey. */
async function step(page, chat, journey, question) {
  const started = Date.now();
  const answer = await ask(page, chat, question);
  const seconds = Math.round((Date.now() - started) / 1000);
  journey.push({ question, seconds, text: answer.text });
  console.log(`  · ${seconds}s  « ${question} »`);
  return { ...answer, seconds };
}

/** Answers in English: a visitor who writes English reads English. */
const english = (text) => (text.match(/\b(the|and|is|you|your|to|of|with|this)\b/gi) ?? []).length >= 6 && !french(text.replace(/model \w+ \{[\s\S]*$/, ''));

/** Creates the model of the last answer in one click; the file it opened. */
async function createModel(page, chat) {
  const create = await chat.$('.chat-turn.ai:last-child button:has-text("Create model")');
  if (!create) return '';
  await create.click();
  await page.waitForTimeout(2000);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(9000);
  return chatFile(chat);
}

/** Keeps the change of the last answer when it was written; whether it was. */
async function keep(page, chat) {
  const written = await chat.$('.chat-turn.ai:last-child .chat-proposal.written');
  if (!written) return false;
  await (await chat.$('.chat-turn.ai:last-child button:has-text("Keep")')).click();
  await page.waitForTimeout(2500);
  return true;
}

/** The files the last answer wrote next to the model. */
const writtenFiles = (chat) => chat.$$eval('.chat-turn.ai:last-child .chat-files .chat-link', (l) => l.map((x) => x.textContent)).catch(() => []);

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
    const before = await fileText(page, 'starter.arc');
    const answer = await ask(page, chat, 'modélise moi un téléphone like Nokia 3310');
    await shot(page, 'phone');
    check('phone', 'another system is a new model', !!(await chat.$('.chat-turn.ai:last-child button:has-text("Create model")')), answer.text.slice(0, 80).replace(/\n/g, ' '));
    check('phone', 'the new model compiles', /✓ compiles/.test(answer.text));
    check('phone', 'the open model is not changed', (await fileText(page, 'starter.arc')) === before);
    await context.close();
  },

  // A change to the open model: written in the file, Keep/Undo, Undo puts it back.
  async edit(browser) {
    const { context, page, chat } = await open(browser);
    const before = await fileText(page, 'starter.arc');
    await ask(page, chat, 'Ajoute une exigence REQ-003 : le contrôleur de freinage répond en moins de 50 ms, et trace-la vers le Brake Controller.');
    await shot(page, 'edit');
    check('edit', 'the change is written in the file', !!(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')));
    const after = await fileText(page, 'starter.arc');
    check('edit', 'the file holds REQ-003', /REQ-003/.test(after));
    check('edit', 'Keep / Undo above the change', (await page.$$eval('.codelens-decoration', (l) => l.map((x) => x.textContent).join(' '))).includes('Keep'));
    const undo = await chat.$('.chat-turn.ai:last-child button:has-text("Undo")');
    if (undo) {
      await undo.click();
      await page.waitForTimeout(2000);
      check('edit', 'Undo puts the file back as it was', (await fileText(page, 'starter.arc')) === before);
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

  // Malek's conversation: a one-pedal car, a follow-up that needs the memory, then modes, safety and simulation in one go, then Continue.
  async onepedal(browser) {
    const { context, page, chat } = await open(browser);
    await command(page, 'View: Close All Editors');
    const first = await ask(page, chat, "fait moi une modelisation d'une voiture one-pedale ?");
    check('onepedal', 'a whole model that compiles', /✓ compiles/.test(first.text), first.text.slice(0, 70).replace(/\n/g, ' '));
    const follow = await ask(page, chat, 'oui c un nouveau model');
    check('onepedal', 'the follow-up keeps the subject (no "which system?")', !/quel système|which system/i.test(follow.text), follow.text.slice(0, 90).replace(/\n/g, ' '));
    const create = await chat.$('.chat-turn.ai:last-child button:has-text("Create model")') ?? (await chat.$$('button:has-text("Create model")')).pop();
    if (!create) return void check('onepedal', 'Create model is offered', false);
    await create.click();
    await page.waitForTimeout(2000);
    await page.keyboard.press('Enter');
    await page.waitForTimeout(9000);
    const more = await ask(page, chat, "je veux les modes de conduite à une pédale (accélération, récupération, arrêt), l'analyse de sécurité et la simulation de tout");
    await shot(page, 'onepedal-more');
    check('onepedal', 'written in the file, not asked back', !!(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), more.text.slice(0, 90).replace(/\n/g, ' '));
    const text = await fileText(page, await chatFile(chat));
    check('onepedal', 'the driving modes as a state machine', /state_machine/.test(text));
    check('onepedal', 'a safety analysis with hazards', /safety_analysis[\s\S]*hazard/.test(text));
    const files = await chat.$$eval('.chat-turn.ai:last-child .chat-files .chat-link', (l) => l.map((x) => x.textContent));
    const continued = files.length ? null : await (async () => {
      const button = await chat.$('.chat-turn.ai:last-child button:has-text("Continue")');
      if (!button) return null;
      await button.click();
      await page.waitForTimeout(3000);
      return ask(page, chat, '');
    })();
    const allFiles = files.length ? files : await chat.$$eval('.chat-files .chat-link', (l) => l.map((x) => x.textContent));
    check('onepedal', 'the simulation: its Modelica package written', allFiles.some((f) => /\.mo$/.test(f)), (allFiles.join(', ') || (continued?.text ?? '').slice(0, 80)).replace(/\n/g, ' '));
    const keep = await chat.$('.chat-turn.ai:last-child button:has-text("Keep")');
    if (keep) await keep.click();
    await page.waitForTimeout(2500);
    check('onepedal', 'the model has no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (process.env.SIM_IMAGE && allFiles.length) await simulate(page, chat, allFiles, 'onepedal');
    await context.close();
  },

  // A beginner on a blank page, no account, plain questions: guided from A to Z, never blocked.
  async novice(browser) {
    const { context, page, chat } = await open(browser);
    const journey = [];
    const slow = [];
    const timed = (a) => { if (a.seconds > 120) slow.push(`${a.seconds}s`); return a; };
    await command(page, 'View: Close All Editors');
    check('novice', 'the welcome offers where to start', !!(await chat.$('.chat-starters, .chat-welcome')));

    const hello = timed(await step(page, chat, journey, "bonjour, je n'y connais rien en modélisation système, je veux juste tester l'outil. je fais quoi ?"));
    await shot(page, 'novice-1-hello');
    check('novice', 'hello: an answer in French', french(hello.text), hello.text.slice(0, 120).replace(/\n/g, ' '));
    check('novice', 'hello: a first step to take, not a lecture', /machine|exemple|décri|propos|modèle de|par exemple|essay/i.test(hello.text) && hello.text.length < 2500, `${hello.text.length} chars`);
    check('novice', 'hello: nothing written yet', !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')));

    const make = timed(await step(page, chat, journey, "ok fais moi un modèle d'une machine à café"));
    await shot(page, 'novice-2-model');
    check('novice', 'model: a whole model that compiles', /✓ compiles/.test(make.text), make.text.slice(0, 100).replace(/\n/g, ' '));
    const create = await chat.$('.chat-turn.ai:last-child button:has-text("Create model")');
    check('novice', 'model: Create model in one click', !!create);
    if (create) {
      await create.click();
      await page.waitForTimeout(2000);
      await page.keyboard.press('Enter');
      await page.waitForTimeout(9000);
    }
    const file = await chatFile(chat);
    check('novice', 'model: the file is open and has no error', /\.arc$/.test(file) && (await statusErrors(page)) === 0, `${file}, errors: ${await statusErrors(page)}`);
    const created = Object.entries(await storedFiles(page)).find(([k]) => k.endsWith(`/${file}`));

    const explain = timed(await step(page, chat, journey, "j'ai rien compris, explique moi simplement ce que tu as fait"));
    await shot(page, 'novice-3-explain');
    check('novice', 'explain: in French and plain', french(explain.text) && explain.text.length < 3000, `${explain.text.length} chars`);
    check('novice', 'explain: the model is not changed', created ? (await fileText(page, file)) === created[1] : true);

    const see = timed(await step(page, chat, journey, 'comment je vois le schéma ?'));
    check('novice', 'diagram: names the real command', /Open the workbench beside the source|Open a diagram/i.test(see.text), see.text.slice(0, 160).replace(/\n/g, ' '));
    // What the answer says to do, done as a visitor does it.
    await command(page, 'ArcLang: Open the workbench beside the source');
    await page.waitForTimeout(6000);
    await shot(page, 'novice-4-diagram');
    check('novice', 'diagram: drawn (one workbench, not a blank one)', (await drawnWorkbenches(page)) >= 1 && (await blankWorkbenches(page)) === 0, `drawn ${await drawnWorkbenches(page)}, blank ${await blankWorkbenches(page)}`);

    const add = timed(await step(page, chat, journey, 'ajoute un bouton pour faire un cappuccino'));
    await shot(page, 'novice-5-add');
    const written = await chat.$('.chat-turn.ai:last-child .chat-proposal.written');
    check('novice', 'add: written in the file with Keep / Undo', !!written, add.text.slice(0, 120).replace(/\n/g, ' '));
    if (written) {
      await (await chat.$('.chat-turn.ai:last-child button:has-text("Keep")')).click();
      await page.waitForTimeout(2500);
    }
    check('novice', 'add: still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    check('novice', 'add: the cappuccino is in the model', /cappuccino/i.test(await fileText(page, file)));

    const good = timed(await step(page, chat, journey, 'est ce que mon modèle est bon ?'));
    check('novice', 'review: an answer in French', french(good.text), good.text.slice(0, 160).replace(/\n/g, ' '));

    const sim = timed(await step(page, chat, journey, "simule la chauffe de l'eau"));
    await shot(page, 'novice-6-sim');
    const simWritten = await chat.$('.chat-turn.ai:last-child .chat-proposal.written');
    const simFiles = await chat.$$eval('.chat-turn.ai:last-child .chat-files .chat-link', (l) => l.map((x) => x.textContent)).catch(() => []);
    check('novice', 'simulation: written, with its Modelica package', !!simWritten && simFiles.some((f) => /\.mo$/.test(f)), (simFiles.join(', ') || sim.text.slice(0, 120)).replace(/\n/g, ' '));
    if (simWritten) {
      await (await chat.$('.chat-turn.ai:last-child button:has-text("Keep")')).click();
      await page.waitForTimeout(2500);
    }
    check('novice', 'simulation: still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    check('novice', 'the diagram is still drawn after the AI\'s changes', (await drawnWorkbenches(page)) >= 1 && (await blankWorkbenches(page)) === 0, `drawn ${await drawnWorkbenches(page)}, blank ${await blankWorkbenches(page)}`);
    if (process.env.SIM_IMAGE && simFiles.length) await simulate(page, chat, simFiles, 'novice');

    const run = timed(await step(page, chat, journey, 'comment je lance la simulation ?'));
    check('novice', 'run: the Simulate button of the diagram, and a free account', /Simulate/.test(run.text) && /compte|connect|sign/i.test(run.text) && (!/ArcLang: Run simulations/.test(run.text) || /pas disponible|not available|desktop|poste|ordinateur|VS Code/i.test(run.text)), run.text.slice(0, 200).replace(/\n/g, ' '));
    // The button the answer names, clicked without an account.
    let clicked = false;
    for (const f of page.frames()) {
      const button = await f.$('button:has-text("Simulate")').catch(() => null);
      if (button && !clicked) clicked = !!(await button.click({ force: true }).then(() => true).catch(() => false));
    }
    await page.waitForTimeout(5000);
    await shot(page, 'novice-7-run');
    const notes = await page.$$eval('.notification-toast', (l) => l.map((t) => t.innerText.replace(/\s+/g, ' '))).catch(() => []);
    check('novice', 'run: without an account, a clear sign-in message (no cryptic error)', notes.some((n) => /sign in/i.test(n)) && !notes.some((n) => /undefined|exception|stack|Null Extension/i.test(n)), notes.join(' | ').slice(0, 200));

    const vague = timed(await step(page, chat, journey, 'sa marche pa'));
    check('novice', 'vague: a helpful answer, not an error', french(vague.text) && !/error|erreur interne|failed/i.test(vague.text.slice(0, 80)), vague.text.slice(0, 160).replace(/\n/g, ' '));

    const errors = await errorToasts(page);
    check('novice', 'no error notification during the journey', errors.length === 0, errors.join(' | '));
    check('novice', 'every answer within 2 minutes', slow.length === 0, slow.join(', '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, 'novice-journey.json'), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // The same beginner in English, another product: a visitor from abroad.
  async beginner_en(browser) {
    const { context, page, chat } = await open(browser);
    const journey = [];
    const S = 'beginner_en';
    await command(page, 'View: Close All Editors');
    const hello = await step(page, chat, journey, "Hi, I have never done systems engineering. I just want to try this tool. Where do I start?");
    check(S, 'hello: an answer in English with a first step', english(hello.text) && hello.text.length < 2500, hello.text.slice(0, 120).replace(/\n/g, ' '));
    const make = await step(page, chat, journey, 'Make me a model of an electric scooter');
    await shot(page, 'beginner_en-model');
    check(S, 'model: a whole model that compiles', /✓ compiles/.test(make.text), make.text.slice(0, 100).replace(/\n/g, ' '));
    const file = await createModel(page, chat);
    check(S, 'model: created, open, no error', /\.arc$/.test(file) && (await statusErrors(page)) === 0, `${file}, errors: ${await statusErrors(page)}`);
    const explain = await step(page, chat, journey, "I don't understand any of this. Explain what you did in simple words.");
    check(S, 'explain: plain English, nothing changed', english(explain.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), `${explain.text.length} chars`);
    const add = await step(page, chat, journey, 'Add a headlight that turns on automatically when it gets dark');
    check(S, 'add: written with Keep / Undo', await keep(page, chat), add.text.slice(0, 120).replace(/\n/g, ' '));
    check(S, 'add: still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    const sim = await step(page, chat, journey, 'Can you simulate the battery range?');
    const files = await writtenFiles(chat);
    const simKept = await keep(page, chat);
    check(S, 'simulate: written, with its Modelica package', simKept && files.some((f) => /\.mo$/.test(f)), (files.join(', ') || sim.text.slice(0, 120)).replace(/\n/g, ' '));
    check(S, 'simulate: still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (process.env.SIM_IMAGE && files.length) await simulate(page, chat, files, S);
    const run = await step(page, chat, journey, 'How do I run it?');
    check(S, 'run: the Simulate button, and a free account', /Simulate/.test(run.text) && /sign in|account/i.test(run.text) && (!/ArcLang: Run simulations/.test(run.text) || /not available|desktop|computer|VS Code/i.test(run.text)), run.text.slice(0, 200).replace(/\n/g, ' '));
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Simulation process and data management: requirement → simulation → verdict, results, reports, traceability.
  async spdm(browser) {
    const { context, page, chat } = await open(browser);
    const journey = [];
    const S = 'spdm';
    await command(page, 'View: Close All Editors');
    const how = await step(page, chat, journey, 'I work in simulation process and data management. How does ArcLang tie a simulation result to a requirement?');
    check(S, 'how: test_case, simulate:, expects: and the verdict on the requirement, nothing written', /simulate:/.test(how.text) && /expects/.test(how.text) && /verdict/i.test(how.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), how.text.slice(0, 160).replace(/\n/g, ' '));
    const make = await step(page, chat, journey, 'Build a small model of a battery pack with liquid cooling: requirements on the maximum cell temperature, the components, and a test case that will verify the temperature by simulation.');
    check(S, 'model: compiles', /✓ compiles/.test(make.text), make.text.slice(0, 100).replace(/\n/g, ' '));
    const file = await createModel(page, chat);
    check(S, 'model: created, no error', /\.arc$/.test(file) && (await statusErrors(page)) === 0, `${file}, errors: ${await statusErrors(page)}`);
    const sim = await step(page, chat, journey, 'Now add the simulation so that the requirement gets a verdict.');
    const files = await writtenFiles(chat);
    const kept = await keep(page, chat);
    check(S, 'simulation: written with its Modelica package', kept && files.some((f) => /\.mo$/.test(f)), (files.join(', ') || sim.text.slice(0, 160)).replace(/\n/g, ' '));
    check(S, 'simulation: no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (process.env.SIM_IMAGE && files.length) await simulate(page, chat, files, S);
    const results = await step(page, chat, journey, 'Where are the simulation results kept, and can I export a report of the verdicts?');
    check(S, 'results: names the report export, in English', english(results.text) && /Export the simulation report/.test(results.text), results.text.slice(0, 200).replace(/\n/g, ' '));
    const trace = await step(page, chat, journey, 'How do I see the traceability from requirements to components and tests?');
    check(S, 'traceability: a real view or command, nothing written', /Requirements|traceab|matrix|table/i.test(trace.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), trace.text.slice(0, 200).replace(/\n/g, ' '));
    const reqif = await step(page, chat, journey, 'Can I bring my requirements from DOORS or a ReqIF file?');
    check(S, 'ReqIF: yes, with how', /ReqIF/i.test(reqif.text) && !/^No\b/.test(reqif.text.replace(/^.*?claude-[\w-]+\s*/s, '').trim()), reqif.text.slice(0, 200).replace(/\n/g, ' '));
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Requirements from a spreadsheet export (CSV), in English: the model keeps their ids, then a review.
  async requirements_csv(browser) {
    const S = 'requirements_csv';
    const fixture = path.join(__dirname, '..', 'fixtures', 'chat', 'drone-requirements.csv');
    const { context, page, chat } = await open(browser);
    const journey = [];
    await command(page, 'View: Close All Editors');
    await chat.setInputFiles('input[type=file]', fixture);
    await page.waitForTimeout(3000);
    check(S, 'the CSV is attached', (await chat.$$eval('.chat-attachment', (l) => l.map((x) => x.textContent))).some((t) => /drone-requirements\.csv/.test(t)));
    const answer = await step(page, chat, journey, 'Build the architecture of the drone described by the attached requirements. Keep their ids.');
    check(S, 'the model keeps the four ids', ['DR-001', 'DR-002', 'DR-003', 'DR-004'].every((id) => answer.text.includes(id)), answer.text.slice(0, 100).replace(/\n/g, ' '));
    check(S, 'the model compiles', /✓ compiles/.test(answer.text));
    const file = await createModel(page, chat);
    check(S, 'created, no error', /\.arc$/.test(file) && (await statusErrors(page)) === 0, `${file}, errors: ${await statusErrors(page)}`);
    const gaps = await step(page, chat, journey, 'Which requirements are not yet verified by a test, and what would you add?');
    check(S, 'the gaps: named by id, in English, nothing written', /DR-00\d/.test(gaps.text) && english(gaps.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), gaps.text.slice(0, 160).replace(/\n/g, ' '));
    const add = await step(page, chat, journey, 'Add them.');
    check(S, 'the tests are written with Keep / Undo', await keep(page, chat), add.text.slice(0, 120).replace(/\n/g, ' '));
    check(S, 'still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // An asked method is applied, not explained: CESAM on the open model.
  async cesam_do(browser) {
    const S = 'cesam_do';
    const { context, page, chat } = await open(browser);
    const journey = [];
    const before = (await chatFile(chat)) || 'starter.arc';
    const apply = await step(page, chat, journey, 'Apply the CESAM method to my model.');
    const kept = await keep(page, chat);
    check(S, 'CESAM is written into the model', kept && /method:\s*"cesam"|stakeholder/.test(await fileText(page, before)), apply.text.slice(0, 140).replace(/\n/g, ' '));
    check(S, 'still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // An example opened as a visitor finds it: explained to a manager, then its weak points.
  async example_review(browser) {
    const S = 'example_review';
    const { context, page, chat } = await open(browser);
    const journey = [];
    const manager = await step(page, chat, journey, 'Explain this model to my manager in five lines, no jargon.');
    check(S, 'a short plain answer, nothing written', manager.text.length < 2200 && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), `${manager.text.length} chars`);
    const weak = await step(page, chat, journey, 'What are the weak points of this architecture?');
    check(S, 'weak points named on the real elements', /LC-|REQ-|Threat|Brake/i.test(weak.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), weak.text.slice(0, 160).replace(/\n/g, ' '));
    const fix = await step(page, chat, journey, 'Fix the most important one.');
    check(S, 'the fix is written with Keep / Undo', await keep(page, chat), fix.text.slice(0, 120).replace(/\n/g, ' '));
    check(S, 'still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
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
    check('guide', 'co-simulation: arclang cosim', /cosim/.test(cosim.text) && !/^Non\b/.test(cosim.text.trim()), cosim.text.replace(/\n/g, ' ').slice(0, 300));
    await context.close();
  },
  // ---- The demo exercise: the same words, in the same order, as the guide given to the presenter. ----

  // Part A: requirement → model → simulation → verdict → report → traceability → ReqIF.
  async exercise_a(browser) {
    const S = 'exercise_a';
    const { context, page, chat } = await open(browser);
    const journey = [];
    await command(page, 'View: Close All Editors');
    const how = await step(page, chat, journey, 'I work in simulation process and data management. How does ArcLang tie a simulation result to a requirement?');
    check(S, 'A1 explained (test_case, simulate:, expects:, verdict), nothing written', /simulate:/.test(how.text) && /expects/.test(how.text) && /verdict/i.test(how.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), how.text.slice(0, 120).replace(/\n/g, ' '));
    const make = await step(page, chat, journey, 'Build a small model of a battery pack with liquid cooling: requirements on the maximum cell temperature, the components, and a test case that will verify the temperature by simulation.');
    check(S, 'A2 a whole model that compiles', /✓ compiles/.test(make.text), make.text.slice(0, 100).replace(/\n/g, ' '));
    const file = await createModel(page, chat);
    check(S, 'A2 Create model: open, no error', /\.arc$/.test(file) && (await statusErrors(page)) === 0, `${file}, errors: ${await statusErrors(page)}`);
    await command(page, 'ArcLang: Open the workbench beside the source');
    await page.waitForTimeout(6000);
    await shot(page, 'exercise-a3-diagram');
    check(S, 'A3 the diagram is drawn beside the source', (await drawnWorkbenches(page)) >= 1 && (await blankWorkbenches(page)) === 0, `drawn ${await drawnWorkbenches(page)}, blank ${await blankWorkbenches(page)}`);
    const sim = await step(page, chat, journey, 'Now add the simulation so that the requirement gets a verdict.');
    const files = await writtenFiles(chat);
    const kept = await keep(page, chat);
    check(S, 'A4 simulation written with its Modelica package, kept', kept && files.some((f) => /\.mo$/.test(f)), (files.join(', ') || sim.text.slice(0, 160)).replace(/\n/g, ' '));
    check(S, 'A4 no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    let simulateButton = false;
    for (const f of page.frames()) if (await f.$('button:has-text("Simulate")').catch(() => null)) simulateButton = true;
    check(S, 'A5 the Simulate button is under the diagram', simulateButton);
    if (process.env.SIM_IMAGE && files.length) await simulate(page, chat, files, S);
    const results = await step(page, chat, journey, 'Where are the simulation results kept, and can I export a report of the verdicts?');
    check(S, 'A6 names the report export', /Export the (simulation )?report/i.test(results.text), results.text.slice(0, 200).replace(/\n/g, ' '));
    const trace = await step(page, chat, journey, 'How do I see the traceability from requirements to components and tests?');
    check(S, 'A7 a real view, nothing written', /coverage table|Requirements|traceab|matrix/i.test(trace.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), trace.text.slice(0, 200).replace(/\n/g, ' '));
    await command(page, 'ArcLang: Requirement coverage table');
    await page.waitForTimeout(5000);
    await shot(page, 'exercise-a7-coverage');
    let table = false;
    for (const f of page.frames()) if (await f.evaluate(() => /REQ|BAT|TC-/.test(document.querySelector('table')?.innerText ?? '')).catch(() => false)) table = true;
    check(S, 'A7 the coverage table opens with the requirements', table);
    const reqif = await step(page, chat, journey, 'Can I bring my requirements from DOORS or a ReqIF file, and send them back?');
    check(S, 'A8 ReqIF: yes, import and export', /ReqIF/i.test(reqif.text) && /import/i.test(reqif.text) && /export|save/i.test(reqif.text) && !/^No\b/.test(reqif.text.replace(/^.*?claude-[\w-]+\s*/s, '').trim()), reqif.text.slice(0, 200).replace(/\n/g, ' '));
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Part B: requirements given as a Word file, then as a CSV export.
  async exercise_b(browser) {
    const S = 'exercise_b';
    const { context, page, chat } = await open(browser);
    const journey = [];
    await command(page, 'View: Close All Editors');
    await chat.setInputFiles('input[type=file]', path.join(__dirname, '..', 'fixtures', 'chat', 'requirements.docx'));
    await page.waitForTimeout(3000);
    check(S, 'B1 the Word file is attached', (await chat.$$eval('.chat-attachment', (l) => l.map((x) => x.textContent))).some((t) => /requirements\.docx/.test(t)));
    const word = await step(page, chat, journey, 'Build the model of the system described by the attached requirements. Keep their ids.');
    check(S, 'B1 the model keeps REQ-1 and REQ-2 and compiles', /REQ-1/.test(word.text) && /REQ-2/.test(word.text) && /✓ compiles/.test(word.text), word.text.slice(0, 100).replace(/\n/g, ' '));
    const radio = await createModel(page, chat);
    check(S, 'B1 created, no error', /\.arc$/.test(radio) && (await statusErrors(page)) === 0, `${radio}, errors: ${await statusErrors(page)}`);
    const plus = await chat.$('.chat-clear, button[title*="New conversation"], .chat-new');
    if (plus) await plus.click();
    await page.waitForTimeout(1500);
    await chat.setInputFiles('input[type=file]', path.join(__dirname, '..', 'fixtures', 'chat', 'drone-requirements.csv'));
    await page.waitForTimeout(3000);
    check(S, 'B2 the CSV is attached', (await chat.$$eval('.chat-attachment', (l) => l.map((x) => x.textContent))).some((t) => /drone-requirements\.csv/.test(t)));
    const csv = await step(page, chat, journey, 'Build the architecture of the drone described by the attached requirements. Keep their ids.');
    check(S, 'B2 the model keeps DR-001 to DR-004 and compiles', ['DR-001', 'DR-002', 'DR-003', 'DR-004'].every((id) => csv.text.includes(id)) && /✓ compiles/.test(csv.text), csv.text.slice(0, 100).replace(/\n/g, ' '));
    const drone = await createModel(page, chat);
    check(S, 'B2 created, no error', /\.arc$/.test(drone) && (await statusErrors(page)) === 0, `${drone}, errors: ${await statusErrors(page)}`);
    const gaps = await step(page, chat, journey, 'Which requirements are not yet verified by a test, and what would you add?');
    check(S, 'B3 the gaps named by id, nothing written', /DR-00\d/.test(gaps.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), gaps.text.slice(0, 160).replace(/\n/g, ' '));
    const add = await step(page, chat, journey, 'Add them.');
    check(S, 'B4 the tests written with Keep / Undo, kept', await keep(page, chat), add.text.slice(0, 120).replace(/\n/g, ' '));
    check(S, 'B4 no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Part C: the example open at start: a change undone, explained, reviewed, fixed, CESAM applied.
  async exercise_c(browser) {
    const S = 'exercise_c';
    const { context, page, chat } = await open(browser);
    const journey = [];
    const before = await fileText(page, 'starter.arc');
    await step(page, chat, journey, 'Add a requirement REQ-003: the brake controller responds in less than 50 ms, and trace it to the Brake Controller.');
    check(S, 'C1 written in the file', !!(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')) && /REQ-003/.test(await fileText(page, 'starter.arc')));
    check(S, 'C1 Keep / Undo above the change in the editor', (await page.$$eval('.codelens-decoration', (l) => l.map((x) => x.textContent).join(' '))).includes('Keep'));
    await shot(page, 'exercise-c1-keep-undo');
    const undo = await chat.$('.chat-turn.ai:last-child button:has-text("Undo")');
    if (undo) await undo.click();
    await page.waitForTimeout(2500);
    check(S, 'C2 Undo puts the file back as it was', (await fileText(page, 'starter.arc')) === before);
    const manager = await step(page, chat, journey, 'Explain this model to my manager in five lines, no jargon.');
    check(S, 'C3 short and plain, nothing written', manager.text.length < 2200 && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), `${manager.text.length} chars`);
    const weak = await step(page, chat, journey, 'What are the weak points of this architecture?');
    check(S, 'C4 weak points on the real elements, nothing written', /LC-|REQ-|Brake|TC-/i.test(weak.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), weak.text.slice(0, 160).replace(/\n/g, ' '));
    const fix = await step(page, chat, journey, 'Fix the most important one.');
    check(S, 'C5 the fix written, kept', await keep(page, chat), fix.text.slice(0, 120).replace(/\n/g, ' '));
    check(S, 'C5 no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    const cesam = await step(page, chat, journey, 'Apply the CESAM method to my model.');
    const kept = await keep(page, chat);
    check(S, 'C6 CESAM written into the model, kept', kept && /method:\s*"cesam"|stakeholder/.test(await fileText(page, 'starter.arc')), cesam.text.slice(0, 140).replace(/\n/g, ' '));
    check(S, 'C6 no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Part D: from nothing, three systems: a one-pedal car (with its simulation), a calculator in French, a phone.
  async exercise_d(browser) {
    const S = 'exercise_d';
    const { context, page, chat } = await open(browser);
    const journey = [];
    await command(page, 'View: Close All Editors');
    const car = await step(page, chat, journey, 'Model a one-pedal electric car, as a new model.');
    check(S, 'D1 a whole model that compiles', /✓ compiles/.test(car.text), car.text.slice(0, 100).replace(/\n/g, ' '));
    const carFile = await createModel(page, chat);
    check(S, 'D1 created, no error', /\.arc$/.test(carFile) && (await statusErrors(page)) === 0, `${carFile}, errors: ${await statusErrors(page)}`);
    const more = await step(page, chat, journey, 'Add the one-pedal driving modes (accelerate, regenerate, stop), the safety analysis and the simulation of everything.');
    const files = await writtenFiles(chat);
    const kept = await keep(page, chat);
    const text = await fileText(page, carFile);
    check(S, 'D2 written and kept', kept, more.text.slice(0, 120).replace(/\n/g, ' '));
    check(S, 'D2 modes, hazards and the Modelica package', /state_machine/.test(text) && /safety_analysis[\s\S]*hazard/.test(text) && files.some((f) => /\.mo$/.test(f)), files.join(', '));
    check(S, 'D2 no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (process.env.SIM_IMAGE && files.length) await simulate(page, chat, files, S);
    const plus = await chat.$('.chat-clear, button[title*="New conversation"], .chat-new');
    if (plus) await plus.click();
    await page.waitForTimeout(1500);
    const calc = await step(page, chat, journey, 'modélise une calculatrice scientifique');
    check(S, 'D3 in French, a model that compiles', french(calc.text) && /✓ compiles/.test(calc.text), calc.text.slice(0, 100).replace(/\n/g, ' '));
    const calcFile = await createModel(page, chat);
    check(S, 'D3 created, no error', /\.arc$/.test(calcFile) && (await statusErrors(page)) === 0, `${calcFile}, errors: ${await statusErrors(page)}`);
    const calcSim = await step(page, chat, journey, 'rajoute la simulation openmodelica');
    const calcFiles = await writtenFiles(chat);
    check(S, 'D4 the simulation written, kept', (await keep(page, chat)) && calcFiles.some((f) => /\.mo$/.test(f)), (calcFiles.join(', ') || calcSim.text.slice(0, 120)).replace(/\n/g, ' '));
    check(S, 'D4 no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    if (process.env.SIM_IMAGE && calcFiles.length) await simulate(page, chat, calcFiles, `${S}_calc`);
    const openBefore = await fileText(page, calcFile);
    const phone = await step(page, chat, journey, 'Model a phone like the Nokia 3310');
    check(S, 'D5 another system is a new model that compiles', /✓ compiles/.test(phone.text) && !!(await chat.$('.chat-turn.ai:last-child button:has-text("Create model")')), phone.text.slice(0, 100).replace(/\n/g, ' '));
    check(S, 'D5 the open model is not changed', (await fileText(page, calcFile)) === openBefore);
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Part E: questions on the tooling: guided answers with the real commands, never "no".
  async exercise_e(browser) {
    const S = 'exercise_e';
    const { context, page, chat } = await open(browser);
    const journey = [];
    const embedded = await step(page, chat, journey, 'How do I generate embedded C code from my model and test it?');
    check(S, 'E1 embedded C: the export and SIL', /export/i.test(embedded.text) && /\bsil\b/i.test(embedded.text), embedded.text.slice(0, 120).replace(/\n/g, ' '));
    const cosim = await step(page, chat, journey, 'Can I run a co-simulation with OMSimulator?');
    check(S, 'E2 co-simulation: arclang cosim', /cosim/.test(cosim.text) && !/^No\b/.test(cosim.text.replace(/^.*?claude-[\w-]+\s*/s, '').trim()), cosim.text.slice(0, 160).replace(/\n/g, ' '));
    const ci = await step(page, chat, journey, 'Can I run the checks and the simulations in my CI pipeline?');
    check(S, 'E3 CI: yes, with how', /\bCI\b|pipeline|GitHub|GitLab/i.test(ci.text) && /arclang (check|verify|ci)/i.test(ci.text), ci.text.slice(0, 160).replace(/\n/g, ' '));
    const cesam = await step(page, chat, journey, 'What is the CESAM method and how do I use it here?');
    check(S, 'E4 CESAM: explained, nothing written', /cesam/i.test(cesam.text) && /stakeholder/i.test(cesam.text) && !(await chat.$('.chat-turn.ai:last-child .chat-proposal.written')), cesam.text.slice(0, 160).replace(/\n/g, ' '));
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // Asked how to use a method or a notation, the engineer gets it done: CESAM applied, a SysML v2 model written and drawn.
  async apply_methods(browser) {
    const S = 'apply_methods';
    const { context, page, chat } = await open(browser);
    const journey = [];
    const cesam = await step(page, chat, journey, 'What is the CESAM method and how do I use it here?');
    const kept = await keep(page, chat);
    check(S, 'CESAM: applied to the model with Keep / Undo, not a tutorial', kept && /method:\s*"cesam"/.test(await fileText(page, 'starter.arc')) && cesam.text.length < 2500, `${cesam.text.length} chars: ${cesam.text.replace(/^.*?claude-[\w-]+( · \S+)?\s*/s, '').slice(0, 140).replace(/\n/g, ' ')}`);
    check(S, 'CESAM: still no error', (await statusErrors(page)) === 0, `errors: ${await statusErrors(page)}`);
    const get = await step(page, chat, journey, 'How do I get my model in SysML v2?');
    const files = await writtenFiles(chat);
    const keptSysml = await keep(page, chat);
    check(S, 'SysML v2 of the open model: starter.sysml written beside it by the compiler', keptSysml && files.includes('starter.sysml') && /\bpackage\b|\bpart\b/.test(await fileText(page, 'starter.sysml')), (files.join(', ') || get.text.slice(0, 140)).replace(/\n/g, ' '));
    const plus = await chat.$('.chat-clear, button[title*="New conversation"], .chat-new');
    if (plus) await plus.click();
    await page.waitForTimeout(1500);
    await command(page, 'View: Close All Editors');
    const make = await step(page, chat, journey, 'How do I create a SysML v2 model of a smart thermostat?');
    const blocks = await chat.$$eval('.chat-turn.ai:last-child pre', (l) => l.map((x) => x.innerText));
    check(S, 'new SysML v2 model: its SysML v2 text is shown, written by the compiler', /✓ compiles/.test(make.text) && blocks.some((b) => /^\s*(?:\/\/[^\n]*\n\s*)*(package|part|requirement|doc)\b/m.test(b) && !/^\s*model\b/.test(b)), make.text.slice(0, 140).replace(/\n/g, ' '));
    const create = await chat.$('.chat-turn.ai:last-child button:has-text("Create model")');
    if (create) {
      await create.click();
      await page.waitForTimeout(2000);
      await page.keyboard.press('Enter');
      await page.waitForTimeout(10000);
    }
    const stored = Object.keys(await storedFiles(page)).filter((k) => k.endsWith('.sysml') && !k.endsWith('starter.sysml'));
    await shot(page, 'apply-sysml-drawn');
    check(S, 'Create model writes the .sysml and draws it in the workbench', stored.length > 0 && (await drawnWorkbenches(page)) >= 1 && (await blankWorkbenches(page)) === 0, `${stored.join(', ')}; drawn ${await drawnWorkbenches(page)}, blank ${await blankWorkbenches(page)}`);
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    if (SHOTS) fs.writeFileSync(path.join(SHOTS, `${S}-journey.json`), JSON.stringify(journey, null, 2));
    await context.close();
  },

  // One model, three readings, no AI: Arcadia, SysML v2 and CESAM each show only their own views; CESAM opens on its grid.
  async notations(browser) {
    const S = 'notations';
    const { context, page } = await open(browser);
    await command(page, 'ArcLang: Open the workbench beside the source');
    await page.waitForTimeout(7000);
    let wb = null;
    for (const f of page.frames()) if (await f.$('.wb-notation').catch(() => null)) wb = f;
    check(S, 'the workbench has the notation switch', !!wb);
    if (!wb) return void (await context.close());
    const tabs = () => wb.$$eval('.wb-layers .wb-layer .wb-layer-name', (l) => l.map((x) => x.textContent.trim()));
    let t = await tabs();
    check(S, 'Arcadia: no SysML view among its tabs', !t.some((x) => /General View|Interconnection|Action Flow/.test(x)), t.join(' | '));
    await wb.click('.wb-notation-item:has-text("SysML v2")');
    await page.waitForTimeout(2500);
    t = await tabs();
    check(S, 'SysML v2: its views only', t.includes('General View') && !t.some((x) => /^(Operational|System|Logical|Physical)$/.test(x)), t.join(' | '));
    await wb.click('.wb-notation-item:has-text("CESAM")');
    await page.waitForTimeout(2500);
    const cells = await wb.$$eval('.wb-cesam:not([hidden]) .wb-cesam-cell', (l) => l.length);
    t = await tabs();
    await shot(page, 'notations-cesam-grid');
    check(S, 'CESAM: the grid (3 × 6) and the vision tabs only', cells === 18 && t[0] === 'Grid' && !t.some((x) => /^(System|Logical|General View)$/.test(x)), `${cells} cells; ${t.join(' | ')}`);
    const cell = await wb.$('.wb-cesam-open');
    if (cell) await cell.click();
    await page.waitForTimeout(2500);
    const caption = await wb.$eval('.wb-cesam-caption:not([hidden])', (x) => x.textContent).catch(() => '');
    check(S, 'CESAM: a cell opens its diagram, captioned with its view', !!cell && caption.length > 0 && (await drawnWorkbenches(page)) >= 1, caption);
    await wb.click('.wb-notation-item:has-text("Arcadia")');
    await page.waitForTimeout(2000);
    check(S, 'back to Arcadia: no grid, no caption', await wb.evaluate(() => document.querySelector('.wb-cesam').hidden && document.querySelector('.wb-cesam-caption').hidden));
    const errors = await errorToasts(page);
    check(S, 'no error notification', errors.length === 0, errors.join(' | '));
    await context.close();
  },
};

/** The model and the files the AI wrote, simulated by OpenModelica in the image the server runs. */
async function simulate(page, chat, files, scenario = 'calc') {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arclang-ai-sim-'));
  // The model and the files the AI wrote, as the playground keeps them.
  const stored = await storedFiles(page);
  const model = await chatFile(chat);
  const modelKey = Object.keys(stored).find((k) => k.endsWith(`/${model}`));
  if (!modelKey) return void check(scenario, 'OpenModelica runs the simulation the AI wrote', false, `no ${model} in the workspace`);
  const folder = modelKey.slice(0, modelKey.lastIndexOf('/'));
  fs.writeFileSync(path.join(dir, 'model.arc'), stored[modelKey]);
  for (const file of files) {
    const text = stored[`${folder}/${file}`];
    if (text === undefined) continue;
    fs.mkdirSync(path.join(dir, path.dirname(file)), { recursive: true });
    fs.writeFileSync(path.join(dir, file), text);
  }
  let out = '';
  try {
    out = execFileSync('docker', ['run', '--rm', '--platform', 'linux/amd64', '-v', `${dir}:/job`, '-w', '/job', process.env.SIM_IMAGE, 'arclang', 'verify', 'model.arc'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 600000 });
  } catch (error) {
    out = `${error.stdout ?? ''}${error.stderr ?? ''}`;
  }
  const ran = /Simulated in/.test(out);
  check(scenario, 'OpenModelica runs the simulation the AI wrote', ran, ran ? (out.match(/\d+ verdict\(s\)[^\n]*/) ?? [''])[0] : out.split('\n').filter((l) => /error|Error/.test(l)).slice(0, 2).join(' | ').slice(0, 200));
  // A run is not enough: each case must end in a verdict (pass or fail), not in an error of the expects.
  const cacheDir = path.join(dir, '.arclang', 'sim-cache');
  const verdicts = fs.existsSync(cacheDir) ? fs.readdirSync(cacheDir).filter((f) => f.endsWith('.json')).map((f) => JSON.parse(fs.readFileSync(path.join(cacheDir, f), 'utf8'))) : [];
  const errors = verdicts.filter((v) => v.verdict === 'error');
  if (ran) check(scenario, 'every simulated case ends in a verdict, not an error', verdicts.length > 0 && errors.length === 0, verdicts.map((v) => `${v.test_case}: ${v.verdict}${v.reason ? ` (${v.reason.slice(0, 90)})` : ''}`).join('; '));
  if (SHOTS) fs.cpSync(dir, path.join(SHOTS, `${scenario}-files`), { recursive: true });
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
