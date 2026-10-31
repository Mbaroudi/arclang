import assert from 'node:assert/strict';
import { test } from 'node:test';
import { axisStart, histogram, isConstant, monteCarloCases, niceStep, parseStatistics, probabilityText, shortNumber, shortParameter } from '../../src/pure/montecarlo';
import { monteCarloHtml } from '../../src/pure/montecarlo-html';
import { outcomesLog, parseOutcomes } from '../../src/pure/simulation';

// What `arclang verify --json` prints for a Monte Carlo case: 12 samples, the last two short of charge.
const sample = (i: number) => {
  const soc = 0.25 - i * 0.016;
  return {
    label: `sample ${i + 1} (ambient = ${253 + i} K, target_speed = ${90 + i} km/h)`,
    holds: soc >= 0.1,
    inputs: [['vehicleBody.ambient', 253 + i], ['vehicleControlUnit.speedController.target_speed', (90 + i) / 3.6]],
    values: [
      { expect: 'final(vehicleBody.distance) >= 300 km', value: 'final(vehicleBody.distance) = 300 km', holds: true, margin: 'at the bound', slack: 6e-5 + i * 1e-10, slack_unit: { symbol: 'km', factor: 1000 } },
      { expect: 'min("PN-BATT".soc) >= 0.1', value: `min(batteryPack.soc) = ${soc.toFixed(3)}`, holds: soc >= 0.1, margin: '', slack: soc - 0.1 },
    ],
  };
};
const RUN = JSON.stringify([
  { test_case: 'TC-RANGE-STD', verdict: 'pass', reason: '', values: [] },
  {
    test_case: 'TC-RANGE-LR',
    name: 'Winter range, long range pack',
    verdict: 'fail',
    reason: 'holds in 10 of 12 samples: probability between 61.5 % and 100 % at 95 % confidence, not shown to reach the 90 % required',
    values: [],
    points: Array.from({ length: 12 }, (_, i) => sample(i)),
    probability: { probability: 0.9, confidence: 0.95 },
    estimate: { held: 10, total: 12, low: 0.6151, high: 0.9848 },
    sensitivity: [
      { expect: 'min("PN-BATT".soc) >= 0.1', parameter: 'vehicleControlUnit.speedController.target_speed', rank_correlation: -0.98 },
      { expect: 'min("PN-BATT".soc) >= 0.1', parameter: 'vehicleBody.ambient', rank_correlation: 0.31 },
    ],
  },
]);

test('only the cases that sample are Monte Carlo cases, read with their estimate, units and failing samples', () => {
  const outcomes = parseOutcomes(RUN)!;
  assert.equal(outcomes[0].probability, null, 'a plain case requires no probability');
  const [c] = monteCarloCases(outcomes);
  assert.equal(monteCarloCases(outcomes).length, 1);
  assert.equal(c.name, 'Winter range, long range pack');
  assert.deepEqual(c.estimate, { held: 10, total: 12, low: 0.6151, high: 0.9848 });
  assert.deepEqual(c.parameters, ['vehicleBody.ambient', 'vehicleControlUnit.speedController.target_speed']);
  assert.deepEqual(c.failing.map((f) => f.index), [10, 11], 'a failing sample keeps its place, for its replay');
  assert.deepEqual(c.failing[0].failed, ['min(batteryPack.soc) = 0.090']);
  const [distance, soc] = c.margins;
  assert.equal(distance.unit, 'km');
  assert.ok(distance.constant, 'a run that ends on its bound does not vary');
  assert.equal(distance.bins.length, 1);
  assert.deepEqual(distance.influences, []);
  assert.equal(soc.unit, '');
  assert.equal(soc.failing, 2);
  assert.equal(soc.influences[0].rho, -0.98);
  assert.equal(soc.bins.reduce((n, b) => n + b.count, 0), 12, 'every sample is in a bin');
  assert.ok(soc.bins.filter((b) => b.failing).every((b) => b.to <= 0), 'a failing bin lies below the bound');
  assert.ok(soc.bins.filter((b) => !b.failing).every((b) => b.from >= -1e-12), 'the bins meet at the bound');
});

test('an older compiler, or a case that sweeps, has no Monte Carlo fields', () => {
  const outcomes = parseOutcomes(JSON.stringify([{ test_case: 'T', verdict: 'pass', reason: '', values: [{ value: 'x = 1', expect: 'x >= 0', holds: true, slack: 'NaN', slack_unit: { symbol: 'm', factor: 0 } }] }]))!;
  assert.equal(outcomes[0].probability, null);
  assert.equal(outcomes[0].estimate, null);
  assert.deepEqual(outcomes[0].sensitivity, []);
  assert.deepEqual(outcomes[0].values[0], { value: 'x = 1', expect: 'x >= 0', holds: true }, 'a slack that is not a number, a unit of factor 0: dropped');
});

