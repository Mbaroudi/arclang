import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import { expectsOf, unreadableExpects } from '../../src/pure/sim-expects';

const MO = `package P
  model Water
    parameter Real P(unit = "W") = 2000 "Heater power";
    Real T(unit = "K", start = 293.15, fixed = true) "Water temperature";
    Real level "Share of the tank";
  equation
    der(T) = P / 1000;
    level = 1;
  end Water;
  model Hydraulic
    Water water;
  end Hydraulic;
  model Heating
    Hydraulic plant;
  end Heating;
end P;`;

const tc = (expects: string[]) => `test_case "TC-1" {\n  simulate: { model: "P.Heating" stop_time: 60 s sources: ["physics/P.mo"] }\n  expects: [${expects.map((e) => JSON.stringify(e)).join(', ')}]\n}\n`;

test('the expects of a model are read, quotes unescaped', () => {
  assert.deepEqual(expectsOf(tc(['REQ-1: max("PN-HYD".water.T) <= 96 °C', 'REQ-1: min(plant.water.level) >= 0.1'])), ['REQ-1: max("PN-HYD".water.T) <= 96 °C', 'REQ-1: min(plant.water.level) >= 0.1']);
});

test('a path of the Modelica result, with the unit of its variable, can be judged', () => {
  assert.deepEqual(unreadableExpects('', tc(['REQ-1: max(plant.water.T) <= 96 °C', 'REQ-1: min(plant.water.level) >= 0.1']), MO), []);
});

test('an element of the model, a name declared nowhere, a bound without its unit: each is sent back', () => {
  const problems = unreadableExpects('', tc(['REQ-1: max("PN-HYD".water.T) <= 96 °C', 'REQ-1: max(plant.core.T) <= 96 °C', 'REQ-1: max(plant.water.T) <= 369', 'REQ-1: min(plant.water.level) >= 10 %']), MO);
  assert.equal(problems.length, 4);
  assert.match(problems[0], /an element of the model/);
  assert.match(problems[1], /core is declared nowhere/);
  assert.match(problems[2], /unit "K"\) with a plain number/);
  assert.match(problems[3], /has none in the Modelica/);
});

test('only what the proposal adds is checked; no Modelica, nothing to say', () => {
  const old = tc(['REQ-1: max("PN-HYD".water.T) <= 96 °C']);
  assert.deepEqual(unreadableExpects(old, old, MO), []);
  assert.deepEqual(unreadableExpects('', tc(['REQ-1: max(x) <= 1']), ''), []);
});

test('a package that extends the generated structure may read elements of the model', () => {
  assert.deepEqual(unreadableExpects('', tc(['REQ-1: max("PN-HYD".core.T) <= 96 °C']), `model Heating\n  extends Coffee.Architecture;\nend Heating;`), []);
  assert.equal(unreadableExpects('', tc(['REQ-1: max("PN-HYD".core.T) <= 96 °C']), `model Heating\n  extends Modelica.Icons.Example;\n  Real T(unit = "K");\nend Heating;`).length, 1);
});

test('a Modelica keyword as a name is sent back: the package would not load', () => {
  const mo = `package B\n  model Coolant\n    Real T(unit = "K");\n  end Coolant;\n  model Charge\n    Coolant loop;\n  end Charge;\nend B;`;
  const problems = unreadableExpects('', tc(['REQ-1: max(loop.T) <= 40 °C']), mo);
  assert.equal(problems.length, 1);
  assert.match(problems[0], /"loop", a Modelica keyword/);
});
