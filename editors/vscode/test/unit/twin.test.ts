import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { differenceInUnit, duration, inUnit, niceTicks, parseTwinOutcomes, twinChart, twinSummary, type TwinComparison, type TwinOutcome } from '../../src/pure/twin';

const celsius = { unit_factor: 1, unit_offset: 273.15 };

const comparison = (over: Partial<TwinComparison> = {}): TwinComparison => ({
  target: '"PN-SPACE".room.T', variable: 'livingSpace.room.T', column: 'indoor_c', tolerance: '0.5 K', tolerance_si: 0.5, unit: '°C', ...celsius,
  samples: 3, max_deviation: 0.6, rms_deviation: 0.3, within: 2 / 3, required: 0.99, first_exceedance: 1200, holds: false,
  curve: { time: [0, 600, 1200], measured: [294.15, 294.25, 294.95], simulated: [294.15, 294.3, 294.35] }, ...over,
});

test('a level in °C keeps its offset, a difference does not', () => {
  assert.equal(Math.round(inUnit(294.15, celsius) * 100) / 100, 21);
  assert.equal(differenceInUnit(0.5, celsius), 0.5);
  assert.equal(inUnit(10, { unit_factor: 1000 / 3600, unit_offset: 0 }), 36);
});

test('durations and ticks read as people read them', () => {
  assert.deepEqual([duration(45), duration(720), duration(5400), duration(9000), duration(86400 * 3)], ['45 s', '12 min', '90 min', '2.5 h', '3 d']);
  assert.deepEqual(niceTicks(20.8, 22.1, 5), [21, 21.5, 22]);
});

test('the chart draws the measurement, the model, the band and the first instant beyond', () => {
  const svg = twinChart(comparison());
  for (const part of ['class="band"', 'class="measured"', 'class="simulated"', 'class="exceed"', 'first beyond, 20 min', '>°C<']) assert.ok(svg.includes(part), part);
  assert.ok(!twinChart(comparison({ first_exceedance: null, holds: true })).includes('class="exceed"'));
  assert.ok(twinChart(comparison({ curve: { time: [], measured: [], simulated: [] } })).includes('No sample'));
});

test('names from the model are escaped in the chart', () => {
  assert.ok(twinChart(comparison({ target: '<b>x</b>' })).includes('&lt;b&gt;x&lt;/b&gt;'));
});

test('the outcomes are read and summed up', () => {
  const outcome = { twin: 'TW', name: 'W', verdict: 'pass' } as TwinOutcome;
  assert.equal(parseTwinOutcomes('not json'), null);
  assert.equal(parseTwinOutcomes(JSON.stringify([outcome]))?.length, 1);
  assert.equal(twinSummary([outcome, { ...outcome, verdict: 'fail' }]), '2 twins: 1 hold, 1 depart from the data');
});
