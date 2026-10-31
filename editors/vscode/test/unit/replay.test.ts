import assert from 'node:assert/strict';
import { test } from 'node:test';
import { failingAt, firstFailure, parseReplay, Replay, stepAt, tagsAt, timeLabel, valueAt, valueLabel } from '../../src/pure/replay';

const REPLAY: Replay = {
  case: 'TC-LOSS-25',
  point: 0,
  points: [],
  stale: false,
  time: [0, 1200, 2400],
  elements: [
    { id: 'PN-ESS', instance: 'essentialLoads', values: [{ name: 'voltage', unit: 'V', series: [27.26, 25.0, 23.73] }] },
    { id: 'PN-GEN', instance: 'starterGenerator', values: [{ name: 'online', series: [1, 0, 0] }] },
  ],
  expectations: [
    { text: 'min("PN-ESS".voltage) >= 24 V', requirement: 'REQ-ESS-30MIN', holds: false, value: 'min = 23.73 V', margin: 'short by 0.27 V', first_failure: 2246.4, element: 'PN-ESS' },
    { text: 'at(40 min, "PN-NESS".drawn) <= 1 W', holds: true, value: '0 W', margin: '1 W to spare', element: 'PN-NESS' },
  ],
};

test('a value between two steps is interpolated', () => {
  assert.equal(stepAt(REPLAY.time, 1800), 1);
  assert.equal(stepAt(REPLAY.time, -5), 0);
  assert.equal(stepAt(REPLAY.time, 9999), 2);
  assert.ok(Math.abs(valueAt(REPLAY.time, [27.26, 25.0, 23.73], 600) - 26.13) < 1e-9);
  assert.equal(valueAt(REPLAY.time, [27.26, 25.0, 23.73], 2400), 23.73);
});

test('times and values read the way a reviewer says them', () => {
  assert.equal(timeLabel(2246.4), '37 min 26 s');
  assert.equal(timeLabel(3.24), '3.2 s');
  assert.equal(timeLabel(7260), '2 h 1 min');
  assert.equal(valueLabel(23.7271, 'V'), '23.73 V');
  assert.equal(valueLabel(0.5012, '1'), '0.5012');
  assert.equal(valueLabel(1), 'yes');
  assert.equal(valueLabel(0), 'no');
  assert.equal(valueLabel(350000, 'W'), '3.500e+5 W');
});

test('each box shows its values at the instant, and the elements whose expectation has failed by then', () => {
  assert.deepEqual(tagsAt(REPLAY, 0).get('PN-ESS'), ['voltage 27.26 V']);
  assert.deepEqual(tagsAt(REPLAY, 2400).get('PN-GEN'), ['online no']);
  assert.equal(failingAt(REPLAY, 2000).size, 0);
  assert.deepEqual([...failingAt(REPLAY, 2300).keys()], ['PN-ESS']);
  assert.equal(firstFailure(REPLAY), 2246.4);
  assert.equal(firstFailure({ ...REPLAY, expectations: [REPLAY.expectations[1]] }), null);
});

test('only the compiler\'s replay is read', () => {
  assert.equal(parseReplay('not json'), null);
  assert.equal(parseReplay('{"time": 3}'), null);
  assert.equal(parseReplay(JSON.stringify(REPLAY))?.case, 'TC-LOSS-25');
});
