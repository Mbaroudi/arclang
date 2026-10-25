// What the New commands write: names, the blank model, the skeleton, the layers
// a model lacks, a diagram declaration. With a compiler (ARCLANG_BINARY, the
// repository's build or the PATH), the compiler judges the text: `arclang
// check` for the new models, `arclang edit` for the layers and the diagram.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { test } from 'node:test';
import { arcadiaSkeleton, arcFileName, blankModel, diagramDeclaration, heldBlocks, LAYER_CHOICES, layerBlock, missingLayers, modelNameOf, uniqueName } from '../../src/pure/creation';
import { parseEditAnswer } from '../../src/pure/workbench';

const REFERENCE = fs.readFileSync(path.join(__dirname, '..', '..', 'media', 'reference.arc'), 'utf8');

function compiler(): string | null {
  const candidates = [process.env.ARCLANG_BINARY, path.resolve(__dirname, '../../../../target/release/arclang'), 'arclang'].filter((c): c is string => !!c);
  for (const candidate of candidates) if (spawnSync(candidate, ['--version'], { encoding: 'utf8' }).status === 0) return candidate;
  return null;
}
const ARCLANG = compiler();
const withCompiler = { skip: ARCLANG ? false : 'no arclang binary found (set ARCLANG_BINARY)' };

function inTemp<T>(text: string, use: (file: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arccreate-'));
  try {
    const file = path.join(dir, 'model.arc');
    fs.writeFileSync(file, text);
    return use(file);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

const check = (text: string) => inTemp(text, (file) => spawnSync(ARCLANG!, ['check', file], { encoding: 'utf8' }));
const edit = (text: string, request: unknown) =>
  inTemp(text, (file) => parseEditAnswer(spawnSync(ARCLANG!, ['edit', file, '--request', JSON.stringify(request)], { encoding: 'utf8' }).stdout));

test('a model name comes from the file name, always an identifier', () => {
  assert.equal(modelNameOf('/tmp/lane keeping.arc'), 'lane_keeping');
  assert.equal(modelNameOf('2024-plan.arc'), 'M_2024_plan');
  assert.equal(modelNameOf('---.arc'), 'Model');
  assert.equal(arcFileName('plan'), 'plan.arc');
  assert.equal(arcFileName('plan.ARC'), 'plan.ARC');
});

test('the blank model is a header; the skeleton holds the four levels', () => {
  assert.match(blankModel('plant'), /^model plant \{\n  name: "plant"\n  version: "0.1"\n\}\n$/);
  const skeleton = arcadiaSkeleton('plant');
  for (const keyword of ['operational_analysis', 'system_analysis', 'logical_architecture', 'physical_architecture']) assert.ok(heldBlocks(skeleton).has(keyword), keyword);
});

test('only the layers a model lacks are offered', () => {
  const lacking = missingLayers(arcadiaSkeleton('plant')).map((c) => c.keyword);
  assert.deepEqual(lacking, ['epbs', 'requirements', 'safety_analysis']);
  assert.equal(missingLayers(blankModel('plant')).length, LAYER_CHOICES.length);
  assert.ok(!missingLayers(REFERENCE).some((c) => c.keyword === 'requirements'));
});

test('a layer block and a diagram declaration are plain ArcLang', () => {
  assert.equal(layerBlock(LAYER_CHOICES[2]), 'logical_architecture "Logical Architecture" {\n}');
  assert.equal(layerBlock(LAYER_CHOICES.find((c) => c.keyword === 'requirements')!), 'requirements {\n}');
  assert.equal(diagramDeclaration('Chain', 'lab', ['LC-1', 'LC-2']), 'diagram "Chain" {\n  view: lab\n  shows: ["LC-1", "LC-2"]\n}');
  assert.equal(diagramDeclaration('Empty', 'sab', []), 'diagram "Empty" {\n  view: sab\n}');
  assert.equal(uniqueName('A', ['A', 'A 2']), 'A 3');
});

test('the compiler accepts the blank model and the skeleton', withCompiler, () => {
  for (const text of [blankModel('plant'), arcadiaSkeleton('plant')]) {
    const result = check(text);
    assert.equal(result.status, 0, result.stdout + result.stderr);
  }
});

test('the compiler adds each layer to the blank model, and a diagram to the skeleton', withCompiler, () => {
  for (const choice of LAYER_CHOICES) {
    const outcome = edit(blankModel('plant'), { op: 'add_raw', declaration: layerBlock(choice), top_level: true });
    assert.ok(outcome.ok, `${choice.keyword}: ${outcome.ok ? '' : outcome.error}`);
    if (outcome.ok) assert.ok(heldBlocks(outcome.source).has(choice.keyword));
  }
  const diagram = edit(arcadiaSkeleton('plant'), { op: 'add_raw', declaration: diagramDeclaration('First', 'lab', []), top_level: true });
  assert.ok(diagram.ok, diagram.ok ? '' : diagram.error);
});
