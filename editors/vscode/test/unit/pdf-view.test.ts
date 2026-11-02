import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { currentPage, fitScale, stepZoom } from '../../src/pure/pdf-view';

test('a zoom step from a fitted scale goes to the next step in that direction', () => {
  assert.equal(stepZoom(1, 1), 1.1);
  assert.equal(stepZoom(1, -1), 0.9);
  assert.equal(stepZoom(1.37, 1), 1.5);
  assert.equal(stepZoom(1.37, -1), 1.25);
});

test('zoom stops at its bounds', () => {
  assert.equal(stepZoom(4, 1), 4);
  assert.equal(stepZoom(0.25, -1), 0.25);
});

test('a 16:9 page fits the width of a wide view, or the whole page in its height', () => {
  const page = { width: 960, height: 540 };
  const view = { width: 1200, height: 500 };
  assert.equal(fitScale('width', page, view), (1200 - 48) / 960);
  assert.equal(fitScale('page', page, view), (500 - 48) / 540);
});

test('the page shown is the last whose top is above a third of the view', () => {
  const tops = [0, 600, 1200];
  assert.equal(currentPage(tops, 0, 900), 1);
  assert.equal(currentPage(tops, 400, 900), 2);
  assert.equal(currentPage(tops, 1100, 900), 3);
});
