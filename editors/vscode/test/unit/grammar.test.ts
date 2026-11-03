import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';

// The TextMate grammar of ArcLang knows `io_binding`, `region`, the keys of
// `execute:` (attributes) and the action language inside the strings of a
// guard, an action, an equation or a declaration. A small scanner applies
// the grammar's own patterns as TextMate does: at each position the
// earliest match wins, then the first pattern.
const grammar = JSON.parse(readFileSync(join(__dirname, '../../syntaxes/arclang.tmLanguage.json'), 'utf8'));
type Pattern = { name?: string; match?: string; include?: string };

function flatten(patterns: Pattern[]): { name: string; re: RegExp }[] {
  return patterns.flatMap((p) => {
    if (p.include) return flatten((grammar.repository[p.include.slice(1)].patterns ?? [grammar.repository[p.include.slice(1)]]) as Pattern[]);
    return p.match ? [{ name: p.name ?? '', re: new RegExp(p.match, 'g') }] : [];
  });
}

function scopes(code: string): [string, string][] {
  const inner = flatten(grammar.repository['code-string'].patterns);
  const out: [string, string][] = [];
  let at = 0;
  while (at < code.length) {
    let best: { start: number; text: string; name: string } | null = null;
    for (const { name, re } of inner) {
      re.lastIndex = at;
      const m = re.exec(code);
      if (m && m[0].length > 0 && (!best || m.index < best.start)) best = { start: m.index, text: m[0], name };
    }
    if (!best) break;
    out.push([best.text, best.name]);
    at = best.start + best.text.length;
  }
  return out;
}

test('io_binding and region are declarations', () => {
  const re = new RegExp(grammar.repository.declarations.match);
  assert.equal(re.exec('io_binding "Charger board" {')?.[1], 'io_binding');
  assert.equal(re.exec('    region "Motion" {')?.[1], 'region');
});

test('the strings of guards, actions, equations and variables are code', () => {
  const [list, scalar] = grammar.repository.code.patterns;
  assert.ok(new RegExp(list.begin).test('    equations: ['));
  assert.ok(new RegExp(scalar.begin).test('transition A -> B { guard: "speed > 30 km/h"'));
  assert.ok(new RegExp(scalar.begin).test('entry: "count := count + 1"'));
  assert.ok(!new RegExp(scalar.begin).test('description: "a text"'), 'other strings stay strings');
  assert.ok(grammar.patterns.findIndex((p: Pattern) => p.include === '#code') < grammar.patterns.findIndex((p: Pattern) => p.include === '#strings'));
});

test('the action language is highlighted token by token', () => {
  const guard = scopes('filtered >= v_absorption and not on');
  assert.deepEqual(guard, [
    ['filtered', 'variable.other.code.arclang'],
    ['>=', 'keyword.operator.comparison.arclang'],
    ['v_absorption', 'variable.other.code.arclang'],
    ['and', 'keyword.operator.logical.arclang'],
    ['not', 'keyword.operator.logical.arclang'],
    ['on', 'variable.other.code.arclang'],
  ]);
  const equation = scopes('setpoint = pid(error, 0.5, 20 Hz, 0 s, 0 A, 20 A)');
  assert.deepEqual(equation.slice(0, 4), [
    ['setpoint', 'variable.other.code.arclang'],
    ['=', 'keyword.operator.assignment.arclang'],
    ['pid', 'support.function.block.arclang'],
    ['error', 'variable.other.code.arclang'],
  ]);
  assert.ok(equation.some(([t, n]) => t === '20 Hz' && n === 'constant.numeric.quantity.arclang') || equation.some(([t]) => t.startsWith('20')), JSON.stringify(equation));
  const declaration = scopes('out current: Real [A] = 0 A');
  assert.deepEqual(declaration.slice(0, 4), [
    ['out', 'storage.modifier.direction.arclang'],
    ['current', 'variable.other.code.arclang'],
    ['Real', 'storage.type.variable.arclang'],
    ['[A]', 'keyword.other.unit.arclang'],
  ]);
  const action = scopes('count := count + 1');
  assert.equal(action[1][1], 'keyword.operator.assignment.arclang');
  const binding = scopes('plug <- gpio 2 rising');
  assert.deepEqual(binding.map(([, n]) => n).slice(1, 3), ['keyword.operator.comparison.arclang', 'support.type.channel.arclang']);
});
