// What the AI is told ArcLang can do: it guides (simulation, embedded code,
// twins...), says what runs where the engineer is, and names only commands
// the extension has.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { capabilitiesGuide } from '../../src/ai/core/capabilities';
import { SYSTEM_RULES, systemRulesFor } from '../../src/ai/core/prompts';

const pkg = JSON.parse(readFileSync(join(__dirname, '../../package.json'), 'utf8'));
const titles = new Set<string>(pkg.contributes.commands.map((c: { title: string; category?: string }) => `${c.category ? `${c.category}: ` : ''}${c.title}`));

test('every command the guide names is a command of the extension', () => {
  for (const workspace of ['desktop', 'online', 'online-anonymous'] as const) {
    const named = [...capabilitiesGuide(workspace).matchAll(/"(ArcLang: [^"]+)"/g)].map((m) => m[1]);
    assert.ok(named.length >= 15, `the guide names ${named.length} commands`);
    for (const title of named) assert.ok(titles.has(title), `"${title}" is no command of the extension`);
  }
});

test('the guide says how to simulate with OpenModelica and verify a requirement', () => {
  const guide = capabilitiesGuide('online');
  assert.match(guide, /physics: electrical/);
  assert.match(guide, /simulate: \{ model:/);
  assert.match(guide, /expects:/);
  assert.match(guide, /"ArcLang: Run simulations"/);
  assert.match(guide, /never "no" for something listed here/);
});

test('what runs where: the desktop, arclang.fr signed in, arclang.fr anonymous', () => {
  assert.match(capabilitiesGuide('desktop'), /OpenModelica on this computer/);
  assert.match(capabilitiesGuide('desktop'), /ArcLang Studio/);
  const online = capabilitiesGuide('online');
  assert.match(online, /runs on the server \(OpenModelica/);
  assert.match(online, /compiling and running the code \(sil, pil, hdl, soc, cosim\) needs the desktop/);
  assert.doesNotMatch(online, /sign in/i);
  const anonymous = capabilitiesGuide('online-anonymous');
  assert.match(anonymous, /needs the visitor to sign in on arclang\.fr/);
  assert.match(anonymous, /not signed in/);
});

test('the embedded chain is in the guide', () => {
  const guide = capabilitiesGuide('desktop');
  for (const word of ['MISRA C 2012', 'VHDL', 'Zephyr', 'FMU', 'execute:', 'arclang sil', 'arclang pil', 'arclang cosim', '"ArcLang: Realization ladder: what each stage says of each requirement"']) {
    assert.ok(guide.includes(word), `missing ${word}`);
  }
});

test('the system rules keep their rules and gain the guide', () => {
  const rules = systemRulesFor('online');
  assert.ok(rules.startsWith(SYSTEM_RULES));
  assert.match(rules, /# What ArcLang can do, and how/);
  assert.match(SYSTEM_RULES, /numbered steps/);
});
