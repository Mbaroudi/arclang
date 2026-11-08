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
    check('novice', 'run: the Simulate button of the diagram, and a free account', /Simulate/.test(run.text) && /compte|connect|sign/i.test(run.text) && !/(« ArcLang: Run simulations »|"ArcLang: Run simulations")(?![^.]*(pas disponible|not available|desktop|poste|ordinateur))/.test(run.text), run.text.slice(0, 200).replace(/\n/g, ' '));
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
    check(S, 'run: the Simulate button, and a free account', /Simulate/.test(run.text) && /sign in|account/i.test(run.text) && !/"ArcLang: Run simulations"(?![^.]*(not available|desktop|computer))/.test(run.text), run.text.slice(0, 200).replace(/\n/g, ' '));
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
