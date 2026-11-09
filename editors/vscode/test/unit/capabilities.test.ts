// What the AI is told ArcLang can do: it guides (simulation, embedded code,
// twins...), says what runs where the engineer is, and names only commands
// the extension has.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { readdirSync } from 'node:fs';
import { capabilitiesGuide, TOPICS } from '../../src/ai/core/capabilities';
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
  assert.match(online, /Here: click Simulate at the bottom of the diagram[^\n]*it runs on the server \(OpenModelica/);
  assert.match(online, /compiling and running it \(sil\) needs the desktop/);
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

// The pages of arclang.fr/docs/ (arclang-web's docs-src and the repository's
// pages it publishes). A page added to the site is added here, and the guide
// must then say what it is about.
const SITE_PAGES = ['', 'api', 'arcadia', 'capella', 'cesam', 'cesam-tutorial', 'ci-cd', 'cli', 'cosimulation', 'diagrams',
  'embedded-c', 'embedded-fpga', 'embedded-system', 'embedded-targets', 'exchange-formats', 'getting-started', 'grammar',
  'language', 'mbse', 'metamodel', 'migration-4.0', 'platform', 'production-gate', 'realization', 'review-and-change', 'rules',
  'semantic-git', 'simulation', 'studio', 'sysml-v2', 'sysml-v2-modeling', 'tutorial', 'types-and-constraints', 'versioning', 'vscode'];

test('every page of the documentation is a topic of the guide', () => {
  const linked = new Set(TOPICS.flatMap((t) => t.docs));
  for (const page of SITE_PAGES.filter(Boolean)) assert.ok(linked.has(page), `arclang.fr/docs/${page}/ is in no topic`);
  for (const page of linked) assert.ok(page.startsWith('docs/') || SITE_PAGES.includes(page), `${page} is no page of arclang.fr/docs/`);
});

test('every document of the repository is a topic of the guide, by its page or its file', () => {
  const linked = new Set(TOPICS.flatMap((t) => t.docs));
  const docs = join(__dirname, '../../../../docs');
  for (const file of readdirSync(docs).filter((f) => f.endsWith('.md') && f !== 'README.md')) {
    const slug = file.replace(/\.md$/, '').toLowerCase().replace(/_/g, '-');
    assert.ok(linked.has(slug) || linked.has(`docs/${file}`), `docs/${file} is in no topic`);
  }
  for (const page of linked) if (page.startsWith('docs/')) assert.ok(readdirSync(docs).includes(page.slice(5)), `${page} does not exist`);
});

// arclang --help, the commands a user runs (not the internal ones: build, sync, plugin, explorer, repl, clean, info, review).
const CLI = ['check', 'transition', 'rules', 'format', 'new', 'trace', 'gate', 'diff', 'history', 'merge', 'restore', 'baseline', 'ci',
  'report', 'sarif', 'impact', 'debug', 'verify', 'cosim', 'sil', 'pil', 'hdl', 'soc', 'partition', 'schedule', 'realize', 'twin',
  'document', 'export', 'import', 'safety', 'serve', 'lsp', 'set', 'rename', 'workbench', 'edit', 'table', 'metamodel', 'diagram'];

test('every command of the command line is in the guide', () => {
  const guide = capabilitiesGuide('desktop');
  for (const command of CLI) assert.match(guide, new RegExp(`\\barclang ${command}\\b|arclang [a-z, /]*\\b${command}\\b`), `arclang ${command} is not in the guide`);
});

test('CESAM, co-simulation, embedded C, FPGA, SoC and twins say what runs online and what needs the desktop', () => {
  const online = capabilitiesGuide('online');
  for (const what of ['co-simulated on the server in OMSimulator', 'Processor in the loop (pil) needs the desktop', 'The SoC (soc) needs the desktop', 'the export of the VHDL runs on the server', 'the export of the C runs on the server', 'live sources need the desktop', 'its compiler has CESAM']) {
    assert.ok(online.includes(what), `missing: ${what}`);
  }
});

test('online, the guide never sends a visitor to a command the palette hides there', () => {
  const when = new Map<string, string>((pkg.contributes.menus.commandPalette ?? []).map((m: { command: string; when?: string }) => [m.command, m.when ?? '']));
  const title = (c: { title: string; category?: string }) => `${c.category ? `${c.category}: ` : ''}${c.title}`;
  const commands = pkg.contributes.commands as { command: string; title: string; category?: string }[];
  const never = commands.filter((c) => when.get(c.command) === 'false').map(title);
  const desktop = commands.filter((c) => /!isWeb/.test(when.get(c.command) ?? '')).map(title);
  for (const workspace of ['online', 'online-anonymous'] as const) {
    const guide = capabilitiesGuide(workspace);
    for (const t of never) assert.ok(!guide.includes(`"${t}"`), `"${t}" is hidden everywhere`);
    // Each topic that names a desktop-only command says so where the visitor reads it.
    for (const topic of guide.split('\n- ')) {
      const cited = desktop.filter((t) => topic.includes(`"${t}"`));
      if (cited.length) assert.match(topic, /desktop/, `${topic.slice(0, 60)}… names ${cited.join(', ')} without saying it is the desktop's`);
    }
  }
  assert.match(capabilitiesGuide('online-anonymous'), /sign in first[^\n]*Simulate at the bottom of the diagram/);
});