test('the log of a Monte Carlo case lists the failing samples and what moves the margins, never every sample', () => {
  const log = outcomesLog(parseOutcomes(RUN)!);
  assert.match(log, /Monte Carlo: 12 samples, 10 hold; required: 90 % at 95 % confidence/);
  assert.match(log, /FAIL sample 11 \(ambient = 263 K/);
  assert.doesNotMatch(log, /sample 3 \(/, 'a holding sample is not listed');
  assert.match(log, /-0\.98 {2}vehicleControlUnit\.speedController\.target_speed {2}on {2}min\("PN-BATT"\.soc\) >= 0\.1/);
  assert.match(log, /not shown to reach the 90 % required/, 'a failing Monte Carlo case says why');
});

test('histograms, numbers and the probability axis read well', () => {
  assert.equal(niceStep(0.013), 0.02);
  assert.equal(niceStep(3), 5);
  assert.equal(niceStep(0.1), 0.1);
  const bins = histogram([-0.03, -0.01, 0.02, 0.05, 0.11]);
  assert.ok(bins.some((b) => b.to === 0 || Math.abs(b.to) < 1e-12), 'an edge sits on the bound');
  assert.deepEqual(histogram([]), []);
  assert.equal(histogram([5, 5, 5]).length, 1);
  assert.ok(isConstant([6e-5, 6.0000001e-5]));
  assert.ok(!isConstant([0.14, 0.25]));
  assert.equal(shortNumber(0.141118), '0.1411');
  assert.equal(shortNumber(20500), '20500');
  assert.equal(shortNumber(3e-7), '3e-7');
  assert.equal(shortParameter('vehicleControlUnit.speedController.target_speed'), 'speedController.target_speed');
  assert.equal(shortParameter('body.mass'), 'body.mass');
  assert.equal(axisStart({ held: 40, total: 40, low: 0.9278, high: 1 }, 0.9), 0.85);
  assert.equal(axisStart(null, 0.02), 0);
  assert.equal(probabilityText(0.0025), '0.25 %');
  assert.equal(probabilityText(3.1e-7), '3.1e-7');
  assert.equal(probabilityText(null), '?');
});

test('the report of R or Python is read, and anything else is not', () => {
  const report = parseStatistics(JSON.stringify({
    engine: 'R 4.4.2 (base R)',
    margins: [
      { expect: 'min(soc) >= 0.1', samples: 40, failure_fraction: 0, fitted_failure_probability: 2e-9, constant: false, influences: [{ method: 'PRCC', parameter: 'a.speed', value: -0.91, low: -0.95, high: -0.85 }, { method: 'PRCC', parameter: 'a.air', value: null, low: null, high: null }] },
      { expect: 'final(d) >= 300 km', samples: 40, constant: true, influences: [] },
    ],
  }))!;
  assert.equal(report.engine, 'R 4.4.2 (base R)');
  assert.equal(report.margins[0].influences[1].value, null, 'an undefined influence stays undefined');
  assert.ok(report.margins[1].constant);
  assert.equal(parseStatistics('not json'), null);
  assert.equal(parseStatistics('{"margins": 3}'), null);
});

test('the page escapes what it shows, draws each chart, and offers a replay for each failing sample', () => {
  const outcomes = parseOutcomes(RUN)!;
  const cases = monteCarloCases(outcomes);
  cases[0].name = '<img src=x onerror=alert(1)>';
  const html = monteCarloHtml({ file: 'ev.arc', cases, statistics: { 'TC-RANGE-LR': { state: 'error', message: 'Rscript does not run: install R <now>' } }, stale: true }, 'NONCE', 'vscode-resource:');
  assert.ok(!html.includes('<img src=x'), 'a name is escaped');
  assert.match(html, /&lt;img src=x onerror=alert\(1\)&gt;/);
  assert.match(html, /script-src 'nonce-NONCE'/);
  assert.doesNotMatch(html, / style="/, 'no inline style: the CSP would block it');
  assert.match(html, /class="gauge"[^>]*aria-label="10 of 12 samples hold: at 95 % confidence, the probability is between 61\.51 % and 98\.48 %; 90 % is required\."/);
  assert.match(html, /required 90 %/);
  assert.match(html, /class="band not-shown"/, 'an interval below the requirement reads as not shown');
  assert.equal((html.match(/class="histogram"/g) ?? []).length, 2);
  assert.match(html, /does not vary over the samples/);
  assert.match(html, /data-replay="TC-RANGE-LR" data-index="10"/);
  assert.match(html, /data-replay="TC-RANGE-LR" data-index="11"/);
  assert.match(html, /data-replay="TC-RANGE-LR" data-index="-1"/, 'any sample can be picked');
  assert.match(html, /install R &lt;now&gt;/);
  assert.match(html, /The model changed since these results/);
  assert.match(monteCarloHtml({ file: 'x.arc', cases: [], statistics: {}, stale: false }, 'N', 'c'), /No test case of this model samples/);
});

test('an end of the axis the bound would cover is left out', () => {
  const outcomes = parseOutcomes(RUN)!;
  const [c] = monteCarloCases(outcomes);
  // A margin from -18 to 2 km: the bound sits near the right end, whose label would sit under it.
  const margin = { ...c.margins[1], unit: 'km', constant: false, smallest: -18, largest: 2, bins: Array.from({ length: 10 }, (_, i) => ({ from: -18 + 2 * i, to: -16 + 2 * i, count: i === 9 ? 195 : 1, failing: i < 9 })) };
  const html = monteCarloHtml({ file: 'ev.arc', cases: [{ ...c, margins: [margin] }], statistics: {}, stale: false }, 'N', 'c');
  const histogramSvg = html.slice(html.indexOf('class="histogram"'), html.indexOf('</svg>', html.indexOf('class="histogram"')));
  assert.match(histogramSvg, />-18 km</, 'the far end is labelled');
  assert.match(histogramSvg, />bound</);
  assert.doesNotMatch(histogramSvg, />2 km</, 'the near end is not drawn under the bound');
});

