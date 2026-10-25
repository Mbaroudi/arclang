// Unit tests of the toolbar's pure parts (run with `node --test`).
import test from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const viewer = resolve(dirname(fileURLToPath(import.meta.url)), '../../src/compiler/diagram/viewer');
require(join(viewer, 'arcviz-toolbar.js'));
const { shortcutLabel, tooltip, formatZoom, nextIndex, isMac } = globalThis.ArcVizToolbar;

test('shortcuts are written the way each platform writes them', () => {
  assert.equal(shortcutLabel('Mod+Z', true), '⌘Z');
  assert.equal(shortcutLabel('Mod+Shift+Z', true), '⇧⌘Z');
  assert.equal(shortcutLabel('Mod+Z', false), 'Ctrl+Z');
  assert.equal(shortcutLabel('Mod+Shift+Z', false), 'Ctrl+Shift+Z');
  assert.equal(shortcutLabel('Shift+M', true), '⇧M');
  assert.equal(shortcutLabel('+', true), '+');
  assert.equal(shortcutLabel('Mod++', false), 'Ctrl++');
  assert.equal(shortcutLabel('0', false), '0');
  assert.equal(shortcutLabel('', true), '');
});

test('a tooltip is the name, then the shortcut in parentheses', () => {
  assert.equal(tooltip('Undo move LC-CAM', 'Mod+Z', true), 'Undo move LC-CAM (⌘Z)');
  assert.equal(tooltip('Palette', '', true), 'Palette');
});

test('the zoom reads as a whole percentage', () => {
  assert.equal(formatZoom(1), '100 %');
  assert.equal(formatZoom(0.754), '75 %');
  assert.equal(formatZoom(1.996), '200 %');
  assert.equal(formatZoom(undefined), '0 %');
});

test('arrows wrap round a menu; Home and End go to its ends', () => {
  assert.equal(nextIndex(-1, 4, 'ArrowDown'), 0);
  assert.equal(nextIndex(-1, 4, 'ArrowUp'), 3);
  assert.equal(nextIndex(3, 4, 'ArrowDown'), 0);
  assert.equal(nextIndex(0, 4, 'ArrowUp'), 3);
  assert.equal(nextIndex(2, 4, 'Home'), 0);
  assert.equal(nextIndex(1, 4, 'End'), 3);
  assert.equal(nextIndex(1, 0, 'ArrowDown'), -1);
});

test('a Mac is told apart from other platforms', () => {
  assert.equal(isMac({ platform: 'MacIntel' }), true);
  assert.equal(isMac({ userAgentData: { platform: 'macOS' } }), true);
  assert.equal(isMac({ platform: 'Win32' }), false);
  assert.equal(isMac({ platform: 'Linux x86_64' }), false);
});
